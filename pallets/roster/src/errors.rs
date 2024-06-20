use frame_support::pallet_macros::*;

#[pallet_section]
mod errors {
    #[pallet::error]
    pub enum Error<T> {
        /// The roster title is invalid.
        InvalidRosterTitle,
        /// A roster with the same title already exists for this account.
        RosterExists
    }
}