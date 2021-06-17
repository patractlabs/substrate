use frame_support::{assert_noop, assert_ok, Hashable};
use frame_system::{EventRecord, Phase};

use super::*;
use crate::mock::*;

#[test]
fn propose_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let hash: H256 = proposal.blake2_256().into();

		// only votable member can propose proposal, 4 is ally not have vote rights
		assert_noop!(
			Alliance::propose(Origin::signed(4), Box::new(proposal.clone())),
			Error::<Test, ()>::NotVotableMember
		);

		assert_ok!(Alliance::propose(
			Origin::signed(1),
			Box::new(proposal.clone())
		));
		assert_eq!(*AllianceMotion::proposals(), vec![hash]);
		assert_eq!(AllianceMotion::proposal_of(&hash), Some(proposal));
		assert_eq!(
			System::events(),
			vec![EventRecord {
				phase: Phase::Initialization,
				event: mock::Event::pallet_collective_Instance1(
					pallet_collective::RawEvent::Proposed(1, 0, hash, 3)
				),
				topics: vec![],
			}]
		);
	});
}

#[test]
fn vote_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let hash: H256 = proposal.blake2_256().into();
		assert_ok!(Alliance::propose(
			Origin::signed(1),
			Box::new(proposal.clone())
		));
		assert_ok!(Alliance::vote(Origin::signed(2), hash.clone(), 0, true));

		let record = |event| EventRecord {
			phase: Phase::Initialization,
			event,
			topics: vec![],
		};
		assert_eq!(
			System::events(),
			vec![
				record(mock::Event::pallet_collective_Instance1(
					pallet_collective::RawEvent::Proposed(1, 0, hash.clone(), 3)
				)),
				record(mock::Event::pallet_collective_Instance1(
					pallet_collective::RawEvent::Voted(2, hash.clone(), true, 2, 0)
				)),
			]
		);
	});
}

#[test]
fn veto_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let hash: H256 = proposal.blake2_256().into();
		assert_ok!(Alliance::propose(
			Origin::signed(1),
			Box::new(proposal.clone())
		));
		// only set_rule/elevate_ally can be veto
		assert_noop!(
			Alliance::veto(Origin::signed(1), hash.clone()),
			Error::<Test, ()>::NotVetoableProposal
		);

		let cid = Cid::new(
			"QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n"
				.parse()
				.unwrap(),
		);
		let vetoable_proposal = make_set_rule_proposal(cid);
		let vetoable_hash: H256 = vetoable_proposal.blake2_256().into();
		assert_ok!(Alliance::propose(
			Origin::signed(1),
			Box::new(vetoable_proposal.clone())
		));

		// only founder have veto rights, 3 is fellow
		assert_noop!(
			Alliance::veto(Origin::signed(3), vetoable_hash.clone()),
			Error::<Test, ()>::NotFounder
		);

		assert_ok!(Alliance::veto(Origin::signed(2), vetoable_hash.clone()));
		let record = |event| EventRecord {
			phase: Phase::Initialization,
			event,
			topics: vec![],
		};
		assert_eq!(
			System::events(),
			vec![
				record(mock::Event::pallet_collective_Instance1(
					pallet_collective::RawEvent::Proposed(1, 0, hash.clone(), 3)
				)),
				record(mock::Event::pallet_collective_Instance1(
					pallet_collective::RawEvent::Proposed(1, 1, vetoable_hash.clone(), 3)
				)),
				// record(mock::Event::pallet_collective_Instance1(pallet_collective::RawEvent::Voted(2, hash.clone(), true, 2, 0))),
				record(mock::Event::pallet_collective_Instance1(
					pallet_collective::RawEvent::Disapproved(vetoable_hash.clone())
				)),
			]
		);
	})
}

#[test]
fn close_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let hash = BlakeTwo256::hash_of(&proposal);
		assert_ok!(Alliance::propose(
			Origin::signed(1),
			Box::new(proposal.clone())
		));
		assert_ok!(Alliance::vote(Origin::signed(2), hash.clone(), 0, true));
		assert_ok!(Alliance::vote(Origin::signed(3), hash.clone(), 0, true));
		assert_ok!(Alliance::close(
			Origin::signed(1),
			hash.clone(),
			0,
			proposal_weight,
			proposal_len
		));

		let record = |event| EventRecord {
			phase: Phase::Initialization,
			event,
			topics: vec![],
		};
		assert_eq!(
			System::events(),
			vec![
				record(mock::Event::pallet_collective_Instance1(
					pallet_collective::RawEvent::Proposed(1, 0, hash.clone(), 3)
				)),
				record(mock::Event::pallet_collective_Instance1(
					pallet_collective::RawEvent::Voted(2, hash.clone(), true, 2, 0)
				)),
				record(mock::Event::pallet_collective_Instance1(
					pallet_collective::RawEvent::Voted(3, hash.clone(), true, 3, 0)
				)),
				record(mock::Event::pallet_collective_Instance1(
					pallet_collective::RawEvent::Closed(hash.clone(), 3, 0)
				)),
				record(mock::Event::pallet_collective_Instance1(
					pallet_collective::RawEvent::Approved(hash.clone())
				)),
				record(mock::Event::pallet_collective_Instance1(
					pallet_collective::RawEvent::Executed(
						hash.clone(),
						Err(DispatchError::BadOrigin)
					)
				))
			]
		);
	});
}

#[test]
fn set_rule_works() {
	new_test_ext().execute_with(|| {
		let cid = Cid::new(
			"QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n"
				.parse()
				.unwrap(),
		);
		assert_ok!(Alliance::set_rule(Origin::signed(1), cid));
		assert_eq!(Alliance::rule(), Some(cid));

		System::assert_last_event(mock::Event::pallet_alliance(crate::Event::NewRule(cid)));
	});
}

#[test]
fn announce_works() {
	new_test_ext().execute_with(|| {
		let cid = Cid::new(
			"QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n"
				.parse()
				.unwrap(),
		);
		assert_ok!(Alliance::announce(Origin::signed(1), cid));
		assert_eq!(Alliance::announcements(), vec![cid]);

		System::assert_last_event(mock::Event::pallet_alliance(crate::Event::NewAnnouncement(
			cid,
		)));
	});
}

#[test]
fn submit_candidacy_works() {
	new_test_ext().execute_with(|| {
		// check already member
		assert_noop!(
			Alliance::submit_candidacy(Origin::signed(1)),
			Error::<Test, ()>::AlreadyMember
		);

		// check already in blacklist
		assert_ok!(Alliance::add_blacklist(
			Origin::signed(1),
			vec![UserIdentity::AccountId(4)]
		));
		assert_noop!(
			Alliance::submit_candidacy(Origin::signed(4)),
			Error::<Test, ()>::AlreadyInBlacklist
		);
		assert_ok!(Alliance::remove_blacklist(
			Origin::signed(1),
			vec![UserIdentity::AccountId(4)]
		));

		// check deposit funds
		assert_noop!(
			Alliance::submit_candidacy(Origin::signed(5)),
			Error::<Test, ()>::InsufficientCandidateFunds
		);

		// success to submit
		assert_ok!(Alliance::submit_candidacy(Origin::signed(4)));
		assert_eq!(Alliance::deposit_of(4), Some(25));
		assert_eq!(Alliance::candidates(), vec![4]);

		// check already candidate
		assert_noop!(
			Alliance::submit_candidacy(Origin::signed(4)),
			Error::<Test, ()>::AlreadyCandidate
		);
	});
}

#[test]
fn nominate_candidacy_works() {
	new_test_ext().execute_with(|| {
		// check already member
		assert_noop!(
			Alliance::nominate_candidacy(Origin::signed(1), 2),
			Error::<Test, ()>::AlreadyMember
		);

		// only votable member(founder/fellow) have nominate right
		assert_noop!(
			Alliance::nominate_candidacy(Origin::signed(5), 4),
			Error::<Test, ()>::NotVotableMember
		);

		// check already in blacklist
		assert_ok!(Alliance::add_blacklist(
			Origin::signed(1),
			vec![UserIdentity::AccountId(4)]
		));
		assert_noop!(
			Alliance::nominate_candidacy(Origin::signed(1), 4),
			Error::<Test, ()>::AlreadyInBlacklist
		);
		assert_ok!(Alliance::remove_blacklist(
			Origin::signed(1),
			vec![UserIdentity::AccountId(4)]
		));

		// success to nominate
		assert_ok!(Alliance::nominate_candidacy(Origin::signed(1), 4));
		assert_eq!(Alliance::deposit_of(4), None);
		assert_eq!(Alliance::candidates(), vec![4]);

		// check already candidate
		assert_noop!(
			Alliance::nominate_candidacy(Origin::signed(1), 4),
			Error::<Test, ()>::AlreadyCandidate
		);
	});
}

#[test]
fn approve_candidate_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Alliance::approve_candidate(Origin::signed(1), 4),
			Error::<Test, ()>::NotCandidate
		);

		assert_ok!(Alliance::submit_candidacy(Origin::signed(4)));
		assert_eq!(Alliance::candidates(), vec![4]);

		assert_ok!(Alliance::approve_candidate(Origin::signed(1), 4));
		assert_eq!(Alliance::candidates(), Vec::<u64>::new());
		assert_eq!(Alliance::members(MemberRole::Ally), vec![4]);
	});
}

#[test]
fn reject_candidate_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Alliance::reject_candidate(Origin::signed(1), 4),
			Error::<Test, ()>::NotCandidate
		);

		assert_ok!(Alliance::submit_candidacy(Origin::signed(4)));
		assert_eq!(Alliance::deposit_of(4), Some(25));
		assert_eq!(Alliance::candidates(), vec![4]);

		assert_ok!(Alliance::reject_candidate(Origin::signed(1), 4));
		assert_eq!(Alliance::deposit_of(4), None);
		assert_eq!(Alliance::candidates(), Vec::<u64>::new());
	});
}

#[test]
fn elevate_ally_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Alliance::elevate_ally(Origin::signed(1), 4),
			Error::<Test, ()>::NotAlly
		);

		assert_ok!(Alliance::submit_candidacy(Origin::signed(4)));
		assert_ok!(Alliance::approve_candidate(Origin::signed(1), 4));
		assert_eq!(Alliance::members(MemberRole::Ally), vec![4]);
		assert_eq!(Alliance::members(MemberRole::Fellow), vec![3]);

		assert_ok!(Alliance::elevate_ally(Origin::signed(1), 4));
		assert_eq!(Alliance::members(MemberRole::Ally), Vec::<u64>::new());
		assert_eq!(Alliance::members(MemberRole::Fellow), vec![3, 4]);
	});
}

#[test]
fn retire_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_kick_member_proposal(2);
		assert_ok!(Alliance::propose(
			Origin::signed(1),
			Box::new(proposal.clone())
		));
		assert_noop!(
			Alliance::retire(Origin::signed(2)),
			Error::<Test, ()>::KickingMember
		);

		assert_noop!(
			Alliance::retire(Origin::signed(4)),
			Error::<Test, ()>::NotMember
		);

		assert_eq!(Alliance::members(MemberRole::Fellow), vec![3]);
		assert_ok!(Alliance::retire(Origin::signed(3)));
		assert_eq!(Alliance::members(MemberRole::Fellow), Vec::<u64>::new());
	});
}

#[test]
fn kick_member_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Alliance::kick_member(Origin::signed(1), 2),
			Error::<Test, ()>::NotKickingMember
		);

		let proposal = make_kick_member_proposal(2);
		assert_ok!(Alliance::propose(
			Origin::signed(1),
			Box::new(proposal.clone())
		));
		assert_eq!(Alliance::kicking_member(2), true);
		assert_eq!(Alliance::members(MemberRole::Founder), vec![1, 2]);

		assert_ok!(Alliance::kick_member(Origin::signed(1), 2));
		assert_eq!(Alliance::members(MemberRole::Founder), vec![1]);
	});
}

#[test]
fn add_blacklist_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Alliance::add_blacklist(
			Origin::signed(1),
			vec![
				UserIdentity::AccountId(3),
				UserIdentity::Website("abc".as_bytes().to_vec())
			]
		));
		assert_eq!(Alliance::account_blacklist(), vec![3]);
		assert_eq!(
			Alliance::website_blacklist(),
			vec!["abc".as_bytes().to_vec()]
		);

		assert_noop!(
			Alliance::add_blacklist(Origin::signed(1), vec![UserIdentity::AccountId(3)]),
			Error::<Test, ()>::AlreadyInBlacklist
		);
	});
}

#[test]
fn remove_blacklist_works() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Alliance::remove_blacklist(Origin::signed(1), vec![UserIdentity::AccountId(3)]),
			Error::<Test, ()>::NotInBlacklist
		);

		assert_ok!(Alliance::add_blacklist(
			Origin::signed(1),
			vec![UserIdentity::AccountId(3)]
		));
		assert_eq!(Alliance::account_blacklist(), vec![3]);
		assert_ok!(Alliance::remove_blacklist(
			Origin::signed(1),
			vec![UserIdentity::AccountId(3)]
		));
		assert_eq!(Alliance::account_blacklist(), Vec::<u64>::new());
	});
}
