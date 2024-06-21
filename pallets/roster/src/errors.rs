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
        /// The nominee is already nominated for a roster.
        AlreadyNominated,
        /// Could not add member to roster members.
        CouldNotAddMember,
    }
}