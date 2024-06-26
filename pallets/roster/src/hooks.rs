use frame_support::pallet_macros::*;

#[pallet_section]
mod hooks {
	#[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			// Remove all nominations which have concluded
			for concluded_nomination in ConcludedNominations::<T>::take().iter() {
				let (nominee, roster_id) = concluded_nomination;
                Nominations::<T>::remove(&nominee, &roster_id);

				// Remove references to these nominations from the roster
				let _ = Rosters::<T>::try_mutate(&roster_id, |roster| -> Result<(), ()> {
					if let Some(roster) = roster {
						roster.nominations.retain(|n| n != nominee);
					}
					Ok(())
				});
            }

			Weight::zero()
		}
	}
}