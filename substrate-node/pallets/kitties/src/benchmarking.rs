//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Kitties;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	transfer {
		let caller: T::AccountId = whitelisted_caller();

		let _ = Kitties::<T>::create_kitty(RawOrigin::Signed(caller.clone()).into());

		let owner = KittiesOwned::<T>::get(caller.clone());
		let dna = owner.get(0).unwrap();
		let receiver: T::AccountId = account("receiver", 0, 0);
	}: transfer (RawOrigin::Signed(caller), receiver.clone(), dna.to_vec())
	verify {
		// assert!(KittiesOwned::<T>::get(&receiver).is_some());
	}

	impl_benchmark_test_suite!(Kitties, crate::mock::new_test_ext(), crate::mock::Test);
}
