use md5;
use uuid::Uuid;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{sp_runtime::RuntimeDebug, BoundedVec};
use frame_system::{self as system, pallet_prelude::BlockNumberFor, Config};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use crate::pallet;


pub type RosterTitle<T> = BoundedVec<u8, <T as pallet::Config>::TitleMaxLength>;
pub type MembersList<T> = BoundedVec<<T as Config>::AccountId, <T as pallet::Config>::MembersMax>;
pub type NominationsList<T> = BoundedVec<<T as Config>::AccountId, <T as pallet::Config>::NominationsPerRosterMax>;
pub type ExpulsionProposalsList<T> = BoundedVec<(<T as Config>::AccountId, <T as Config>::AccountId), <T as pallet::Config>::ExpulsionProposalsPerRosterMax>;
pub type SecondsList<T> = BoundedVec<<T as Config>::AccountId, <T as pallet::Config>::SecondsMax>;
pub type ExpulsionReason<T> = BoundedVec<u8, <T as pallet::Config>::ExpulsionReasonMaxLength>;

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct RosterId([u8; 16]);

impl RosterId {
    pub fn from_tuple<T: Config + pallet::Config>((founder, title): (&T::AccountId, &RosterTitle<T>)) -> Self {
        let namespace = Uuid::from_bytes(md5::compute(founder.encode()).0);
        let roster_uuid = Uuid::new_v3(&namespace, title);
        RosterId(*roster_uuid.as_bytes())
    }

    pub fn from_tuple_with_unbounded_title<T: Config>((founder, title): (&T::AccountId, &Vec<u8>)) -> Self {
        let namespace = Uuid::from_bytes(md5::compute(founder.encode()).0);
        let roster_uuid = Uuid::new_v3(&namespace, &title);
        RosterId(*roster_uuid.as_bytes())
    }
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum RosterStatus {
    Active,
    Inactive,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Roster<T: pallet::Config> {
    pub id: RosterId,
    pub founder: T::AccountId,
    pub title: RosterTitle<T>,
    pub members: MembersList<T>,
    pub nominations: NominationsList<T>,
    pub expulsion_proposals: ExpulsionProposalsList<T>,
    pub founded_on: BlockNumberFor<T>,
    pub status: RosterStatus,
}

impl<T: pallet::Config> Roster<T> {
    pub fn new(
        founder: &T::AccountId,
        title: &RosterTitle<T>,
    ) -> Self {
        Self {
            id: RosterId::from_tuple::<T>((&founder, &title)),
            founder: founder.clone(),
            title: title.clone(),
            members: BoundedVec::default(),
            nominations:  BoundedVec::default(),
            expulsion_proposals: BoundedVec::default(),
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
    pub roster: RosterId,
    pub nominee: T::AccountId,
    pub nominator: T::AccountId,
    pub nominated_on: BlockNumberFor<T>,
    pub votes: NominationVotes<T>,
    pub status: NominationStatus,
}

impl<T: Config + pallet::Config> Nomination<T> {
    pub fn new(
        roster: &RosterId,
        nominee: &T::AccountId,
        nominator: &T::AccountId
    ) -> Self {
        Self {
            roster: roster.clone(),
            nominee: nominee.clone(),
            nominator: nominator.clone(),
            nominated_on: <system::Pallet<T>>::block_number(),
            votes: BoundedVec::default(),
            status: NominationStatus::Pending,
        }
    }
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct NominationVote<T: Config> {
    pub voter: T::AccountId,
    pub vote: NominationVoteValue,
    pub voted_on: BlockNumberFor<T>,
}

impl<T: Config> NominationVote<T> {
    pub fn new(
        voter: &T::AccountId,
        vote: &NominationVoteValue
    ) -> Self {
        Self {
            voter: voter.clone(),
            vote: vote.clone(),
            voted_on: <system::Pallet<T>>::block_number(),
        }
    }
}

pub type NominationVotes<T> = BoundedVec<NominationVote<T>, <T as pallet::Config>::NominationVotesMax>;


#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum NominationVoteValue {
    Aye,
    Nay,
}


#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ExpulsionProposalStatus {
    Proposed,
    Seconded,
    Voting,
    Passed,
    Dismissed,
    DismissedWithPrejudice,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct ExpulsionProposal<T: Config + pallet::Config> {
    pub motioner: T::AccountId,
    pub seconds: SecondsList<T>,
    pub subject: T::AccountId,
    pub roster: RosterId,
    pub reason: ExpulsionReason<T>,
    pub proposed_on: BlockNumberFor<T>,
    pub voting_opened_on: Option<BlockNumberFor<T>>,
    pub decided_on: Option<BlockNumberFor<T>>,
    pub votes: ExpulsionProposalVotes<T>,
    pub status: ExpulsionProposalStatus,
}

impl<T: Config + pallet::Config> ExpulsionProposal<T> {
    pub fn new(
        motioner: &T::AccountId,
        subject: &T::AccountId,
        roster: &RosterId,
        reason: &ExpulsionReason<T>,
    ) -> Self {
        Self {
            motioner: motioner.clone(),
            seconds:  BoundedVec::default(),
            subject: subject.clone(),
            roster: roster.clone(),
            reason: reason.clone(),
            proposed_on: <system::Pallet<T>>::block_number(),
            voting_opened_on: None,
            decided_on: None,
            votes: BoundedVec::default(),
            status: ExpulsionProposalStatus::Proposed,
        }
    }
}


#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct ExpulsionProposalVote<T: Config> {
    pub voter: T::AccountId,
    pub vote: ExpulsionProposalVoteValue,
    pub voted_on: BlockNumberFor<T>,
}

impl<T: Config + pallet::Config> ExpulsionProposalVote<T> {
    pub fn new(
        voter: &T::AccountId,
        vote: &ExpulsionProposalVoteValue
    ) -> Self {
        Self {
            voter: voter.clone(),
            vote: vote.clone(),
            voted_on: <system::Pallet<T>>::block_number(),
        }
    }
}

pub type ExpulsionProposalVotes<T> = BoundedVec<ExpulsionProposalVote<T>, <T as pallet::Config>::ExpulsionProposalVotesMax>;


#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ExpulsionProposalVoteValue {
    Aye,
    Nay,
    Abstain,
}