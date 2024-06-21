use frame_support::pallet_macros::*;

#[pallet_section]
mod events {
	#[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New roster created [created by, roster title]
        NewRoster(T::AccountId, RosterTitle<T>),
        /// New nomination [nominator, nominee, roster id, roster title]
        NewNomination(T::AccountId, T::AccountId, RosterId, RosterTitle<T>),
    }
}
