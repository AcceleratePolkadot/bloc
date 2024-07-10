use frame_support::pallet_macros::*;

#[pallet_section]
mod errors {
	#[pallet::error]
	pub enum Error<T> {
		/// Account does not have enough funds available to perform the operation.
		InsufficientFunds,
		/// Voting period has not started yet
		VotingPeriodHasNotStarted,
		/// Voting period has ended.
		VotingPeriodEnded,
		/// Voting period cannot be closed at this time.
		VotingPeriodHasNotEnded,
		/// Voting period has ended
		VotingPeriodHasEnded,
		/// Account has already voted
		AlreadyVoted,
		/// Account has not voted
		NotVoted,
		/// Could not add vote
		CouldNotAddVote,
		/// The roster title is invalid.
		InvalidRosterTitle,
		/// A roster with the same title already exists for this account.
		RosterExists,
		/// Roster does not exist.
		RosterDoesNotExist,
		/// Roster is active.
		RosterActive,
		/// Roster is not active.
		RosterNotActive,
		/// THe sender does not have required permissions.
		PermissionDenied,
		/// The account is already a member of the roster.
		AlreadyMember,
		/// Account is already nominated for a roster.
		AlreadyNominated,
		/// Account is not nominated for a roster.
		NotNominated,
		/// Nomination for that account and roster does not exist.
		NominationDoesNotExist,
		/// Nomination has already been approved.
		NominationAlreadyApproved,
		/// Nomination has already been rejected.
		NominationAlreadyRejected,
		/// Could not add member to roster members.
		CouldNotAddMember,
		/// Could not add nomination to roster nominations list.
		CouldNotAddNomination,
		/// Error converting between types
		ConversionError,
		/// Could not add item to the list of concluded/closed items
		CouldNotAddToConcluded,
		/// The expulsion reason is invalid (probably a length issue)
		InvalidExpulsionReason,
		/// Expulsion proposal does not exist for that key
		ExpulsionProposalDoesNotExist,
		/// Could not add expulsion proposal to roster expulsion proposals list
		CouldNotAddExpulsionProposal,
		/// Could not add seconder to expulsion proposal
		CouldNotAddSeconder,
		/// Not enough seconds supporting the expulsion proposal
		InsufficientSeconds,
		/// Could not remove all expulsion proposals from storage
		CouldNotRemoveAllExpulsionProposals,
	}
}
