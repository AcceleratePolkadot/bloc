use frame_support::pallet_macros::*;

#[pallet_section]
mod errors {
    #[pallet::error]
    pub enum Error<T> {
        /// The roster title is invalid.
        InvalidRosterTitle,
        /// A roster with the same title already exists for this account.
        RosterExists,
        /// Roster does not exist.
        RosterDoesNotExist,
        /// Roster is not active.
        RosterNotActive,
        /// THe sender does not have required permissions.
        PermissionDenied,
        /// The nominee is already a member of the roster.
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
        /// Nomination voting period has ended.
        NominationVotingPeriodEnded,
        /// Nomination voting period cannot be closed at this time.
        NominationVotingPeriodHasNotEnded,
        /// Account has already voted in this nomination.
        AlreadyVoted,
        /// Account has not voted in this nomination.
        NotVoted,
        /// Could not add vote to nomination votes.
        CouldNotAddVote,
        /// Could not add member to roster members.
        CouldNotAddMember,
        /// Error converting between types
        ConversionError,
        /// Could not add item to the list of concluded/closed items
        CouldNotAddToConcluded,
    }
}