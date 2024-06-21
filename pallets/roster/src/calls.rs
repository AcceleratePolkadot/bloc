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
            let roster_id = RosterId::from_tuple::<T>((&founder, &title));

            ensure!(!Rosters::<T>::contains_key(&roster_id), Error::<T>::RosterExists);

            let mut roster = Roster::new(founder.clone(), title.clone());
            // Add founder as first member
            roster.members.try_push(founder.clone()).map_err(|_| Error::<T>::CouldNotAddMember)?;
            Rosters::<T>::insert(&roster_id, roster);

            Self::deposit_event(Event::NewRoster(founder,  title));

            Ok(().into())
        }

        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn nominate(origin: OriginFor<T>, roster: (T::AccountId, RosterTitle<T>), nominee: T::AccountId) -> DispatchResultWithPostInfo {
            let nominator = ensure_signed(origin)?;
            let (founder, title) = roster;
            let roster_id = RosterId::from_tuple::<T>((&founder, &title));

            // Roster must exist and be active.
            let nomination_roster = Rosters::<T>::get(&roster_id).ok_or(Error::<T>::RosterDoesNotExist)?;
            ensure!(nomination_roster.status == RosterStatus::Active, Error::<T>::RosterNotActive);

            // Nominator must be a member of the roster.
            ensure!(nomination_roster.members.contains(&nominator), Error::<T>::PermissionDenied);

            // Nominee must not be an existing member of the roster.
            ensure!(!nomination_roster.members.contains(&nominee), Error::<T>::AlreadyMember);

            // Nominee must not have already been nominated
            ensure!(!Nominations::<T>::contains_key(&nominee, &roster_id), Error::<T>::AlreadyNominated);

            // Create new nomination
            let nomination = Nomination::new((founder, title.clone()), nominee.clone(), nominator.clone());
            Nominations::<T>::insert(&nominee, &roster_id, nomination);
            Self::deposit_event(Event::NewNomination(nominator, nominee, roster_id, title));

            Ok(().into())
        }
    }
}