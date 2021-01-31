use crate::{Error, mock::*};
use frame_support::{
	assert_ok, assert_noop,
	traits::{OnFinalize, OnInitialize},
};
use super::*;

fn run_to_block(n: u64) {
	while System::block_number() < n {
		KittiesModule::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		KittiesModule::on_initialize(System::block_number());
	}
}

#[test]
fn create_works() {
    new_test_ext().execute_with(|| {
		run_to_block(2);
		assert_ok!(KittiesModule::create(Origin::signed(5), 5000));

		let expected_event = TestEvent::kitties_event(RawEvent::Created(5, 0, 5000));
		assert_eq!(
			System::events()[1].event,
			expected_event,
		);

		assert_eq!(KittiesModule::kitties_count(), 1);
		assert_eq!(KittiesModule::kitty_owner(0), Some(5));
		assert_eq!(KittiesModule::owned_kitties(&(5, None)), Some(KittyLinkedItem::<TestRuntime> {
			prev: Some(0),
			next: Some(0),
		}));

		assert_ok!(KittiesModule::create(Origin::signed(5), 5000));
		assert_eq!(KittiesModule::kitties_count(), 2);
		assert_eq!(KittiesModule::kitty_owner(1), Some(5));
		assert_eq!(KittiesModule::owned_kitties(&(5, None)), Some(KittyLinkedItem::<TestRuntime> {
			prev: Some(1),
			next: Some(0),
		}));
		assert_eq!(KittiesModule::owned_kitties(&(5, Some(1))), Some(KittyLinkedItem::<TestRuntime> {
			prev: Some(0),
			next: None,
		}));
	});
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with(|| {
		run_to_block(2);
		assert_ok!(KittiesModule::create(Origin::signed(5), 5000));

		assert_eq!(KittiesModule::kitties_count(), 1);
		assert_eq!(KittiesModule::kitty_owner(0), Some(5));

		assert_ok!(KittiesModule::transfer(Origin::signed(5), 10, 0));

		let expected_event = TestEvent::kitties_event(RawEvent::Transferred(5, 10, 0, 5000));
		assert_eq!(
			System::events()[7].event,
			expected_event,
		);

		assert_eq!(KittiesModule::kitties_count(), 1);
		assert_eq!(KittiesModule::kitty_owner(0), Some(10));
	});
}

#[test]
fn transfer_not_exist() {
    new_test_ext().execute_with(|| {
		run_to_block(2);
		assert_ok!(KittiesModule::create(Origin::signed(5), 5000));

		assert_eq!(KittiesModule::kitties_count(), 1);
		assert_eq!(KittiesModule::kitty_owner(0), Some(5));

		assert_noop!(
			KittiesModule::transfer(Origin::signed(5), 10, 1),
			Error::<TestRuntime>::NotKittyOwner
		);

		assert_eq!(KittiesModule::kitties_count(), 1);
		assert_eq!(KittiesModule::kitty_owner(0), Some(5));
	});
}

#[test]
fn transfer_not_owner() {
    new_test_ext().execute_with(|| {
		run_to_block(2);
		assert_ok!(KittiesModule::create(Origin::signed(5), 5000));

		assert_eq!(KittiesModule::kitties_count(), 1);
		assert_eq!(KittiesModule::kitty_owner(0), Some(5));

		assert_noop!(
			KittiesModule::transfer(Origin::signed(10), 10, 0),
			Error::<TestRuntime>::NotKittyOwner
		);

		assert_eq!(KittiesModule::kitties_count(), 1);
		assert_eq!(KittiesModule::kitty_owner(0), Some(5));
	});
}

#[test]
fn breed_works() {
    new_test_ext().execute_with(|| {
		run_to_block(2);
		assert_ok!(KittiesModule::create(Origin::signed(5), 5000));
		assert_ok!(KittiesModule::create(Origin::signed(5), 5000));

		assert_eq!(KittiesModule::kitties_count(), 2);

		assert_ok!(KittiesModule::breed(Origin::signed(5), 0, 1, 5000));
		assert_eq!(KittiesModule::kitties_count(), 3);
	});
}

#[test]
fn breed_kitty_not_exist() {
    new_test_ext().execute_with(|| {
		run_to_block(2);
		assert_ok!(KittiesModule::create(Origin::signed(5), 5000));
		assert_ok!(KittiesModule::create(Origin::signed(5), 5000));

		assert_eq!(KittiesModule::kitties_count(), 2);

		assert_noop!(
			KittiesModule::breed(Origin::signed(5), 3, 4, 5000),
			Error::<TestRuntime>::InvalidKittyId
		);
		assert_noop!(
			KittiesModule::breed(Origin::signed(5), 0, 4, 5000),
			Error::<TestRuntime>::InvalidKittyId
		);
		assert_noop!(
			KittiesModule::breed(Origin::signed(5), 1, 4, 5000),
			Error::<TestRuntime>::InvalidKittyId
		);
		assert_eq!(KittiesModule::kitties_count(), 2);
	});
}

#[test]
fn breed_kitty_not_owner() {
    new_test_ext().execute_with(|| {
		run_to_block(2);
		assert_ok!(KittiesModule::create(Origin::signed(5), 5000));
		assert_ok!(KittiesModule::create(Origin::signed(5), 5000));

		assert_eq!(KittiesModule::kitties_count(), 2);

		assert_noop!(
			KittiesModule::breed(Origin::signed(10), 0, 1, 5000),
			Error::<TestRuntime>::NotKittyOwner
		);
		assert_eq!(KittiesModule::kitties_count(), 2);
	});
}

#[test]
fn breed_kitty_with_same() {
    new_test_ext().execute_with(|| {
		run_to_block(2);
		assert_ok!(KittiesModule::create(Origin::signed(5), 5000));

		assert_eq!(KittiesModule::kitties_count(), 1);

		assert_noop!(
			KittiesModule::breed(Origin::signed(5), 0, 0, 5000),
			Error::<TestRuntime>::RequireDifferentParent
		);
		assert_eq!(KittiesModule::kitties_count(), 1);
	});
}
