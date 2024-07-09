use frame_support::pallet_macros::*;

#[pallet_section]
mod config {

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Pallet ID, root account for fungible assets
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Trait for handling fungible tokens
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// Maximum length of the Roster title byte array
		#[pallet::constant]
		type TitleMaxLength: Get<u32>;

		/// Maximum number of members that can be in each roster
		#[pallet::constant]
		type MembersMax: Get<u32>;

		/// Maximum number of votes that can be submitted for each nomination
		/// This should probably be equal to MembersMax
		#[pallet::constant]
		type NominationVotesMax: Get<u32>;

		/// Length of the nomination voting period in blocks
		#[pallet::constant]
		type NominationVotingPeriod: Get<BlockNumberFor<Self>>;

		/// Maximum number of nominations that can be in progress per roster
		/// at any one time
		#[pallet::constant]
		type NominationsPerRosterMax: Get<u32>;

		/// Maximum number of nominations which can be concluded at the same time
		/// Concluded nominations get removed when a new block is initialized
		/// so this is effectively the number of concluded nominations per block
		#[pallet::constant]
		type ConcludedNominationsMax: Get<u32>;

		/// Period in which seconds should come forward to support an expulsion proposal
		/// If this period expires and a proposal has not been seconded it can be
		/// closed with prejudice
		#[pallet::constant]
		type ExpulsionProposalAwaitingSecondPeriod: Get<BlockNumberFor<Self>>;

		/// Length of the expulsion proposal voting period in blocks
		#[pallet::constant]
		type ExpulsionProposalVotingPeriod: Get<BlockNumberFor<Self>>;

		/// Maximum number of expulsion proposals that can be in progress per roster
		/// at any one time
		#[pallet::constant]
		type ExpulsionProposalsPerRosterMax: Get<u32>;

		/// Minimum number of seconds required to open an expulsion proposal for voting
		#[pallet::constant]
		type ExpulsionProposalSecondThreshold: Get<u32>;

		/// Maximum number of accounts which can second an expulsion proposal
		#[pallet::constant]
		type SecondsMax: Get<u32>;

		/// Maximum length of the expulsion reason byte array
		#[pallet::constant]
		type ExpulsionReasonMaxLength: Get<u32>;

		/// Minimum length of the expulsion reason byte array
		#[pallet::constant]
		type ExpulsionReasonMinLength: Get<u32>;

		/// Maximum number of votes that can be submitted for each expulsion proposal
		/// This should probably be equal to MembersMax
		#[pallet::constant]
		type ExpulsionProposalVotesMax: Get<u32>;

		/// Period in which an account can not motion or second a new proposal after a previous
		/// proposal was dismissed with prejudice
		#[pallet::constant]
		type ExpulsionProposalLockoutPeriod: Get<BlockNumberFor<Self>>;

		/// Percentage of members voting nay required to dismiss expulsion proposal with prejudice
		#[pallet::constant]
		type ExpulsionProposalSuperMajority: Get<u32>;

		/// Percentage of total members voting required for quorum to be reached
		#[pallet::constant]
		type ExpulsionProposalQuorum: Get<u32>;
	}
}
