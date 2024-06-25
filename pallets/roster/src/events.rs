use frame_support::pallet_macros::*;

#[pallet_section]
mod events {
	#[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New roster created [created by, roster title, roster id]
        NewRoster(T::AccountId, RosterTitle<T>, RosterId),
        /// New nomination [nominator, nominee, roster id]
        NewNomination(T::AccountId, T::AccountId, RosterId),
        /// Nomination period has ended [nominator, nominee, closed by, nomination status]
        NominationClosed(T::AccountId, RosterId, T::AccountId, NominationStatus),
        /// Vote added to nomination [voter, vote value, nominee, roster id]
        Voted(T::AccountId, NominationVoteValue, T::AccountId, RosterId),
        /// Vote recanted [voter, nominee, roster id]
        VoteRecanted(T::AccountId, T::AccountId, RosterId),
    }
}
