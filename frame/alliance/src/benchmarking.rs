// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Alliance pallet benchmarking.

use sp_runtime::traits::Bounded;
use sp_std::mem::size_of;

use frame_benchmarking::{benchmarks_instance_pallet, impl_benchmark_test_suite};
use frame_system::{Call as SystemCall, Pallet as System, RawOrigin as SystemOrigin};

use super::*;
use crate::Pallet as Alliance;

benchmarks_instance_pallet! {
	set_rule {
		let cid = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n".parse().unwrap();
		let rule = Cid::new(cid);
	}: _(SystemOrigin::Signed(1), rule.clone())
	verify {
		assert_eq!(Alliance::<T, I>::rule(), Some(rule));
	}

	announce {
		let cid = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n".parse().unwrap();
		let announcement = Cid::new(cid);
	}: _(SystemOrigin::Signed(1), announcement.clone())
	verify {
		assert!(Alliance::<T, I>::announcements().contains(announcement));
	}

	submit_candidacy {

	}: _(SystemOrigin::Signed(100))
	verify {

	}

	nominate_candidacy {

	}: _(SystemOrigin::Signed(1))
	verify {

	}

	approve_candidate {

	}: _(SystemOrigin::Signed(1))
	verify {

	}

	reject_candidate {

	}: _(SystemOrigin::Signed(1))
	verify {

	}

	elevate_ally {

	}: _(SystemOrigin::Signed(1))
	verify {

	}

	retire {

	}: _(SystemOrigin::Signed(1))
	verify {

	}

	kick_member {

	}: _(SystemOrigin::Signed(1))
	verify {

	}

	add_blacklist {

	}: _(SystemOrigin::Signed(1))
	verify {

	}

	remove_blacklist {

	}: _(SystemOrigin::Signed(1))
	verify {

	}
}

impl_benchmark_test_suite!(Alliance, crate::tests::new_test_ext(), crate::tests::Test);
