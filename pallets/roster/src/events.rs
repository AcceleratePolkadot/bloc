use frame_support::pallet_macros::*;

#[pallet_section]
mod events {
	use crate::RosterStatus;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New roster created [created by, roster title, roster id]
		NewRoster { founder: T::AccountId, title: RosterTitle<T>, roster_id: RosterId },
		/// Roster status has been changed [changed by, roster id, new status]
		RosterStatusChanged {
			changed_by: T::AccountId,
			roster_id: RosterId,
			new_status: RosterStatus,
		},
		/// Roster has been removed [removed by, roster id]
		RosterRemoved { removed_by: T::AccountId, roster_id: RosterId },
		/// New nomination [nominator, nominee, roster id]
		NewNomination { nominator: T::AccountId, nominee: T::AccountId, roster_id: RosterId },
		/// Nomination has closed [nominator, nominee, closed by, roster id, nomination status]
		NominationClosed {
			nominee: T::AccountId,
			closed_by: T::AccountId,
			roster_id: RosterId,
			status: NominationStatus,
		},
		/// Vote added to nomination [voter, vote value, nominee, roster id]
		Voted {
			voter: T::AccountId,
			vote: NominationVoteValue,
			nominee: T::AccountId,
			roster_id: RosterId,
		},
		/// Vote recanted [voter, nominee, roster id]
		VoteRecanted { voter: T::AccountId, nominee: T::AccountId, roster_id: RosterId },
		/// New member added to roster [member, roster id]
		MemberAdded { member: T::AccountId, roster_id: RosterId },
		/// Member removed from roster [member, roster id]
		MemberRemoved { member: T::AccountId, roster_id: RosterId },
		/// New expulsion proposal [motioner, subject, roster id, reason]
		NewExpulsionProposal {
			motioner: T::AccountId,
			subject: T::AccountId,
			roster_id: RosterId,
			reason: ExpulsionReason<T>,
		},
		/// Expulsion Proposal has been seconded
		/// [seconder, motioner, subject, roster id, count of seconders]
		SeconderAddedToExpulsionProposal {
			seconder: T::AccountId,
			motioner: T::AccountId,
			subject: T::AccountId,
			roster_id: RosterId,
			seconds_count: u32,
		},
		/// Expulsion proposal opened for voting [motioner, subject, roster id]
		ExpulsionVoteOpened { motioner: T::AccountId, subject: T::AccountId, roster_id: RosterId },
		/// Vote added to expulsion proposal [voter, motioner, subject, roster id, vote value]
		ExpulsionVoteSubmitted {
			voter: T::AccountId,
			motioner: T::AccountId,
			subject: T::AccountId,
			roster_id: RosterId,
			vote: ExpulsionProposalVoteValue,
		},
		/// Vote added to expulsion proposal [voter, motioner, subject, roster id]
		ExpulsionVoteRecanted {
			voter: T::AccountId,
			motioner: T::AccountId,
			subject: T::AccountId,
			roster_id: RosterId,
		},
		/// Expulsion proposal has been dismissed with prejudice
		/// [closer, motioner, subject, roster id]
		ExpulsionProposalDismissedWithPrejudice {
			closer: T::AccountId,
			motioner: T::AccountId,
			subject: T::AccountId,
			roster_id: RosterId,
		},
		/// Expulsion proposal has been dismissed [closer, motioner, subject, roster id]
		ExpulsionProposalDismissed {
			closer: T::AccountId,
			motioner: T::AccountId,
			subject: T::AccountId,
			roster_id: RosterId,
		},
		/// Expulsion proposal has passed [closer, motioner, subject, roster id]
		ExpulsionProposalPassed {
			closer: T::AccountId,
			motioner: T::AccountId,
			subject: T::AccountId,
			roster_id: RosterId,
		},
	}
}
