use crate::*;
use frame_support::{pallet_prelude::*, traits::NamedReservableCurrency};
use sp_runtime::{traits::Saturating, Percent};

pub struct NominationCalls<T> {
	_phantom: PhantomData<T>,
}

impl<T: Config> NominationCalls<T> {
	fn in_voting_period(nomination: &Nomination<T>) -> Result<bool, pallet::Error<T>> {
		ensure!(
			nomination.status != NominationStatus::Approved,
			Error::<T>::NominationAlreadyApproved
		);
		ensure!(
			nomination.status != NominationStatus::Rejected,
			Error::<T>::NominationAlreadyRejected
		);
		ensure!(
			nomination.nominated_on + T::NominationVotingPeriod::get()
				>= <frame_system::Pallet<T>>::block_number(),
			Error::<T>::VotingPeriodEnded
		);
		Ok(true)
	}

	fn calculate_percentage_elapsed(
		nomination: &Nomination<T>,
	) -> Result<Percent, pallet::Error<T>> {
		let voting_period = T::NominationVotingPeriod::get();
		let start_block = nomination.nominated_on;
		let current_block = <frame_system::Pallet<T>>::block_number();

		let elapsed_blocks = current_block.saturating_sub(start_block);
		let percentage_elapsed = Percent::from_rational(elapsed_blocks, voting_period);

		Ok(percentage_elapsed)
	}

	// Calculate the threshold of votes required to reach quorum
	// The percentage of votes required is equal to the percentage of the
	// voting period that is remaining times `QuorumModifier` percent.
	//
	// The minimum votes required is `QuorumMin` percent of members
	// and the maximum can't exceed the total number of members.
	//
	// For example:
	// p = 60 (percentage of voting period remaining)
	// m = 300 (total number of members)
	// qm = 110 (quorum modifier percent)
	// min = 50 (minimum quorum percent)
	//
	// (m * (p / 100)) * (qm / 100) = 198
	//
	// 198 is greater than (m * (min / 100)) so 198 votes is the minimum required for quorum
	//
	// If we change p to 40
	//
	// (m * (p / 100)) * (qm / 100) = 132
	// 132 is less than (m * (min / 100)) so the number of votes required for quorum is 150
	fn calculate_quorum_threshold(
		roster: &Roster<T>,
		nomination: &Nomination<T>,
	) -> Result<u32, pallet::Error<T>> {
		let percentage_elapsed = Self::calculate_percentage_elapsed(&nomination)?;
		let percentage_remaining = Percent::from_percent(100).saturating_sub(percentage_elapsed);
		let quorum_minimum_percent = Percent::from_percent(T::QuorumMin::get().min(100));
		let quorum_modifier = Percent::from_percent(T::QuorumModifier::get());

		let number_of_members = roster.members.len() as u32;
		let minimum_threshold = quorum_minimum_percent.mul_ceil(number_of_members);
		let number_of_votes_required =
			quorum_modifier.mul_ceil(percentage_remaining.mul_ceil(number_of_members));

		Ok(number_of_votes_required.max(minimum_threshold).min(number_of_members))
	}

	pub(crate) fn new(
		nominator: T::AccountId,
		roster_id: RosterId,
		nominee: T::AccountId,
	) -> DispatchResultWithPostInfo {
		// Roster must exist and be active.
		let mut nomination_roster =
			Rosters::<T>::get(&roster_id).ok_or(Error::<T>::RosterDoesNotExist)?;
		ensure!(nomination_roster.status == RosterStatus::Active, Error::<T>::RosterNotActive);

		// Nominator must be a member of the roster.
		ensure!(nomination_roster.members.contains(&nominator), Error::<T>::PermissionDenied);

		// Nominee must not be an existing member of the roster.
		ensure!(!nomination_roster.members.contains(&nominee), Error::<T>::AlreadyMember);

		// Nominee must not have already been nominated
		ensure!(
			!Nominations::<T>::contains_key(&nominee, &roster_id),
			Error::<T>::AlreadyNominated
		);

		T::Currency::reserve_named(
			&pallet::Pallet::<T>::reserved_currency_name(
				types::ReservedCurrencyReason::NewNomination(roster_id.clone(), nominee.clone()),
			),
			&nominator,
			T::NewNominationDeposit::get(),
		)
		.map_err(|_| Error::<T>::InsufficientFunds)?;

		// Create new nomination
		let nomination = Nomination::new(&roster_id, &nominee, &nominator);
		Nominations::<T>::insert(&nominee, &roster_id, &nomination);
		nomination_roster
			.nominations
			.try_push(nominee.clone())
			.map_err(|_| Error::<T>::CouldNotAddNomination)?;
		Rosters::<T>::insert(&roster_id, nomination_roster);

		pallet::Pallet::deposit_event(Event::<T>::NewNomination { nominator, nominee, roster_id });

		Ok(().into())
	}

	pub(crate) fn vote(
		voter: T::AccountId,
		roster_id: RosterId,
		nominee: T::AccountId,
		vote: NominationVoteValue,
	) -> DispatchResultWithPostInfo {
		// Check if nomination is in voting period
		let mut nomination = Nominations::<T>::get(&nominee, &roster_id)
			.ok_or(Error::<T>::NominationDoesNotExist)?;
		Self::in_voting_period(&nomination)?;

		// Voter must be a member of the roster
		let roster = Rosters::<T>::get(&roster_id).ok_or(Error::<T>::RosterDoesNotExist)?;
		ensure!(roster.members.contains(&voter), Error::<T>::PermissionDenied);

		// Voter must not have already voted.
		// If a voter wishes to change their vote they must first recant their vote.
		ensure!(!nomination.votes.iter().any(|v| v.voter == voter), Error::<T>::AlreadyVoted);

		// Add vote to nomination
		let nomination_vote = NominationVote::new(&voter, &vote);
		nomination
			.votes
			.try_push(nomination_vote)
			.map_err(|_| Error::<T>::CouldNotAddVote)?;
		Nominations::<T>::insert(&nominee, &roster_id, nomination);

		pallet::Pallet::deposit_event(Event::<T>::Voted { voter, vote, nominee, roster_id });

		Ok(().into())
	}

	pub(crate) fn recant_vote(
		voter: T::AccountId,
		roster_id: RosterId,
		nominee: T::AccountId,
	) -> DispatchResultWithPostInfo {
		// Check if nomination is in voting period
		let mut nomination = Nominations::<T>::get(&nominee, &roster_id)
			.ok_or(Error::<T>::NominationDoesNotExist)?;
		Self::in_voting_period(&nomination)?;

		// Voter must have voted in order to recant vote.
		ensure!(nomination.votes.iter().any(|v| v.voter == voter), Error::<T>::NotVoted);

		// Remove vote from nomination
		nomination.votes.retain(|v| v.voter != voter);
		Nominations::<T>::insert(&nominee, &roster_id, nomination);

		pallet::Pallet::deposit_event(Event::<T>::VoteRecanted { voter, nominee, roster_id });

		Ok(().into())
	}

	pub(crate) fn close(
		who: T::AccountId,
		roster_id: RosterId,
		nominee: T::AccountId,
	) -> DispatchResultWithPostInfo {
		// Check if nomination is in voting period
		let mut nomination = Nominations::<T>::get(&nominee, &roster_id)
			.ok_or(Error::<T>::NominationDoesNotExist)?;

		ensure!(nomination.status == NominationStatus::Pending, Error::<T>::VotingPeriodHasEnded);

		// Voting can end if:
		// - the voting period has passed ||
		// - the ayes or nays have exceeded the voting threshold ||
		// - all members have voted
		let mut roster = Rosters::<T>::get(&roster_id).ok_or(Error::<T>::RosterDoesNotExist)?;
		let ayes =
			nomination.votes.iter().filter(|v| v.vote == NominationVoteValue::Aye).count() as u32;
		let nays =
			nomination.votes.iter().filter(|v| v.vote == NominationVoteValue::Nay).count() as u32;
		let quorum_threshold = Self::calculate_quorum_threshold(&roster, &nomination)?;

		ensure!(
			nomination.nominated_on + T::NominationVotingPeriod::get()
				< <frame_system::Pallet<T>>::block_number()
				|| ayes + nays >= quorum_threshold
				|| nomination.votes.len() >= roster.members.len(),
			Error::<T>::VotingPeriodHasNotEnded
		);

		// If Ayes == Nays we default to existing roster state which is nominee is not a member
		nomination.status = match ayes > nays {
			true => NominationStatus::Approved,
			false => NominationStatus::Rejected,
		};

		Nominations::<T>::insert(&nominee, &roster_id, &nomination);

		// If nomination has been rejected refund nominator and
		// add nomination to concluded nominations
		if nomination.status == NominationStatus::Rejected {
			T::Currency::unreserve_named(
				&pallet::Pallet::<T>::reserved_currency_name(
					types::ReservedCurrencyReason::NewNomination(
						roster_id.clone(),
						nominee.clone(),
					),
				),
				&nomination.nominator,
				T::NewNominationDeposit::get(),
			);

			ConcludedNominations::<T>::try_append((&nominee, &roster_id))
				.map_err(|_| Error::<T>::CouldNotAddToConcluded)?;

			// Delete reference to nomination from roster
			roster.nominations.retain(|n| n != &nominee);
			Rosters::<T>::insert(&roster_id, roster);
		}

		pallet::Pallet::deposit_event(Event::<T>::NominationClosed {
			nominee,
			closed_by: who,
			roster_id,
			status: nomination.status,
		});

		Ok(().into())
	}

	pub(crate) fn add_member(
		member: T::AccountId,
		roster_id: RosterId,
	) -> DispatchResultWithPostInfo {
		let nomination =
			Nominations::<T>::get(&member, &roster_id).ok_or(Error::<T>::NominationDoesNotExist)?;

		ensure!(nomination.status == NominationStatus::Approved, Error::<T>::NotApproved);

		Rosters::<T>::try_mutate(&roster_id, |roster| -> DispatchResult {
			if let Some(roster) = roster {
				roster
					.members
					.try_push(member.clone())
					.map_err(|_| Error::<T>::CouldNotAddMember)?;

				// Delete reference to nomination from roster
				roster.nominations.retain(|n| n != &member);

				pallet::Pallet::deposit_event(Event::<T>::MemberAdded {
					member: member.clone(),
					roster_id: roster_id.clone(),
				});

				Ok(())
			} else {
				return Err(Error::<T>::RosterDoesNotExist.into());
			}
		})?;

		T::Currency::reserve_named(
			&pallet::Pallet::<T>::reserved_currency_name(
				types::ReservedCurrencyReason::MembershipDues(roster_id.clone()),
			),
			&member,
			T::MembershipDues::get(),
		)
		.map_err(|_| Error::<T>::InsufficientFunds)?;

		T::Currency::unreserve_named(
			&pallet::Pallet::<T>::reserved_currency_name(
				types::ReservedCurrencyReason::NewNomination(roster_id.clone(), member.clone()),
			),
			&nomination.nominator,
			T::NewNominationDeposit::get(),
		);

		ConcludedNominations::<T>::try_append((&member, &roster_id))
			.map_err(|_| Error::<T>::CouldNotAddToConcluded)?;

		Ok(().into())
	}

	pub(crate) fn remove_member(
		member: T::AccountId,
		roster_id: RosterId,
	) -> DispatchResultWithPostInfo {
		Rosters::<T>::try_mutate(&roster_id, |roster| -> DispatchResult {
			if let Some(roster) = roster {
				ensure!(roster.members.contains(&member), Error::<T>::PermissionDenied);
				// Founder cannot be removed from roster
				ensure!(roster.founder != member.clone(), Error::<T>::PermissionDenied);
				roster.members.retain(|m| m != &member);

				T::Currency::unreserve_named(
					&pallet::Pallet::<T>::reserved_currency_name(
						types::ReservedCurrencyReason::MembershipDues(roster_id.clone()),
					),
					&member,
					T::MembershipDues::get(),
				);

				pallet::Pallet::deposit_event(Event::<T>::MemberRemoved {
					member: member.clone(),
					roster_id: roster_id.clone(),
				});

				Ok(())
			} else {
				return Err(Error::<T>::RosterDoesNotExist.into());
			}
		})?;

		Ok(().into())
	}

	pub(crate) fn force_add_member(
		member: T::AccountId,
		roster_id: RosterId,
	) -> DispatchResultWithPostInfo {
		Rosters::<T>::try_mutate(&roster_id, |roster| -> DispatchResult {
			if let Some(roster) = roster {
				roster
					.members
					.try_push(member.clone())
					.map_err(|_| Error::<T>::CouldNotAddMember)?;

				pallet::Pallet::deposit_event(Event::<T>::MemberAdded {
					member: member.clone(),
					roster_id: roster_id.clone(),
				});

				Ok(())
			} else {
				return Err(Error::<T>::RosterDoesNotExist.into());
			}
		})?;

		T::Currency::reserve_named(
			&pallet::Pallet::<T>::reserved_currency_name(
				types::ReservedCurrencyReason::MembershipDues(roster_id.clone()),
			),
			&member,
			T::MembershipDues::get(),
		)
		.map_err(|_| Error::<T>::InsufficientFunds)?;

		Ok(().into())
	}

	pub(crate) fn force_add_members(
		members: Vec<T::AccountId>,
		roster_id: RosterId,
	) -> DispatchResultWithPostInfo {
		for member in members {
			Self::force_add_member(member, roster_id.clone())?;
		}

		Ok(().into())
	}
}
