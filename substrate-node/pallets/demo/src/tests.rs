use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(DemoModule::create_student(Origin::signed(1), "daitran".as_bytes().to_vec(), 25));
		// Read pallet storage and assert an expected result.
		assert_eq!(DemoModule::student_id(), 1);
	});
}

#[test]
fn it_does_not_work_with_invalid_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when creating a student with age less than 18.
		assert_noop!(DemoModule::create_student(Origin::signed(1), "daitran".as_bytes().to_vec(), 15), Error::<Test>::TooYoung);
	});
}
