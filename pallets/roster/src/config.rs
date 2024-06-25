use frame_support::pallet_macros::*;

#[pallet_section]
mod config {

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type TitleMaxLength: Get<u32>;

        #[pallet::constant]
        type MembersMax: Get<u32>;

        #[pallet::constant]
        type NominationVotesMax: Get<u32>;

        #[pallet::constant]
        type NominationVotingPeriod: Get<BlockNumberFor<Self>>;
	}
}