use frame_support::pallet_macros::*;

pub mod nominations;

#[pallet_section]
mod calls {
    use sp_std::vec::Vec;
    use crate::calls::nominations::NominationCalls;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn new_roster(origin: OriginFor<T>, title: Vec<u8>) -> DispatchResultWithPostInfo {
            let founder = ensure_signed(origin)?;
            let bounded_title: BoundedVec<_, _> = title.try_into().map_err(|_| Error::<T>::InvalidRosterTitle)?;
            let roster_id = RosterId::from_tuple::<T>((&founder, &bounded_title));

            ensure!(!Rosters::<T>::contains_key(&roster_id), Error::<T>::RosterExists);

            let mut roster = Roster::new(&founder, &bounded_title);
            // Add founder as first member
            roster.members.try_push(founder.clone()).map_err(|_| Error::<T>::CouldNotAddMember)?;
            Rosters::<T>::insert(&roster_id, roster);

            Self::deposit_event(Event::NewRoster(founder,  bounded_title, roster_id));

            Ok(().into())
        }

        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn nomination_new(origin: OriginFor<T>, roster_id: RosterId, nominee: T::AccountId) -> DispatchResultWithPostInfo {
            let nominator = ensure_signed(origin)?;
            NominationCalls::<T>::new(nominator, roster_id, nominee)
        }

        #[pallet::call_index(20)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn nomination_vote(origin: OriginFor<T>, roster_id: RosterId, nominee: T::AccountId, vote: NominationVoteValue) -> DispatchResultWithPostInfo {
            let voter = ensure_signed(origin)?;
            NominationCalls::<T>::vote(voter, roster_id, nominee, vote)
        }

        #[pallet::call_index(30)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn nomination_vote_recant(origin: OriginFor<T>, roster_id: RosterId, nominee: T::AccountId) -> DispatchResultWithPostInfo {
            let voter = ensure_signed(origin)?;
            NominationCalls::<T>::recant_vote(voter, roster_id, nominee)
        }

        #[pallet::call_index(40)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn nomination_close(origin: OriginFor<T>, roster_id: RosterId, nominee: T::AccountId) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            NominationCalls::<T>::close(who, roster_id, nominee) 
        }
    }
}