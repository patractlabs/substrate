use frame_support::{assert_ok, Hashable};
use sp_runtime::traits::BlakeTwo256;

use super::*;
use crate::mock::*;

#[test]
fn propose_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let _proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let hash = BlakeTwo256::hash_of(&proposal);
		let _end = 4;
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
		let cid = Cid::new(
			"QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n"
				.parse()
				.unwrap(),
		);
		let proposal = make_set_rule_proposal(cid);
		let _proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let hash: H256 = proposal.blake2_256().into();
		let _end = 4;
		assert_ok!(Alliance::propose(
			Origin::signed(1),
			Box::new(proposal.clone())
		));
		assert_eq!(*AllianceMotion::proposals(), vec![hash]);
		assert_eq!(AllianceMotion::proposal_of(&hash), Some(proposal));
	});
}

#[test]
fn set_member() {}
