use md5;
use uuid::Uuid;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{sp_runtime::RuntimeDebug, BoundedVec};
use frame_system::{self as system, pallet_prelude::BlockNumberFor, Config};
use scale_info::TypeInfo;

use crate::pallet;

pub type RosterTitle<T> = BoundedVec<u8, <T as pallet::Config>::TitleMaxLength>;
pub type MembersList<T> = BoundedVec<<T as Config>::AccountId, <T as pallet::Config>::MembersMax>;
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct RosterId([u8; 16]);

impl RosterId {
    pub fn from_tuple<T: Config + pallet::Config>((founder, title): (T::AccountId, RosterTitle<T>)) -> Self {
        let mut bytes = founder.encode();
        bytes.extend(title);
        let digest = md5::compute(bytes);
        
        RosterId(*Uuid::new_v8(digest.into()).as_bytes())
    }

}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum RosterStatus {
    Active,
    Inactive,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Roster<T: Config + pallet::Config> {
    pub id: RosterId,
    pub founder: T::AccountId,
    pub title: RosterTitle<T>,
    pub members: MembersList<T>,
    pub founded_on: BlockNumberFor<T>,
    pub status: RosterStatus,
}

impl<T: Config + pallet::Config> Roster<T> {
    pub fn new(
        founder: T::AccountId,
        title: RosterTitle<T>,
    ) -> Self {
        Self {
            id: RosterId::from_tuple::<T>((founder.clone(), title.clone())),
            founder,
            title,
            members: BoundedVec::default(),
            founded_on: <system::Pallet<T>>::block_number(),
            status: RosterStatus::Active,
        }
    }
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum NominationStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Nomination<T: Config + pallet::Config> {
    pub roster: (T::AccountId, RosterTitle<T>),
    pub nominee: T::AccountId,
    pub nominator: T::AccountId,
    pub nominated_on: BlockNumberFor<T>,
    pub status: NominationStatus,
}

impl<T: Config + pallet::Config> Nomination<T> {
    pub fn new(
        roster: (T::AccountId, RosterTitle<T>),
        nominee: T::AccountId,
        nominator: T::AccountId
    ) -> Self {
        Self {
            roster,
            nominee,
            nominator,
            nominated_on: <system::Pallet<T>>::block_number(),
            status: NominationStatus::Pending,
        }
    }
}