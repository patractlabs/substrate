//! Jupiter primitives - IO Module
#![allow(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
use sp_runtime_interface::runtime_interface;

/// ZK-SNARKs runtime interface
#[runtime_interface]
pub trait ZkSnarks {
    fn bls12_377_add() {
        megaclite::tests::bls12_377_add();
    }

    fn bls12_377_mul() {
        megaclite::tests::bls12_377_mul();
    }

    fn bls12_377_pairing_two() {
        megaclite::tests::bls12_377_pairing();
    }

    fn bls12_377_pairing_six() {
        megaclite::tests::bls12_377_pairing_six();
    }

    fn bls12_381_add() {
        megaclite::tests::bls12_381_add();
    }

    fn bls12_381_mul() {
        megaclite::tests::bls12_381_mul();
    }

    fn bls12_381_pairing_two() {
        megaclite::tests::bls12_381_pairing();
    }

    fn bls12_381_pairing_six() {
        megaclite::tests::bls12_381_pairing_six();
    }

    fn bn254_add() {
        megaclite::tests::bn254_add();
    }

    fn bn254_mul() {
        megaclite::tests::bn254_mul();
    }

    fn bn254_pairing_two() {
        megaclite::tests::bn254_pairing();
    }

    fn bn254_pairing_six() {
        megaclite::tests::bn254_pairing_six();
    }

    fn bw6_761_add() {
        megaclite::tests::bw6_761_add();
    }

    fn bw6_761_mul() {
        megaclite::tests::bw6_761_mul();
    }

    fn bw6_761_pairing_two() {
        megaclite::tests::bw6_761_pairing();
    }

    fn bw6_761_pairing_six() {
        megaclite::tests::bw6_761_pairing_six();
    }

    fn cp6_782_add() {
        megaclite::tests::cp6_782_add();
    }

    fn cp6_782_mul() {
        megaclite::tests::cp6_782_mul();
    }

    fn cp6_782_pairing_two() {
        megaclite::tests::cp6_782_pairing();
    }

    fn cp6_782_pairing_six() {
        megaclite::tests::cp6_782_pairing_six();
    }
}
