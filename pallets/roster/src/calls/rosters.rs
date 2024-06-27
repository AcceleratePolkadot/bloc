use crate::*;
use frame_support::pallet_prelude::*;
use sp_std::vec::Vec;

pub struct RosterCalls<T> {
    _phantom: PhantomData<T>
}


impl<T: Config> RosterCalls<T> {
    
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

    pub(crate) fn new(founder: T::AccountId, title: Vec<u8>) -> DispatchResultWithPostInfo {
        let bounded_title: BoundedVec<_, _> = title.try_into().map_err(|_| Error::<T>::InvalidRosterTitle)?;
        let roster_id = RosterId::from_tuple::<T>((&founder, &bounded_title));

        ensure!(!Rosters::<T>::contains_key(&roster_id), Error::<T>::RosterExists);

        let mut roster = Roster::new(&founder, &bounded_title);
        // Add founder as first member
        roster.members.try_push(founder.clone()).map_err(|_| Error::<T>::CouldNotAddMember)?;
        Rosters::<T>::insert(&roster_id, roster);

        pallet::Pallet::deposit_event(Event::<T>::NewRoster(founder,  bounded_title, roster_id));
        Ok(().into())
    }

    pub(crate) fn activate(founder: T::AccountId,  roster_id: RosterId) -> DispatchResultWithPostInfo {
        Self::update_roster_status(&founder, &roster_id, RosterStatus::Active)?;
        pallet::Pallet::deposit_event(Event::<T>::RosterStatusChanged(founder, roster_id, RosterStatus::Active));
        Ok(().into())
    }

    pub(crate) fn deactivate(founder: T::AccountId,  roster_id: RosterId) -> DispatchResultWithPostInfo {
        Self::update_roster_status(&founder, &roster_id, RosterStatus::Inactive)?;
        pallet::Pallet::deposit_event(Event::<T>::RosterStatusChanged(founder.clone(), roster_id.clone(), RosterStatus::Inactive));

        // When a roster is deactivated all active nominations are rejected
        let roster = Rosters::<T>::get(&roster_id).ok_or(Error::<T>::RosterDoesNotExist)?;
        for nominee in roster.nominations.iter() {
            Nominations::<T>::try_mutate(&nominee, &roster_id, |nomination| -> DispatchResult {
                nomination.as_mut().ok_or(Error::<T>::NominationDoesNotExist)?.status = NominationStatus::Rejected;
                Ok(())
            })?;
            ConcludedNominations::<T>::try_append((&nominee, &roster_id)).map_err(|_| Error::<T>::CouldNotAddToConcluded)?;
            pallet::Pallet::deposit_event(Event::<T>::NominationClosed(nominee.clone(), roster_id.clone(), founder.clone(),  NominationStatus::Rejected));
        }

        Ok(().into())

    }

    pub(crate) fn remove(founder: T::AccountId,  roster_id: RosterId) -> DispatchResultWithPostInfo {
        // We use `take()` here to also remove the item from storage
        // If an error gets raised later all state changes within the extrinsic will be discarded
        let roster = Rosters::<T>::take(&roster_id).ok_or(Error::<T>::RosterDoesNotExist)?;

        ensure!(roster.founder == founder, Error::<T>::PermissionDenied);
        ensure!(roster.status == RosterStatus::Inactive, Error::<T>::RosterActive);

        pallet::Pallet::deposit_event(Event::<T>::RosterRemoved(founder, roster_id));

        Ok(().into())

    }
}