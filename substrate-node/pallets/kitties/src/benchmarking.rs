//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Kitties;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	transfer {
		let caller: T::AccountId = whitelisted_caller();
		let caller_origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));

		Kitties::<T>::create_kitty(caller_origin);

		let dna = KittiesOwned::<T>::get(caller.clone()).get(0).unwrap();
		let receiver: T::AccountId = whitelisted_caller();
	}: transfer (RawOrigin::Signed(caller), receiver, dna.to_vec())
	verify {
		// assert_eq!(KittiesOwned::<T>::get(receiver.clone()).get(0).unwrap(), dna);
	}

	impl_benchmark_test_suite!(Kitties, crate::mock::new_test_ext(), crate::mock::Test);
}
