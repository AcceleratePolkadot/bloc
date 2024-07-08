#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/build/custom-pallets/>
pub use pallet::*;

use frame_support::pallet_macros::*;

mod calls;
mod config;
mod errors;
mod events;
mod hooks;
pub mod types;

use crate::types::*;

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
#[cfg(test)]
mod mock;

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
#[cfg(test)]
mod tests;

// Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// for each dispatchable and generates this pallet's weight.rs file.
// Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[import_section(hooks::hooks)]
#[import_section(events::events)]
#[import_section(errors::errors)]
#[import_section(config::config)]
#[import_section(calls::calls)]
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// A map of all Rosters
	/// The `RosterId` is a v8 UUID generated from the md5 hash of the founder AccountId and the
	/// Roster title See `RosterId::from_tuple()` for more details on how this key is generated
	#[pallet::storage]
	pub type Rosters<T: Config> = StorageMap<_, Blake2_128Concat, RosterId, Roster<T>>;

	/// Open roster membership nominations
	/// Concluded nominations (approved or rejected) are removed from storage on block
	/// initialization
	///
	/// Storage keys are the nominee's AccountId and the RosterId they are being nominated to
	#[pallet::storage]
	pub type Nominations<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		RosterId,
		Nomination<T>,
	>;

	/// List of concluded nominations
	///
	/// When a nomination is closed it is added to this list. This saves us from having to iterate
	/// over all nominations during block initialization to find those which are no longer
	/// `NominationStatus::Pending`
	///
	/// This storage value is taken and removed during each block initialization
	#[pallet::storage]
	pub type ConcludedNominations<T: Config> = StorageValue<
		_,
		BoundedVec<(T::AccountId, RosterId), T::ConcludedNominationsMax>,
		ValueQuery,
	>;

	/// Expulsion proposals
	///
	/// Storage keys are:
	///  - the id of the roster the proposal is to expel the subject from
	///  - the motioner's accountId
	///  - the subject's accountId
	#[pallet::storage]
	pub type ExpulsionProposals<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, RosterId>,
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Blake2_128Concat, T::AccountId>,
		),
		ExpulsionProposal<T>,
	>;
}
