#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/build/custom-pallets/>
pub use pallet::*;

use codec::Encode;
use frame_support::{pallet_macros::*, traits::Get};
use sp_std::{vec, vec::Vec};

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
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		traits::{Currency, NamedReservableCurrency},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

	/// The Treasury account
	#[pallet::storage]
	pub type Treasury<T: Config> = StorageValue<_, T::AccountId>;

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

impl<T: Config> Pallet<T> {
	/// The account ID of the treasury pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn account_id() -> Option<T::AccountId> {
		Treasury::<T>::get()
	}

	pub fn reserved_currency_name(reason: ReservedCurrencyReason<T>) -> [u8; 27] {
		let prefixes: ReservedCurrencyNamePrefixes = ReservedCurrencyNamePrefixes {
			new_roster: vec![0, 0, 1],
			new_nomination: vec![0, 0, 2],
			membership_dues: vec![0, 0, 3],
			new_expulsion_proposal: vec![0, 0, 4],
		};
		let mut name = Vec::new();
		match reason {
			ReservedCurrencyReason::NewRoster(roster_id) => {
				name.extend(prefixes.new_roster);
				name.extend(T::PalletId::get().0.to_vec());
				name.extend(roster_id.0.to_vec());
			},
			ReservedCurrencyReason::NewNomination(roster_id, nominee) => {
				name.extend(prefixes.new_nomination);
				name.extend(T::PalletId::get().0.to_vec());
				name.extend(roster_id.0.to_vec().iter().take(8));
				let nominee_account_id: Vec<u8> = nominee.encode();
				name.extend(nominee_account_id.iter().take(8));
			},
			ReservedCurrencyReason::MembershipDues(roster_id) => {
				name.extend(prefixes.membership_dues);
				name.extend(T::PalletId::get().0.to_vec());
				name.extend(roster_id.0.to_vec());
			},
			ReservedCurrencyReason::NewExpulsionProposal(roster_id, subject) => {
				name.extend(prefixes.new_expulsion_proposal);
				name.extend(T::PalletId::get().0.to_vec());
				name.extend(roster_id.0.to_vec().iter().take(8));
				let subject_account_id: Vec<u8> = subject.encode();
				name.extend(subject_account_id.iter().take(8));
			},
		};
		match name.try_into() {
			Ok(name) => name,
			Err(_) => [0u8; 27],
		}
	}
}
