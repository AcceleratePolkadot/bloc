use crate::*;
use frame_support::{
	pallet_prelude::*,
	traits::{BalanceStatus, NamedReservableCurrency},
};
use sp_runtime::Percent;
use sp_std::vec::Vec;

pub struct ExpulsionCalls<T> {
	_phantom: PhantomData<T>,
}

impl<T: Config> ExpulsionCalls<T> {
	fn active_roster(roster_id: &RosterId) -> Result<Roster<T>, pallet::Error<T>> {
		Rosters::<T>::get(&roster_id)
			.ok_or(Error::<T>::RosterDoesNotExist)
			.and_then(|roster| {
				ensure!(roster.status == RosterStatus::Active, Error::<T>::RosterNotActive);
				Ok(roster)
			})
	}

	fn in_lockout_period(account: &T::AccountId, roster_id: &RosterId) -> bool {
		// If account has seconded or motioned an expulsion proposal for this roster
		// which was dismissed with prejudice, and it was < ExpulsionProposalLockoutPeriod blocks
		// ago they are in the lockout period
		!ExpulsionProposals::<T>::iter_prefix_values((&roster_id,)).any(|proposal| {
			if let Some(decided_on) = proposal.decided_on {
				return (proposal.seconds.contains(account) || proposal.motioner == *account)
					&& proposal.status == ExpulsionProposalStatus::DismissedWithPrejudice
					&& decided_on + T::ExpulsionProposalLockoutPeriod::get()
						>= <frame_system::Pallet<T>>::block_number();
			}
			false
		})
	}

	fn can_call_expulsion_vote(
		motioner: &T::AccountId,
		subject: &T::AccountId,
		roster: &Roster<T>,
	) -> Result<bool, pallet::Error<T>> {
		// Motioner must be a member of the roster.
		ensure!(roster.members.contains(&motioner), Error::<T>::PermissionDenied);
		// Subject must be a member of the roster
		ensure!(roster.members.contains(&subject), Error::<T>::PermissionDenied);
		// Motioner must not be in lockout period
		ensure!(Self::in_lockout_period(&motioner, &roster.id), Error::<T>::PermissionDenied);

		// Motioner can only have 1 open expulsion proposal at a time
		// Motioner can not open a new proposal against subject if a previous proposal was dismissed
		// with prejudice
		ensure!(
			!ExpulsionProposals::<T>::iter_prefix((&roster.id, &motioner)).any(|(_, proposal)| {
				(proposal.status == ExpulsionProposalStatus::Proposed
					|| proposal.status == ExpulsionProposalStatus::Seconded
					|| proposal.status == ExpulsionProposalStatus::Voting)
					|| (proposal.subject == *subject
						&& proposal.status == ExpulsionProposalStatus::DismissedWithPrejudice)
			}),
			Error::<T>::PermissionDenied
		);

		// Subject can only be the target of 1 open expulsion proposal at a time per roster
		for open_proposal in roster.expulsion_proposals.iter() {
			ensure!(subject != &open_proposal.1, Error::<T>::PermissionDenied);
		}

		Ok(true)
	}

	fn can_second_expulsion_proposal(
		seconder: &T::AccountId,
		roster: &Roster<T>,
		expulsion_proposal: &ExpulsionProposal<T>,
	) -> Result<bool, pallet::Error<T>> {
		// Seconder must be a member of the roster.
		ensure!(roster.members.contains(&seconder), Error::<T>::PermissionDenied);
		// Expulsion proposal must be in Proposed or Seconded state
		ensure!(
			expulsion_proposal.status == ExpulsionProposalStatus::Proposed
				|| expulsion_proposal.status == ExpulsionProposalStatus::Seconded,
			Error::<T>::PermissionDenied
		);
		// Seconder must not be in lockout period
		ensure!(Self::in_lockout_period(&seconder, &roster.id), Error::<T>::PermissionDenied);

		Ok(true)
	}

	fn can_dismiss_expulsion_proposal_with_prejudice(
		roster: &Roster<T>,
		expulsion_proposal: &ExpulsionProposal<T>,
	) -> bool {
		// If the proposal has not reached the second threshold, and the
		// `ExpulsionProposalAwaitingSecondPeriod` has passed it can be dismissed with prejudice
		let seconds_count = expulsion_proposal.seconds.len() as u32;
		if seconds_count < T::ExpulsionProposalSecondThreshold::get()
			&& expulsion_proposal.proposed_on + T::ExpulsionProposalAwaitingSecondPeriod::get()
				< <frame_system::Pallet<T>>::block_number()
		{
			return true;
		} else {
			// If the voting period has ended and the Nay votes have exceeded the
			// `ExpulsionProposalSuperMajority`, the proposal can be dismissed with prejudice
			let voting_opened_on = expulsion_proposal
				.voting_opened_on
				.unwrap_or(<frame_system::Pallet<T>>::block_number());
			let nays = expulsion_proposal
				.votes
				.iter()
				.filter(|v| v.vote == ExpulsionProposalVoteValue::Nay)
				.count() as u32;
			let nays_threshold = Percent::from_percent(
				T::ExpulsionProposalSuperMajority::get().try_into().unwrap_or(100),
			) * roster.members.len() as u32;
			voting_opened_on + T::ExpulsionProposalVotingPeriod::get()
				< <frame_system::Pallet<T>>::block_number()
				&& nays >= nays_threshold
		}
	}

	fn remove_proposal_from_roster(
		roster: &Roster<T>,
		motioner: &T::AccountId,
		subject: &T::AccountId,
	) {
		// Remove proposal from roster
		let mut updated_roster = roster.clone();
		updated_roster
			.expulsion_proposals
			.retain(|(m, s)| m != motioner || s != subject);
		Rosters::<T>::insert(&roster.id, updated_roster);
	}

	pub(crate) fn new(
		motioner: T::AccountId,
		subject: T::AccountId,
		roster_id: RosterId,
		reason: Vec<u8>,
	) -> DispatchResultWithPostInfo {
		let mut roster = Self::active_roster(&roster_id)?;
		// Reason must be >= `ExpulsionReasonMinLength` and < `ExpulsionReasonMaxLength`
		let bounded_reason: BoundedVec<_, _> =
			reason.try_into().map_err(|_| Error::<T>::InvalidExpulsionReason)?;
		ensure!(
			bounded_reason.len() as u32 >= <T>::ExpulsionReasonMinLength::get(),
			Error::<T>::InvalidExpulsionReason
		);

		Self::can_call_expulsion_vote(&motioner, &subject, &roster)?;

		// Create new expulsion proposal
		let expulsion_proposal: ExpulsionProposal<_> =
			ExpulsionProposal::new(&motioner, &subject, &roster_id, &bounded_reason);
		ExpulsionProposals::<T>::insert((&roster_id, &motioner, &subject), &expulsion_proposal);
		roster
			.expulsion_proposals
			.try_push((motioner.clone(), subject.clone()))
			.map_err(|_| Error::<T>::CouldNotAddExpulsionProposal)?;
		Rosters::<T>::insert(&roster_id, roster);

		pallet::Pallet::deposit_event(Event::<T>::NewExpulsionProposal {
			motioner: motioner.clone(),
			subject: subject.clone(),
			roster_id: roster_id.clone(),
			reason: bounded_reason,
		});

		T::Currency::reserve_named(
			&pallet::Pallet::<T>::reserved_currency_name(
				types::ReservedCurrencyReason::NewExpulsionProposal(roster_id, subject),
			),
			&motioner,
			T::NewExpulsionProposalDeposit::get(),
		)
		.map_err(|_| Error::<T>::InsufficientFunds)?;

		Ok(().into())
	}
	pub(crate) fn second(
		seconder: T::AccountId,
		motioner: T::AccountId,
		subject: T::AccountId,
		roster_id: RosterId,
	) -> DispatchResultWithPostInfo {
		/* We do not check if the proposal is still within the
		 * `ExpulsionProposalAwaitingSecondPeriod`  If a proposal has not been closed, seconds
		 * can still be added even after the period has expired.  It is in the subject's best
		 * interest to close the proposal as soon as `ExpulsionProposalAwaitingSecondPeriod`
		 * passes
		 */
		let roster = Self::active_roster(&roster_id)?;
		let mut expulsion_proposal =
			ExpulsionProposals::<T>::get((&roster.id, &motioner, &subject))
				.ok_or(Error::<T>::ExpulsionProposalDoesNotExist)?;
		Self::can_second_expulsion_proposal(&seconder, &roster, &expulsion_proposal)?;

		expulsion_proposal
			.seconds
			.try_push(seconder.clone())
			.map_err(|_| Error::<T>::CouldNotAddSeconder)?;
		expulsion_proposal.status = ExpulsionProposalStatus::Seconded;
		ExpulsionProposals::<T>::insert((&roster_id, &motioner, &subject), &expulsion_proposal);

		pallet::Pallet::deposit_event(Event::<T>::SeconderAddedToExpulsionProposal {
			seconder,
			motioner,
			subject,
			roster_id,
			seconds_count: expulsion_proposal.seconds.len() as u32,
		});

		Ok(().into())
	}
	pub(crate) fn open(
		motioner: T::AccountId,
		subject: T::AccountId,
		roster_id: RosterId,
	) -> DispatchResultWithPostInfo {
		let roster = Self::active_roster(&roster_id)?;
		let mut expulsion_proposal =
			ExpulsionProposals::<T>::get((&roster.id, &motioner, &subject))
				.ok_or(Error::<T>::ExpulsionProposalDoesNotExist)?;
		ensure!(
			expulsion_proposal.seconds.len() as u32 >= T::ExpulsionProposalSecondThreshold::get(),
			Error::<T>::InsufficientSeconds
		);

		// Expulsion proposal must be in seconded state
		ensure!(
			expulsion_proposal.status == ExpulsionProposalStatus::Seconded,
			Error::<T>::PermissionDenied
		);

		// Open the proposal for voting
		expulsion_proposal.status = ExpulsionProposalStatus::Voting;
		expulsion_proposal.voting_opened_on = Some(<frame_system::Pallet<T>>::block_number());
		ExpulsionProposals::<T>::insert((&roster_id, &motioner, &subject), &expulsion_proposal);

		pallet::Pallet::deposit_event(Event::<T>::ExpulsionVoteOpened {
			motioner,
			subject,
			roster_id,
		});

		Ok(().into())
	}
	pub(crate) fn vote(
		voter: T::AccountId,
		motioner: T::AccountId,
		subject: T::AccountId,
		roster_id: RosterId,
		vote: ExpulsionProposalVoteValue,
	) -> DispatchResultWithPostInfo {
		let roster = Self::active_roster(&roster_id)?;
		let mut expulsion_proposal =
			ExpulsionProposals::<T>::get((&roster.id, &motioner, &subject))
				.ok_or(Error::<T>::ExpulsionProposalDoesNotExist)?;

		// Voter must be a member of the roster
		ensure!(roster.members.contains(&voter), Error::<T>::PermissionDenied);

		// Expulsion proposal must be in Voting state
		ensure!(
			expulsion_proposal.status == ExpulsionProposalStatus::Voting,
			Error::<T>::VotingPeriodHasNotStarted
		);

		// Voting period must not have ended
		let voting_opened_on = expulsion_proposal
			.voting_opened_on
			.ok_or(Error::<T>::VotingPeriodHasNotStarted)?;
		ensure!(
			voting_opened_on + T::ExpulsionProposalVotingPeriod::get()
				>= <frame_system::Pallet<T>>::block_number(),
			Error::<T>::VotingPeriodHasEnded
		);

		// Voter must not have already voted
		ensure!(
			!expulsion_proposal.votes.iter().any(|v| v.voter == voter),
			Error::<T>::AlreadyVoted
		);

		// Add vote to proposal
		let voting_record = ExpulsionProposalVote::new(&voter, &vote);
		expulsion_proposal
			.votes
			.try_push(voting_record)
			.map_err(|_| Error::<T>::CouldNotAddVote)?;
		ExpulsionProposals::<T>::insert((&roster_id, &motioner, &subject), &expulsion_proposal);

		pallet::Pallet::deposit_event(Event::<T>::ExpulsionVoteSubmitted {
			voter,
			motioner,
			subject,
			roster_id,
			vote,
		});

		Ok(().into())
	}
	pub(crate) fn recant_vote(
		voter: T::AccountId,
		motioner: T::AccountId,
		subject: T::AccountId,
		roster_id: RosterId,
	) -> DispatchResultWithPostInfo {
		let roster = Self::active_roster(&roster_id)?;

		// Attempt to remove vote if it exists
		ExpulsionProposals::<T>::try_mutate(
			(&roster.id, &motioner, &subject),
			|ep| -> DispatchResult {
				if let Some(ep) = ep {
					let previous_vote_count = ep.votes.len();
					ep.votes.retain(|v| v.voter != voter);
					ensure!(ep.votes.len() < previous_vote_count, Error::<T>::NotVoted);
					Ok(())
				} else {
					Err(Error::<T>::ExpulsionProposalDoesNotExist.into())
				}
			},
		)?;

		pallet::Pallet::deposit_event(Event::<T>::ExpulsionVoteRecanted {
			voter,
			motioner,
			subject,
			roster_id,
		});

		Ok(().into())
	}
	pub(crate) fn close(
		closer: T::AccountId,
		motioner: T::AccountId,
		subject: T::AccountId,
		roster_id: RosterId,
	) -> DispatchResultWithPostInfo {
		let mut roster = Self::active_roster(&roster_id)?;
		let mut expulsion_proposal =
			ExpulsionProposals::<T>::get((&roster.id, &motioner, &subject))
				.ok_or(Error::<T>::ExpulsionProposalDoesNotExist)?;

		// Closer must be a member of the roster.
		ensure!(roster.members.contains(&closer), Error::<T>::PermissionDenied);

		if Self::can_dismiss_expulsion_proposal_with_prejudice(&roster, &expulsion_proposal) {
			// Dismiss proposal with prejudice
			expulsion_proposal.status = ExpulsionProposalStatus::DismissedWithPrejudice;
			expulsion_proposal.decided_on = Some(<frame_system::Pallet<T>>::block_number());
			ExpulsionProposals::<T>::insert((&roster_id, &motioner, &subject), &expulsion_proposal);
			Self::remove_proposal_from_roster(&roster, &motioner, &subject);
			pallet::Pallet::deposit_event(Event::<T>::ExpulsionProposalDismissedWithPrejudice {
				closer,
				motioner: motioner.clone(),
				subject: subject.clone(),
				roster_id: roster_id.clone(),
			});

			// Slash the motioner's deposit
			// The subject receives `ExpulsionProposalReparations` percent of the deposit
			// The rest goes into the pot
			let pot = pallet::Pallet::<T>::account_id().ok_or(Error::<T>::TreasuryDoesNotExist)?;
			let balance_status = BalanceStatus::Free;
			let reparations = Percent::from_percent(
				T::ExpulsionProposalReparations::get().try_into().unwrap_or(50),
			) * T::NewExpulsionProposalDeposit::get();

			T::Currency::repatriate_reserved_named(
				&pallet::Pallet::<T>::reserved_currency_name(
					types::ReservedCurrencyReason::NewExpulsionProposal(
						roster_id.clone(),
						subject.clone(),
					),
				),
				&motioner,
				&subject,
				reparations,
				balance_status,
			)?;

			T::Currency::repatriate_all_reserved_named(
				&pallet::Pallet::<T>::reserved_currency_name(
					types::ReservedCurrencyReason::NewExpulsionProposal(
						roster_id.clone(),
						subject.clone(),
					),
				),
				&motioner,
				&pot,
				balance_status,
			)?;
		} else {
			// Expulsion proposal must be in Voting state
			ensure!(
				expulsion_proposal.status == ExpulsionProposalStatus::Voting,
				Error::<T>::PermissionDenied
			);

			let voting_opened_on = expulsion_proposal
				.voting_opened_on
				.unwrap_or(<frame_system::Pallet<T>>::block_number());
			ensure!(
				voting_opened_on + T::ExpulsionProposalVotingPeriod::get()
					< <frame_system::Pallet<T>>::block_number(),
				Error::<T>::VotingPeriodHasNotEnded
			);

			let total_votes = expulsion_proposal.votes.len() as u32;
			let votes_needed_for_quorum =
				Percent::from_percent(T::ExpulsionProposalQuorum::get().try_into().unwrap_or(100))
					* roster.members.len() as u32;

			let ayes = expulsion_proposal
				.votes
				.iter()
				.filter(|v| v.vote == ExpulsionProposalVoteValue::Aye)
				.count() as u32;
			let nays = expulsion_proposal
				.votes
				.iter()
				.filter(|v| v.vote == ExpulsionProposalVoteValue::Nay)
				.count() as u32;

			// If the nays >= ayes or total votes < votes_needed_for_quorum, dismiss the proposal
			if nays >= ayes || total_votes < votes_needed_for_quorum {
				// Dismiss proposal
				expulsion_proposal.status = ExpulsionProposalStatus::Dismissed;
				expulsion_proposal.decided_on = Some(<frame_system::Pallet<T>>::block_number());
				ExpulsionProposals::<T>::insert(
					(&roster_id, &motioner, &subject),
					&expulsion_proposal,
				);
				Self::remove_proposal_from_roster(&roster, &motioner, &subject);
				pallet::Pallet::deposit_event(Event::<T>::ExpulsionProposalDismissed {
					closer,
					motioner,
					subject,
					roster_id,
				});
			} else {
				// Proposal has passed
				expulsion_proposal.status = ExpulsionProposalStatus::Passed;
				expulsion_proposal.decided_on = Some(<frame_system::Pallet<T>>::block_number());
				ExpulsionProposals::<T>::insert(
					(&roster_id, &motioner, &subject),
					&expulsion_proposal,
				);
				Self::remove_proposal_from_roster(&roster, &motioner, &subject);
				pallet::Pallet::deposit_event(Event::<T>::ExpulsionProposalPassed {
					closer,
					motioner,
					subject: subject.clone(),
					roster_id: roster_id.clone(),
				});

				// Proposal passed, kick subject from roster
				roster.members.retain(|m| m != &subject);
				Rosters::<T>::insert(&roster_id, roster);
				pallet::Pallet::deposit_event(Event::<T>::MemberRemoved {
					member: subject.clone(),
					roster_id: roster_id.clone(),
				});

				// Slash their membership dues
				let pot =
					pallet::Pallet::<T>::account_id().ok_or(Error::<T>::TreasuryDoesNotExist)?;
				let balance_status = BalanceStatus::Free;
				T::Currency::repatriate_all_reserved_named(
					&pallet::Pallet::<T>::reserved_currency_name(
						types::ReservedCurrencyReason::MembershipDues(roster_id),
					),
					&subject,
					&pot,
					balance_status,
				)?;
			}
		}

		Ok(().into())
	}
}
