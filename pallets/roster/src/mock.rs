use frame_support::{
	derive_impl, parameter_types,
	traits::{Everything, OnFinalize, OnInitialize},
	weights::Weight,
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u32;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Roster: crate::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type RuntimeTask = RuntimeTask;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = MockedMigrator;
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

parameter_types! {
	pub static Ongoing: bool = false;
}

pub struct MockedMigrator;
impl frame_support::migrations::MultiStepMigrator for MockedMigrator {
	fn ongoing() -> bool {
		Ongoing::get()
	}

	fn step() -> Weight {
		Weight::zero()
	}
}

parameter_types! {
	pub const TitleMaxLength: u32 = 200;
	pub const MembersMax: u32 = u32::MAX;
	pub const NominationVotesMax: u32 = u32::MAX;
	pub const NominationVotingPeriod: BlockNumber = 1;
	pub const ConcludedNominationsMax: u32 = u32::MAX;
	pub const NominationsPerRosterMax: u32 = u32::MAX;
	pub const ExpulsionProposalAwaitingSecondPeriod: BlockNumber = 1;
	pub const ExpulsionProposalVotingPeriod: BlockNumber = 1;
	pub const ExpulsionProposalsPerRosterMax: u32 = u32::MAX;
	pub const ExpulsionProposalSecondThreshold: u32 = 1;
	pub const SecondsMax: u32 = u32::MAX;
	pub const ExpulsionReasonMaxLength: u32 = 5000;
	pub const ExpulsionReasonMinLength: u32 = 200;
	pub const ExpulsionProposalVotesMax: u32 = u32::MAX;
	pub const ExpulsionProposalLockoutPeriod: BlockNumber = 1;
	pub const ExpulsionProposalSuperMajority: u32 = 75;
	pub const ExpulsionProposalQuorum: u32 = 50;
}

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type TitleMaxLength = TitleMaxLength;
	type MembersMax = MembersMax;
	type NominationVotesMax = NominationVotesMax;
	type NominationVotingPeriod = NominationVotingPeriod;
	type ConcludedNominationsMax = ConcludedNominationsMax;
	type NominationsPerRosterMax = NominationsPerRosterMax;
	type ExpulsionProposalAwaitingSecondPeriod = ExpulsionProposalAwaitingSecondPeriod;
	type ExpulsionProposalVotingPeriod = ExpulsionProposalVotingPeriod;
	type ExpulsionProposalsPerRosterMax = ExpulsionProposalsPerRosterMax;
	type ExpulsionProposalSecondThreshold = ExpulsionProposalSecondThreshold;
	type SecondsMax = SecondsMax;
	type ExpulsionReasonMaxLength = ExpulsionReasonMaxLength;
	type ExpulsionReasonMinLength = ExpulsionReasonMinLength;
	type ExpulsionProposalVotesMax = ExpulsionProposalVotesMax;
	type ExpulsionProposalLockoutPeriod = ExpulsionProposalLockoutPeriod;
	type ExpulsionProposalSuperMajority = ExpulsionProposalSuperMajority;
	type ExpulsionProposalQuorum = ExpulsionProposalQuorum;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}

/// Run until a particular block.
pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			Roster::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::reset_events();
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Roster::on_initialize(System::block_number());
	}
}
