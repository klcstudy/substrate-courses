use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use super::*;

#[test]
fn create_claim_works() {
    new_test_ext().execute_with(|| {
        let proof = vec![0, 1];

        assert_ok!(PoeModule::create_claim(Origin::signed(1), proof.clone()));
        assert_eq!(Proofs::<Test>::get(&proof), (1, frame_system::Module::<Test>::block_number()));
    })
}

#[test]
fn create_claim_fail_when_already_exist() {
    new_test_ext().execute_with(|| {
        let proof = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), proof.clone());

        assert_noop!(
			PoeModule::create_claim(Origin::signed(1), proof.clone()),
			Error::<Test>::ProofAlreadyClaimed
		);
    })
}

#[test]
fn revoke_claim_works() {
    new_test_ext().execute_with(|| {
        let proof = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), proof.clone());

        assert_ok!(PoeModule::revoke_claim(Origin::signed(1), proof.clone()));
    })
}

#[test]
fn revoke_claim_not_exist() {
    new_test_ext().execute_with(|| {
        let proof = vec![0, 1];

        assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), proof.clone()),
			Error::<Test>::NoSuchProof
		);
    })
}

#[test]
fn revoke_claim_not_owner() {
    new_test_ext().execute_with(|| {
        let proof = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), proof.clone());

        assert_noop!(
			PoeModule::revoke_claim(Origin::signed(2), proof.clone()),
			Error::<Test>::NotProofOwner
		);
    })
}

#[test]
fn transfer_claim_works() {
    new_test_ext().execute_with(|| {
        let proof = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), proof.clone());

        assert_ok!(PoeModule::transfer_claim(Origin::signed(1), proof.clone(), 2));
    })
}

#[test]
fn transfer_claim_not_exist() {
    new_test_ext().execute_with(|| {
        let proof = vec![0, 1];

        assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), proof.clone(), 2),
			Error::<Test>::NoSuchProof
		);
    })
}

#[test]
fn transfer_claim_not_owner() {
    new_test_ext().execute_with(|| {
        let proof = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), proof.clone());

        assert_noop!(
			PoeModule::transfer_claim(Origin::signed(2), proof.clone(), 3),
			Error::<Test>::NotProofOwner
		);
    })
}

#[test]
fn create_claim_fail_when_to_large_hash() {
    new_test_ext().execute_with(|| {
        let proof = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        assert_noop!(
			PoeModule::create_claim(Origin::signed(1), proof.clone()),
			Error::<Test>::ProofTooLarge
		);
    })
}
