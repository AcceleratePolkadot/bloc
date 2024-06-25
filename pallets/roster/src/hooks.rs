use frame_support::pallet_macros::*;

#[pallet_section]
mod hooks {
	#[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			// Remove all nominations which have concluded
			for concluded_nomination in ConcludedNominations::<T>::take().iter() {
                Nominations::<T>::remove(&concluded_nomination.0, &concluded_nomination.1);
            }

			Weight::zero()
		}
	}
}