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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

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

    // Keys are the founder's account id and the roster's title, so titles must be unique per founder.
    #[pallet::storage]
    pub type Rosters<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        RosterId,
        Roster<T>,
    >;

    #[pallet::storage]
    pub type Nominations<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        RosterId,
        Nomination<T>,
    >;

}
