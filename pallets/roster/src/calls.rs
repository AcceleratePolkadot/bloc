use frame_support::pallet_macros::*;

#[pallet_section]
mod calls {
    use sp_std::vec::Vec;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn new_roster(origin: OriginFor<T>, unbounded_title: Vec<u8>) -> DispatchResultWithPostInfo {
            let founder = ensure_signed(origin)?;
            let title: BoundedVec<_, _> = unbounded_title.try_into().map_err(|_| Error::<T>::InvalidRosterTitle)?;

            ensure!(!Rosters::<T>::contains_key(&founder, &title), Error::<T>::RosterExists);

            let roster = Roster::new(founder.clone(), title.clone());
            Rosters::<T>::insert(&founder, &title, roster);

            Self::deposit_event(Event::NewRoster(founder,  title));

            Ok(().into())
        }
    }
}