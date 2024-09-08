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
			}

			// Remove all expulsion proposals which have concluded
			for concluded_expulsion_proposal in ConcludedExpulsionProposals::<T>::take().iter() {
				let (roster_id, motioner, subject) = concluded_expulsion_proposal;
				// Remove expulsion proposal
				ExpulsionProposals::<T>::remove((&roster_id, &motioner, &subject));
			}

			Weight::zero()
		}
	}
}
