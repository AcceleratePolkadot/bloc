use frame_support::pallet_macros::*;

#[pallet_section]
mod hooks {
	#[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
}