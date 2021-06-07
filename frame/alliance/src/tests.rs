use super::*;
use crate::{Error, mock::*};
use sp_runtime::{TokenError, traits::BlakeTwo256};
use frame_support::{assert_ok, assert_noop, traits::Currency, Hashable};
use pallet_balances::Error as BalancesError;
use frame_system::{self as system, EventRecord, Phase};

#[test]
fn propose_works() {
    new_test_ext().execute_with(|| {
        let proposal = make_proposal(42);
        let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
        let hash: H256 = proposal.blake2_256().into();
        let end = 4;
        assert_ok!(Alliance::propose(Origin::signed(1), Box::new(proposal.clone())));
        assert_eq!(*AllianceMotion::proposals(), vec![hash]);
        assert_eq!(AllianceMotion::proposal_of(&hash), Some(proposal));
    });
}

#[test]
fn  propose_set_rule_works() {
    new_test_ext().execute_with(|| {
        let proposal = make_set_rule_proposal(Cid::default());
        let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
        let hash: H256 = proposal.blake2_256().into();
        let end = 4;
        assert_ok!(Alliance::propose(Origin::signed(1), Box::new(proposal.clone())));
        assert_eq!(*AllianceMotion::proposals(), vec![hash]);
        assert_eq!(AllianceMotion::proposal_of(&hash), Some(proposal));
    });
}