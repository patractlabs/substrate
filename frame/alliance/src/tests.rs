use frame_support::{assert_ok, Hashable};
use frame_system::{EventRecord, Phase};

use super::*;
use crate::mock::*;

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
fn veto_set_rule_works() {
	new_test_ext().execute_with(|| {
		let cid = Cid::new(
			"QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n"
				.parse()
				.unwrap(),
		);
		let proposal = make_set_rule_proposal(cid);
		let hash: H256 = proposal.blake2_256().into();
		assert_ok!(Alliance::propose(
			Origin::signed(1),
			Box::new(proposal.clone())
		));
		assert_ok!(Alliance::veto(Origin::signed(2), hash.clone()));
	})
}

#[test]
fn close_set_rule_works() {
	new_test_ext().execute_with(|| {
		let cid = Cid::new(
			"QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n"
				.parse()
				.unwrap(),
		);
		let proposal = make_set_rule_proposal(cid);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let hash = BlakeTwo256::hash_of(&proposal);
		assert_ok!(Alliance::propose(
			Origin::signed(1),
			Box::new(proposal.clone())
		));
		assert_ok!(AllianceMotion::vote(
			Origin::signed(2),
			hash.clone(),
			0,
			true
		));
		assert_ok!(AllianceMotion::vote(
			Origin::signed(3),
			hash.clone(),
			0,
			true
		));
		assert_ok!(Alliance::close(
			Origin::signed(1),
			hash.clone(),
			0,
			proposal_weight,
			proposal_len
		));

		// assert_eq!(Alliance::rule(), Some(cid));
		System::assert_last_event(mock::Event::pallet_alliance(crate::Event::NewRule(cid)));
	});
}
