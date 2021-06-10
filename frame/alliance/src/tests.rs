use frame_support::{assert_noop, assert_ok, traits::Currency, Hashable};
use frame_system::{self as system, EventRecord, Phase};
use pallet_balances::Error as BalancesError;
use sp_runtime::{traits::BlakeTwo256, TokenError};

use super::*;
use crate::{mock::*, Error};

#[test]
fn propose_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let hash = BlakeTwo256::hash_of(&proposal);
		let end = 4;
		assert_ok!(Alliance::propose(
			Origin::signed(1),
			Box::new(proposal.clone())
		));
		assert_eq!(*AllianceMotion::proposals(), vec![hash]);
		assert_eq!(AllianceMotion::proposal_of(&hash), Some(proposal));
	});
}

#[test]
fn propose_set_rule_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_set_rule_proposal(Cid::default());
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let hash: H256 = proposal.blake2_256().into();
		let end = 4;
		assert_ok!(Alliance::propose(
			Origin::signed(1),
			Box::new(proposal.clone())
		));
		assert_eq!(*AllianceMotion::proposals(), vec![hash]);
		assert_eq!(AllianceMotion::proposal_of(&hash), Some(proposal));
	});
}
