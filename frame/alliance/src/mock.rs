pub use cid::Cid;
pub use frame_support::{ord_parameter_types, parameter_types, traits::SortedMembers};
pub use frame_system::EnsureSignedBy;
pub use multihash::U64;
pub use sp_core::H256;
pub use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

use super::*;
use crate as pallet_alliance;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for Test {
	type AccountData = pallet_balances::AccountData<u64>;
	type AccountId = u64;
	type BaseCallFilter = ();
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockNumber = u64;
	type BlockWeights = ();
	type Call = Call;
	type DbWeight = ();
	type Event = Event;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type Origin = Origin;
	type PalletInfo = PalletInfo;
	type SS58Prefix = ();
	type SystemWeightInfo = ();
	type Version = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxLocks: u32 = 10;
}
impl pallet_balances::Config for Test {
	type AccountStore = System;
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = MaxLocks;
	type WeightInfo = ();
}

parameter_types! {
	pub const MotionDuration: u64 = 3;
	pub const MaxProposals: u32 = 100;
	pub const MaxMembers: u32 = 100;
}
type AllianceCollective = pallet_collective::Instance1;
impl pallet_collective::Config<AllianceCollective> for Test {
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type Event = Event;
	type MaxMembers = MaxMembers;
	type MaxProposals = MaxProposals;
	type MotionDuration = MotionDuration;
	type Origin = Origin;
	type Proposal = Call;
	type WeightInfo = ();
}

pub struct AllyIdentityVerifier;
impl IdentityVerifier<u64> for AllyIdentityVerifier {
	fn super_account_id(_who: &u64) -> Option<u64> {
		None
	}

	fn verify_identity(_who: &u64, _field: u64) -> bool {
		true
	}

	fn verify_judgement(_who: &u64) -> bool {
		true
	}
}
pub struct AlliProposalProvider;
impl ProposalProvider<u64, H256, Call> for AlliProposalProvider {
	fn propose_proposal(
		who: u64,
		threshold: u32,
		proposal: Call,
		proposal_hash: H256,
	) -> Result<u32, DispatchError> {
		AllianceMotion::do_propose(who, threshold, proposal, proposal_hash)
	}

	fn vote_proposal(
		who: u64,
		proposal: H256,
		index: ProposalIndex,
		approve: bool,
	) -> Result<bool, DispatchError> {
		AllianceMotion::do_vote(who, proposal, index, approve)
	}

	fn veto_proposal(proposal_hash: H256) -> u32 {
		AllianceMotion::do_disapprove_proposal(proposal_hash)
	}

	fn close_proposal(
		proposal_hash: H256,
		proposal_index: ProposalIndex,
		proposal_weight_bound: Weight,
		length_bound: u32,
	) -> Result<(Weight, Pays), DispatchError> {
		AllianceMotion::do_close(
			proposal_hash,
			proposal_index,
			proposal_weight_bound,
			length_bound,
		)
	}

	fn proposal_of(proposal_hash: H256) -> Option<Call> {
		AllianceMotion::proposal_of(proposal_hash)
	}
}
ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
	pub const Three: u64 = 3;
	pub const Four: u64 = 4;
	pub const Five: u64 = 5;
}
parameter_types! {
	pub const CandidateDeposit: u64 = 25;
}
impl Config for Test {
	type CandidateDeposit = CandidateDeposit;
	type Currency = Balances;
	type Event = Event;
	type IdentityVerifier = AllyIdentityVerifier;
	type InitializeMembers = AllianceMotion;
	type SuperMajorityOrigin = EnsureSignedBy<One, u64>;
	type MembershipChanged = AllianceMotion;
	type Proposal = Call;
	type ProposalProvider = AlliProposalProvider;
	type Slashed = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		AllianceMotion: pallet_collective::<Instance1>::{Pallet, Storage, Origin<T>, Event<T>, Config<T>},
		Alliance: pallet_alliance::{Pallet, Call, Storage, Event<T>, Config<T>},
	}
);

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext: sp_io::TestExternalities = GenesisConfig {
		frame_system: Default::default(),
		pallet_balances: pallet_balances::GenesisConfig {
			balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 20)],
		},
		pallet_collective_Instance1: pallet_collective::GenesisConfig {
			..Default::default()
		},
		pallet_alliance: pallet_alliance::GenesisConfig {
			founders: vec![1, 2],
			fellows: vec![3],
			allies: vec![],
			phantom: Default::default(),
		},
	}
	.build_storage()
	.unwrap()
	.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn make_proposal(value: u64) -> Call {
	Call::System(frame_system::Call::remark(value.encode()))
}

pub fn make_set_rule_proposal(cid: Cid) -> Call {
	Call::Alliance(pallet_alliance::Call::set_rule(cid))
}

pub fn make_kick_member_proposal(who: u64) -> Call {
	Call::Alliance(pallet_alliance::Call::kick_member(who))
}
