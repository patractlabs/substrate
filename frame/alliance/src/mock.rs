use crate as pallet_alliance;
use super::*;
pub use multihash::U64;
pub use cid::Cid;
pub use frame_support::{parameter_types, ord_parameter_types};
pub use frame_system::EnsureSignedBy;
pub use sp_core::H256;
pub use sp_runtime::{traits::{BlakeTwo256, IdentityLookup}, BuildStorage, testing::Header};

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Call = Call;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxLocks: u32 = 10;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type Balance = u64;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

parameter_types! {
    pub const MotionDuration: u64 = 3;
    pub const MaxProposals: u32 = 100;
    pub const MaxMembers: u32 = 100;
}

type AllianceCollective = pallet_collective::Instance1;
impl pallet_collective::Config<AllianceCollective> for Test {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    type MotionDuration = MotionDuration;
    type MaxProposals = MaxProposals;
    type MaxMembers = MaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = ();
}

pub struct AllyIdentityVerifier;

impl IdentityVerifier<u64> for AllyIdentityVerifier {
    fn verify_identity(_who: u64, _field: u64) -> bool {
        true
    }
}

pub struct AlliProposalProvider;
impl ProposalProvider<u64, H256, Call> for AlliProposalProvider {
    fn propose_proposal(who: u64, threshold: u32, proposal: Call,
                        proposal_hash: H256) -> Result<u32, DispatchError> {
        AllianceMotion::do_propose(who, threshold, proposal, proposal_hash)
    }

    fn veto_proposal(proposal_hash: H256) -> u32 {
        AllianceMotion::do_disapprove_proposal(proposal_hash)
    }

    fn close_proposal(proposal_hash: H256,
                      proposal_index: ProposalIndex,
                      proposal_weight_bound: Weight,
                      length_bound: u32,
    ) -> Result<(Weight, Pays), DispatchError> {
        AllianceMotion::close_proposal(proposal_hash, proposal_index, proposal_weight_bound, length_bound)
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
    type Event = Event;
    type Proposal = Call;
    type FounderInitOrigin = EnsureSignedBy<Five, u64>;
    type MajorityOrigin = EnsureSignedBy<Three, u64>;
    type Currency = Balances;
    type InitializeMembers = AllianceMotion;
    type MembershipChanged = AllianceMotion;
    type Slashed = ();
    type IdentityVerifier = AllyIdentityVerifier;
    type ProposalProvider = AlliProposalProvider;
    type CandidateDeposit = CandidateDeposit;
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
        AllianceMotion: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
		Alliance: pallet_alliance::{Pallet, Call, Storage, Event<T>, Config<T>},
	}
);

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext: sp_io::TestExternalities = GenesisConfig {
        frame_system: Default::default(),
        pallet_balances: pallet_balances::GenesisConfig {
            balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50)],
        },
        pallet_collective_Instance1: pallet_collective::GenesisConfig {
            members: vec![1, 2, 3],
            phantom: Default::default(),
        },
        pallet_alliance: pallet_alliance::GenesisConfig {
            founders: vec![1, 2],
            fellows: vec![],
            phantom: Default::default(),
        },
    }.build_storage().unwrap().into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn make_proposal(value: u64) -> Call {
    Call::System(frame_system::Call::remark(value.encode()))
}

pub fn make_set_rule_proposal(cid: Cid) -> Call {
    Call::Alliance(pallet_alliance::Call::set_rule(cid))
}