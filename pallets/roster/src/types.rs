use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{sp_runtime::RuntimeDebug, BoundedVec};
use frame_system::{self as system, pallet_prelude::BlockNumberFor, Config};
use scale_info::TypeInfo;

use crate::pallet;

pub type RosterTitle<T> = BoundedVec<u8, <T as pallet::Config>::TitleMaxLength>;

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum RosterStatus {
    Active,
    Inactive,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Roster<T: Config + pallet::Config> {
    pub founder: T::AccountId,
    pub title: RosterTitle<T>,
    pub members: BoundedVec<T::AccountId, <T>::MembersMax>,
    pub founded_on: BlockNumberFor<T>,
    pub status: RosterStatus,
}

impl<T: Config + pallet::Config> Roster<T> {
    pub fn new(
        founder: T::AccountId,
        title: BoundedVec<u8, <T>::TitleMaxLength>,
    ) -> Self {
        Self {
            founder,
            title,
            members: BoundedVec::default(),
            founded_on: <system::Pallet<T>>::block_number(),
            status: RosterStatus::Active,
        }
    }
}