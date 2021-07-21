// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Test utilities

#![cfg(test)]

pub use cid::Cid;

pub use sp_core::H256;
pub use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

pub use frame_support::{ord_parameter_types, parameter_types, traits::SortedMembers};
pub use frame_system::EnsureSignedBy;

pub use crate as pallet_alliance;

use super::*;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
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
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
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

pub struct AllianceIdentityVerifier;
impl IdentityVerifier<u64> for AllianceIdentityVerifier {
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

pub struct AllianceProposalProvider;
impl ProposalProvider<u64, H256, Call> for AllianceProposalProvider {
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
	pub const MaxBlacklistCount: u32 = 100;
}
impl Config for Test {
	type Event = Event;
	type Proposal = Call;
	type SuperMajorityOrigin = EnsureSignedBy<One, u64>;
	type Currency = Balances;
	type Slashed = ();
	type InitializeMembers = AllianceMotion;
	type MembershipChanged = AllianceMotion;
	type IdentityVerifier = AllianceIdentityVerifier;
	type ProposalProvider = AllianceProposalProvider;
	type MaxBlacklistCount = MaxBlacklistCount;
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
		System: frame_system::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		AllianceMotion: pallet_collective::<Instance1>::{Pallet, Storage, Origin<T>, Event<T>},
		Alliance: pallet_alliance::{Pallet, Call, Storage, Event<T>, Config<T>},
	}
);

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = GenesisConfig {
		balances: pallet_balances::GenesisConfig {
			balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 20)],
		},
		alliance: pallet_alliance::GenesisConfig {
			founders: vec![1, 2],
			fellows: vec![3],
			allies: vec![],
			phantom: Default::default(),
		},
	}
	.build_storage()
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[cfg(feature = "runtime-benchmarks")]
pub fn new_bench_ext() -> sp_io::TestExternalities {
	GenesisConfig::default().build_storage().unwrap().into()
}

pub fn test_cid() -> Cid {
	let cid = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n"
		.parse()
		.unwrap();
	Cid::new(cid)
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
