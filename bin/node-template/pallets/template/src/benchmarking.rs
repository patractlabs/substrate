// This file is part of Substrate.

// Copyright (C) 2020 Parity Technologies (UK) Ltd.
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

//! Balances pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller};

benchmarks! {
	_ { }

	bls12_377_pairing {
        let caller: T::AccountId = whitelisted_caller();
	}: bls12_377_pairing(RawOrigin::Signed(caller.clone()))
	verify {
	}

    bls12_377_ops {
        let caller: T::AccountId = whitelisted_caller();
	}: bls12_377_ops(RawOrigin::Signed(caller.clone()))
	verify {
	}

    bls12_381_pairing {
        let caller: T::AccountId = whitelisted_caller();
	}: bls12_381_pairing(RawOrigin::Signed(caller.clone()))
	    verify {
	    }

    bls12_381_ops {
        let caller: T::AccountId = whitelisted_caller();
	}: bls12_381_ops(RawOrigin::Signed(caller.clone()))
	verify {
    }

    alt_bn128_pairing {
        let caller: T::AccountId = whitelisted_caller();
	}: alt_bn128_pairing(RawOrigin::Signed(caller.clone()))
	    verify {
	    }

    alt_bn128_ops {
        let caller: T::AccountId = whitelisted_caller();
	}: alt_bn128_ops(RawOrigin::Signed(caller.clone()))
	verify {
    }

    bw6_761_pairing {
        let caller: T::AccountId = whitelisted_caller();
	}: bw6_761_pairing(RawOrigin::Signed(caller.clone()))
	    verify {
	    }

    bw6_761_ops {
        let caller: T::AccountId = whitelisted_caller();
	}: bw6_761_ops(RawOrigin::Signed(caller.clone()))
    verify {
    }

    cp6_782_pairing {
        let caller: T::AccountId = whitelisted_caller();
	}: cp6_782_pairing(RawOrigin::Signed(caller.clone()))
	    verify {
	    }

    cp6_782_ops {
        let caller: T::AccountId = whitelisted_caller();
	}: cp6_782_ops(RawOrigin::Signed(caller.clone()))
    verify {
    }
}
