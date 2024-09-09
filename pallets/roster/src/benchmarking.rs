// Only enable this module for benchmarking.
#![cfg(feature = "runtime-benchmarks")]

use crate::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

// Details on using the benchmarks macro can be seen at:
//   https://paritytech.github.io/substrate/master/frame_benchmarking/trait.Benchmarking.html#tymethod.benchmarks
#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn roster_new() {
		let caller = whitelisted_caller();
		let roster_id = types::RosterId::from_tuple_with_unbounded_title::<T>((
			&caller,
			&"My Roster".as_bytes().to_vec(),
		));

		#[extrinsic_call]
		roster_new(RawOrigin::Signed(caller.clone()), "My Roster".as_bytes().to_vec());

		let roster = Rosters::<T>::get(&roster_id);

		assert!(roster.is_some());
		assert_eq!(roster.unwrap().founder, caller)
	}

	// This line generates test cases for benchmarking, and could be run by:
	//   `cargo test -p pallet-example-basic --all-features`, you will see one line per case:
	//   `test benchmarking::bench_sort_vector ... ok`
	//   `test benchmarking::bench_accumulate_dummy ... ok`
	//   `test benchmarking::bench_set_dummy_benchmark ... ok` in the result.
	//
	// The line generates three steps per benchmark, with repeat=1 and the three steps are
	//   [low, mid, high] of the range.
	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
