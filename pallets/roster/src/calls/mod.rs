use frame_support::pallet_macros::*;

pub mod nominations;

#[pallet_section]
mod calls {
    use sp_std::vec::Vec;
    use crate::{calls::nominations::NominationCalls, RosterId};

    impl<T: Config> Pallet<T> {
        fn update_roster_status(founder: &T::AccountId, roster_id: &RosterId, new_status: RosterStatus) -> DispatchResult {
            let roster = Rosters::<T>::get(&roster_id).ok_or(Error::<T>::RosterDoesNotExist)?;
            ensure!(roster.founder == *founder, Error::<T>::PermissionDenied);

            ensure!(roster.status != new_status, match new_status {
                RosterStatus::Active => Error::<T>::RosterActive,
                RosterStatus::Inactive =>  Error::<T>::RosterNotActive,
            });

            Rosters::<T>::try_mutate(&roster_id, |mut_roster| -> DispatchResult {
                mut_roster.as_mut().ok_or(Error::<T>::RosterDoesNotExist)?.status = new_status;
                Ok(())
            })?;

            Ok(())
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new roster
        ///
        /// New rosters are created as active
		///
		/// The dispatch origin of this call must be _Signed_
        /// The origin will be made the founder of the roster and added as its first member
		///
		/// - `title`: A vector of bytes representing the string title of the roster (must be smaller than `TitleMaxLength`)
		///
		/// Emits `NewRoster`
        #[pallet::call_index(100)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn roster_create(origin: OriginFor<T>, title: Vec<u8>) -> DispatchResultWithPostInfo {
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

        /// Activate a Roster
		///
		/// The dispatch origin of this call must be _Signed_
        /// The origin must be the roster founder
        ///
        /// - `roster_id`: The v8 UUID for the roster to activate
		///
		/// Emits `RosterStatusChanged`
        #[pallet::call_index(110)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn roster_activate(origin: OriginFor<T>,  roster_id: RosterId) -> DispatchResultWithPostInfo {
            let founder = ensure_signed(origin)?;

            Self::update_roster_status(&founder, &roster_id, RosterStatus::Active)?;
            Self::deposit_event(Event::RosterStatusChanged(founder, roster_id, RosterStatus::Active));

            Ok(().into())
        }

        /// Deactivate a Roster
		///
		/// The dispatch origin of this call must be _Signed_
        /// The origin must be the roster founder
        ///
        /// - `roster_id`: The v8 UUID for the roster to deactivate
		///
		/// Emits `RosterStatusChanged`
        /// Emits `NominationClosed`
        #[pallet::call_index(111)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn roster_deactivate(origin: OriginFor<T>,  roster_id: RosterId) -> DispatchResultWithPostInfo {
            let founder = ensure_signed(origin)?;

            Self::update_roster_status(&founder, &roster_id, RosterStatus::Inactive)?;
            Self::deposit_event(Event::RosterStatusChanged(founder.clone(), roster_id.clone(), RosterStatus::Inactive));

            // When a roster is deactivated all active nominations are rejected
            let roster = Rosters::<T>::get(&roster_id).ok_or(Error::<T>::RosterDoesNotExist)?;
            for nominee in roster.nominations.iter() {
                Nominations::<T>::try_mutate(&nominee, &roster_id, |nomination| -> DispatchResult {
                    nomination.as_mut().ok_or(Error::<T>::NominationDoesNotExist)?.status = NominationStatus::Rejected;
                    Ok(())
                })?;
                ConcludedNominations::<T>::try_append((&nominee, &roster_id)).map_err(|_| Error::<T>::CouldNotAddToConcluded)?;
                Self::deposit_event(Event::NominationClosed(nominee.clone(), roster_id.clone(), founder.clone(),  NominationStatus::Rejected));
            }

            Ok(().into())
        }

        /// Remove a Roster
		///
		/// The dispatch origin of this call must be _Signed_
        /// The origin must be the roster founder
        ///
        /// Only deactivated rosters can be removed
        ///
        /// - `roster_id`: The v8 UUID for the roster to remove
		///
		/// Emits `RosterRemoved`
        #[pallet::call_index(120)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn roster_remove(origin: OriginFor<T>,  roster_id: RosterId) -> DispatchResultWithPostInfo {
            let founder = ensure_signed(origin)?;

            // We use `take()` here to also remove the item from storage
            // If an error gets raised later all state changes within the extrinsic will be discarded
            let roster = Rosters::<T>::take(&roster_id).ok_or(Error::<T>::RosterDoesNotExist)?;

            ensure!(roster.founder == founder, Error::<T>::PermissionDenied);
            ensure!(roster.status == RosterStatus::Inactive, Error::<T>::RosterActive);

            Self::deposit_event(Event::RosterRemoved(founder, roster_id));

            Ok(().into())
        }

        /// Nominate an account to join a roster
		///
		/// The dispatch origin of this call must be _Signed_
        /// The origin will be made the nominator
		///
		/// - `roster_id`: The v8 UUID for the roster the account is being nominated to join
        /// - `nominee`: AccountId of the account being nominated
		///
		/// Emits `NewNomination`
        #[pallet::call_index(200)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn nomination_new(origin: OriginFor<T>, roster_id: RosterId, nominee: T::AccountId) -> DispatchResultWithPostInfo {
            let nominator = ensure_signed(origin)?;
            NominationCalls::<T>::new(nominator, roster_id, nominee)
        }

        /// Vote on a nomination
		///
		/// The dispatch origin of this call must be _Signed_
        /// The origin must already be a member of the roster the nomination is for
		///
		/// - `roster_id`: The v8 UUID for the roster
        /// - `nominee`: AccountId of the account being nominated
        /// - `vote`: The `NominationVoteValue` (Aye or Nay)
		///
		/// Emits `Voted`
        #[pallet::call_index(210)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn nomination_vote(origin: OriginFor<T>, roster_id: RosterId, nominee: T::AccountId, vote: NominationVoteValue) -> DispatchResultWithPostInfo {
            let voter = ensure_signed(origin)?;
            NominationCalls::<T>::vote(voter, roster_id, nominee, vote)
        }

        /// Recant (withdraw) a vote on a nomination
		///
		/// The dispatch origin of this call must be _Signed_
        /// The origin can only recant votes they have submitted
		///
		/// - `roster_id`: The v8 UUID for the roster
        /// - `nominee`: AccountId of the account being nominated
		///
		/// Emits `VoteRecanted`
        #[pallet::call_index(220)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn nomination_vote_recant(origin: OriginFor<T>, roster_id: RosterId, nominee: T::AccountId) -> DispatchResultWithPostInfo {
            let voter = ensure_signed(origin)?;
            NominationCalls::<T>::recant_vote(voter, roster_id, nominee)
        }

        /// Close a nomination and the voting period has ended or the voting threshold has been reached
		///
		/// The dispatch origin of this call must be _Signed_
        /// Anyone can close a nomination, the account does not need to be a roster member.
        /// This ensures nominees can close their own nominations
        ///
        /// If the vote passes nominees will be added to the roster members list
		///
		/// - `roster_id`: The v8 UUID for the roster
        /// - `nominee`: AccountId of the account being nominated
		///
		/// Emits `NominationClosed`
        #[pallet::call_index(230)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn nomination_close(origin: OriginFor<T>, roster_id: RosterId, nominee: T::AccountId) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            NominationCalls::<T>::close(who, roster_id, nominee)
        }
    }
}