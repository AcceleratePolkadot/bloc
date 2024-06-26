use crate::*;
use frame_support::pallet_prelude::*;
use sp_runtime::Percent;

pub struct NominationCalls<T> {
    _phantom: PhantomData<T>
}


impl<T: Config> NominationCalls<T> {

    fn in_voting_period(nomination: &Nomination<T>) -> Result<bool, pallet::Error<T>> {
        ensure!(nomination.status !=  NominationStatus::Approved, Error::<T>::NominationAlreadyApproved);
        ensure!(nomination.status !=  NominationStatus::Rejected, Error::<T>::NominationAlreadyRejected);
        ensure!(nomination.nominated_on + T::NominationVotingPeriod::get() >= <frame_system::Pallet<T>>::block_number(), Error::<T>::NominationVotingPeriodEnded);
        Ok(true)
    }

    fn calculate_votes_threshold(roster: &Roster<T>, nomination: &Nomination<T>) -> Result<u32, pallet::Error<T>> {
        /* Calculate what percentage of the nomination's voting period has passed
        *  p = voting period duration in blocks
        *  n = block nominated on
        *  b = current block
        *
        *  percentage of voting period passed = ((b - n) / p) * 100
        */
        let p = TryInto::<u32>::try_into(T::NominationVotingPeriod::get()).map_err(|_| Error::<T>::ConversionError)?;
        let n = TryInto::<u32>::try_into(nomination.nominated_on).map_err(|_| Error::<T>::ConversionError)?;
        let b = TryInto::<u32>::try_into(<frame_system::Pallet<T>>::block_number()).map_err(|_| Error::<T>::ConversionError)?;
        let blocks_passed = b - n;
        let fraction_passed = match blocks_passed.checked_div(p) {
            Some(result) => result,
            None => return Err(Error::<T>::ConversionError),
        };


        /* Calculate the threshold of same votes required to end voting period early
        *  To pass the threshold the percentage of members who have voted in the same way must be greater or equal 
        *  to the percentage of the voting period remaining, plus an additional buffer.
        *
        *  The minimum threshold required is always 50%.
        *
        *  For example:
        *  p = 25 (percentage of voting period passed)
        *  m = 300 (total number of members)
        *
        * (m / 100)(100 - p * 0.8) = 240 members would need to vote the same way to close the nomination early
        */
        let fraction_remaining = 1 - (Percent::from_percent(80) * fraction_passed);
        let threshold_percentage = match fraction_remaining.checked_mul(100) {
            Some(result) if result < 50 => Percent::from_percent(50),
            Some(result) => Percent::from_percent(result.try_into().unwrap_or(100)),
            None => return Err(Error::<T>::ConversionError),
        };

        let threshold = threshold_percentage * roster.members.len() as u32;

        Ok(threshold)
    }

    pub(crate) fn new(nominator: T::AccountId, roster_id: RosterId, nominee: T::AccountId) -> DispatchResultWithPostInfo {

        // Roster must exist and be active.
        let mut nomination_roster = Rosters::<T>::get(&roster_id).ok_or(Error::<T>::RosterDoesNotExist)?;
        ensure!(nomination_roster.status == RosterStatus::Active, Error::<T>::RosterNotActive);

        // Nominator must be a member of the roster.
        ensure!(nomination_roster.members.contains(&nominator), Error::<T>::PermissionDenied);

        // Nominee must not be an existing member of the roster.
        ensure!(!nomination_roster.members.contains(&nominee), Error::<T>::AlreadyMember);

        // Nominee must not have already been nominated
        ensure!(!Nominations::<T>::contains_key(&nominee, &roster_id), Error::<T>::AlreadyNominated);

        // Create new nomination
        let nomination = Nomination::new(&roster_id, &nominee, &nominator);
        Nominations::<T>::insert(&nominee, &roster_id, &nomination);
        nomination_roster.nominations.try_push(nominee.clone()).map_err(|_| Error::<T>::CouldNotAddNomination)?;
        Rosters::<T>::insert(&roster_id, nomination_roster);

        pallet::Pallet::deposit_event(Event::<T>::NewNomination(nominator, nominee, roster_id));

        Ok(().into())
    }

    pub(crate) fn vote(voter: T::AccountId, roster_id: RosterId, nominee: T::AccountId, vote: NominationVoteValue) -> DispatchResultWithPostInfo {
        // Check if nomination is in voting period
        let mut nomination = Nominations::<T>::get(&nominee, &roster_id).ok_or(Error::<T>::NominationDoesNotExist)?;
        Self::in_voting_period(&nomination)?;


        // Voter must be a member of the roster
        let roster = Rosters::<T>::get(&roster_id).ok_or(Error::<T>::RosterDoesNotExist)?;
        ensure!(roster.members.contains(&voter), Error::<T>::PermissionDenied);

        // Voter must not have already voted.
        // If a voter wishes to change their vote they must first recant their vote.
        ensure!(!nomination.votes.iter().any(|v| v.voter == voter), Error::<T>::AlreadyVoted);

        // Add vote to nomination
        let nomination_vote = NominationVote { voter: voter.clone(), vote: vote.clone() };
        nomination.votes.try_push(nomination_vote).map_err(|_| Error::<T>::CouldNotAddVote)?;
        Nominations::<T>::insert(&nominee, &roster_id, nomination);
        
        pallet::Pallet::deposit_event(Event::<T>::Voted(voter, vote, nominee, roster_id));

        Ok(().into())
    }

    pub(crate) fn recant_vote(voter: T::AccountId, roster_id: RosterId, nominee: T::AccountId) -> DispatchResultWithPostInfo {
        // Check if nomination is in voting period
        let mut nomination = Nominations::<T>::get(&nominee, &roster_id).ok_or(Error::<T>::NominationDoesNotExist)?;
        Self::in_voting_period(&nomination)?;

        // Voter must have voted in order to recant vote.
        ensure!(nomination.votes.iter().any(|v| v.voter == voter), Error::<T>::NotVoted);

        // Remove vote from nomination
        nomination.votes.retain(|v| v.voter != voter);
        Nominations::<T>::insert(&nominee, &roster_id, nomination);

        pallet::Pallet::deposit_event(Event::<T>::VoteRecanted(voter, nominee, roster_id));

        Ok(().into())
    }

    pub(crate) fn close(who: T::AccountId, roster_id: RosterId, nominee: T::AccountId) -> DispatchResultWithPostInfo {
        // Check if nomination is in voting period
        let mut nomination = Nominations::<T>::get(&nominee, &roster_id).ok_or(Error::<T>::NominationDoesNotExist)?;
        Self::in_voting_period(&nomination)?;

        // Voting can end if the voting period has passed
        // or if the ayes or nays have exceeded the voting threshold
        let mut roster = Rosters::<T>::get(&roster_id).ok_or(Error::<T>::RosterDoesNotExist)?;
        let ayes = nomination.votes.iter().filter(|v| v.vote == NominationVoteValue::Aye).count() as u32;
        let nays = nomination.votes.iter().filter(|v| v.vote == NominationVoteValue::Nay).count() as u32;
        let votes_threshold = Self::calculate_votes_threshold(&roster, &nomination)?;

        ensure!(nomination.nominated_on + T::NominationVotingPeriod::get() < <frame_system::Pallet<T>>::block_number() || ayes >= votes_threshold || nays >= votes_threshold,  Error::<T>::NominationVotingPeriodHasNotEnded);

        nomination.status = match ayes > nays {
            true => NominationStatus::Approved,
            false => NominationStatus::Rejected,
        };

        Nominations::<T>::insert(&nominee, &roster_id, &nomination);

        // If nomination has been accepted add nominee to roster members
        if nomination.status == NominationStatus::Approved {
            roster.members.try_push(nominee.clone()).map_err(|_| Error::<T>::CouldNotAddMember)?;
            Rosters::<T>::insert(&roster_id, roster);
        }

        ConcludedNominations::<T>::try_append((&nominee, &roster_id)).map_err(|_| Error::<T>::CouldNotAddToConcluded)?;

        pallet::Pallet::deposit_event(Event::<T>::NominationClosed(nominee, roster_id, who, nomination.status));

        Ok(().into())
                    
    }
}
