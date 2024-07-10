use frame_support::pallet_macros::*;

pub mod expulsions;
pub mod nominations;
pub mod rosters;

#[pallet_section]
mod calls {
	use crate::calls::{
		expulsions::ExpulsionCalls, nominations::NominationCalls, rosters::RosterCalls,
	};
	use sp_std::vec::Vec;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new roster
		///
		/// New rosters are created as active
		///
		/// The dispatch origin of this call must be _Signed_
		/// The origin will be made the founder of the roster and added as its first member
		///
		/// - `title`: A vector of bytes representing the string title of the roster (must be
		///   smaller than `TitleMaxLength`)
		///
		/// Emits `NewRoster`
		#[pallet::call_index(10)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn roster_new(origin: OriginFor<T>, title: Vec<u8>) -> DispatchResultWithPostInfo {
			let founder = ensure_signed(origin)?;
			RosterCalls::<T>::new(founder, title)
		}

		/// Activate a Roster
		///
		/// The dispatch origin of this call must be _Signed_
		/// The origin must be the roster founder
		///
		/// - `roster_id`: The UUID for the roster to activate
		///
		/// Emits `RosterStatusChanged`
		#[pallet::call_index(11)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn roster_activate(
			origin: OriginFor<T>,
			roster_id: RosterId,
		) -> DispatchResultWithPostInfo {
			let founder = ensure_signed(origin)?;
			RosterCalls::<T>::activate(founder, roster_id)
		}

		/// Deactivate a Roster
		///
		/// The dispatch origin of this call must be _Signed_
		/// The origin must be the roster founder
		///
		/// - `roster_id`: The UUID for the roster to deactivate
		///
		/// Emits `RosterStatusChanged`
		/// Emits `NominationClosed`
		/// Emits `ExpulsionProposalDismissed`
		#[pallet::call_index(12)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn roster_deactivate(
			origin: OriginFor<T>,
			roster_id: RosterId,
		) -> DispatchResultWithPostInfo {
			let founder = ensure_signed(origin)?;
			RosterCalls::<T>::deactivate(founder, roster_id)
		}

		/// Remove a Roster
		///
		/// The dispatch origin of this call must be _Signed_
		/// The origin must be the roster founder
		///
		/// Only deactivated rosters can be removed
		///
		/// - `roster_id`: The UUID for the roster to remove
		///
		/// Emits `RosterRemoved`
		#[pallet::call_index(13)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn roster_remove(
			origin: OriginFor<T>,
			roster_id: RosterId,
		) -> DispatchResultWithPostInfo {
			let founder = ensure_signed(origin)?;
			RosterCalls::<T>::remove(founder, roster_id)
		}

		/// Nominate an account to join a roster
		///
		/// The dispatch origin of this call must be _Signed_
		/// The origin will be made the nominator
		///
		/// - `roster_id`: The UUID for the roster the account is being nominated to join
		/// - `nominee`: AccountId of the account being nominated
		///
		/// Emits `NewNomination`
		#[pallet::call_index(30)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn nomination_new(
			origin: OriginFor<T>,
			roster_id: RosterId,
			nominee: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let nominator = ensure_signed(origin)?;
			NominationCalls::<T>::new(nominator, roster_id, nominee)
		}

		/// Vote on a nomination
		///
		/// The dispatch origin of this call must be _Signed_
		/// The origin must already be a member of the roster the nomination is for
		///
		/// - `roster_id`: The UUID for the roster
		/// - `nominee`: AccountId of the account being nominated
		/// - `vote`: The `NominationVoteValue` (Aye or Nay)
		///
		/// Emits `Voted`
		#[pallet::call_index(31)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn nomination_vote(
			origin: OriginFor<T>,
			roster_id: RosterId,
			nominee: T::AccountId,
			vote: NominationVoteValue,
		) -> DispatchResultWithPostInfo {
			let voter = ensure_signed(origin)?;
			NominationCalls::<T>::vote(voter, roster_id, nominee, vote)
		}

		/// Recant (withdraw) a vote on a nomination
		///
		/// The dispatch origin of this call must be _Signed_
		/// The origin can only recant votes they have submitted
		///
		/// - `roster_id`: The UUID for the roster
		/// - `nominee`: AccountId of the account being nominated
		///
		/// Emits `VoteRecanted`
		#[pallet::call_index(32)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn nomination_recant_vote(
			origin: OriginFor<T>,
			roster_id: RosterId,
			nominee: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let voter = ensure_signed(origin)?;
			NominationCalls::<T>::recant_vote(voter, roster_id, nominee)
		}

		/// Close a nomination
		///
		/// The dispatch origin of this call must be _Signed_
		/// Anyone can close a nomination, the account does not need to be a roster member.
		/// This ensures nominees can close their own nominations
		///
		/// Nominations can only be closed if the voting period has ended or the vote threshold
		/// has been reached
		///
		/// If the vote passes nominees will be added to the roster members list
		///
		/// - `roster_id`: The UUID for the roster
		/// - `nominee`: AccountId of the account being nominated
		///
		/// Emits `NominationClosed`
		#[pallet::call_index(33)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn nomination_close(
			origin: OriginFor<T>,
			roster_id: RosterId,
			nominee: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			NominationCalls::<T>::close(who, roster_id, nominee)
		}

		/// Add a member to a roster
		///
		/// The dispatch origin of this call must be _Signed_
		/// The origin of the call must be the nominee
		/// The origin will pay the membership dues
		///
		/// Members can only be added if the nomination has been approved
		///
		/// - `roster_id`: The UUID for the roster
		/// - `nominee`: AccountId of the account being nominated
		///
		/// Emits `MemberAdded`
		#[pallet::call_index(34)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn add_member(origin: OriginFor<T>, roster_id: RosterId) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			NominationCalls::<T>::add_member(who, roster_id)
		}

		/// Submit an expulsion proposal
		///
		/// The dispatch origin of this call must be _Signed_
		/// The origin account will be the motioner and may be subject to punishment if the proposal
		/// is unsuccessful
		///
		/// - `subject`: AccountId of the member facing expulsion
		/// - `roster_id`: The UUID of the roster the subject could be expelled from
		/// - `reason`: A vector of bytes of the string describing the reason the subject should be
		///   expelled (must be smaller than `ExpulsionReasonMaxLength`)
		///
		/// Emits `NewExpulsionProposal`
		#[pallet::call_index(50)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn expulsion_proposal_new(
			origin: OriginFor<T>,
			subject: T::AccountId,
			roster_id: RosterId,
			reason: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let motioner = ensure_signed(origin)?;
			ExpulsionCalls::<T>::new(motioner, subject, roster_id, reason)
		}

		/// Second an expulsion proposal
		///
		/// The dispatch origin of this call must be _Signed_
		/// The origin account will be added as a seconder to the motioner's expulsion vote
		/// Seconders may be subject to punishment if the vote is unsuccessful
		///
		/// - `motioner`: AccountId of the member who started the expulsion vote
		/// - `subject`: AccountId of the member facing expulsion
		/// - `roster_id`: The UUID of the roster the subject could be expelled from
		///
		/// Emits `SeconderAddedToExpulsionProposal`
		#[pallet::call_index(51)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn expulsion_proposal_second(
			origin: OriginFor<T>,
			motioner: T::AccountId,
			subject: T::AccountId,
			roster_id: RosterId,
		) -> DispatchResultWithPostInfo {
			let seconder = ensure_signed(origin)?;
			ExpulsionCalls::<T>::second(seconder, motioner, subject, roster_id)
		}

		/// Open an expulsion vote
		///
		/// The dispatch origin of this call must be _Signed_
		/// The origin account must be the moitioner of the expulsion vote
		///
		/// A proposal can only be moved to the Voting stage if the number of seconders is greater
		/// than or equal to the `ExpulsionProposalSecondThreshold`
		///
		/// - `subject`: AccountId of the member facing expulsion
		/// - `roster_id`: The UUID of the roster the subject could be expelled from
		///
		/// Emits `ExpulsionVoteOpened`
		#[pallet::call_index(52)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn expulsion_vote_open(
			origin: OriginFor<T>,
			subject: T::AccountId,
			roster_id: RosterId,
		) -> DispatchResultWithPostInfo {
			let motioner = ensure_signed(origin)?;
			ExpulsionCalls::<T>::open(motioner, subject, roster_id)
		}

		/// Vote on an expulsion vote
		///
		/// The dispatch origin of this call must be _Signed_
		/// The origin account will be recorded as the voter, they cannot have voted on this
		/// proposal before
		///
		/// Vote values cannot be changed via this extrinsic, to change a vote first recant it and
		/// then vote again
		///
		/// - `motioner`: AccountId of the member who started the expulsion vote
		/// - `subject`: AccountId of the member facing expulsion
		/// - `roster_id`: The UUID of the roster the subject could be expelled from
		/// - `vote`: The `ExpulsionProposalVoteValue` (Aye, Nay, or Abstain)
		///
		/// Emits `ExpulsionVoteSubmitted`
		#[pallet::call_index(53)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn expulsion_vote_submit_vote(
			origin: OriginFor<T>,
			motioner: T::AccountId,
			subject: T::AccountId,
			roster_id: RosterId,
			vote: ExpulsionProposalVoteValue,
		) -> DispatchResultWithPostInfo {
			let voter = ensure_signed(origin)?;
			ExpulsionCalls::<T>::vote(voter, motioner, subject, roster_id, vote)
		}

		/// Recant vote on an expulsion vote
		///
		/// The dispatch origin of this call must be _Signed_
		/// The origin account must be the voter of the vote they wish to recant
		///
		/// - `motioner`: AccountId of the member who started the expulsion vote
		/// - `subject`: AccountId of the member facing expulsion
		/// - `roster_id`: The UUID of the roster the subject could be expelled from
		///
		/// Emits `ExpulsionVoteRecanted`
		#[pallet::call_index(54)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn expulsion_vote_recant_vote(
			origin: OriginFor<T>,
			motioner: T::AccountId,
			subject: T::AccountId,
			roster_id: RosterId,
		) -> DispatchResultWithPostInfo {
			let voter = ensure_signed(origin)?;
			ExpulsionCalls::<T>::recant_vote(voter, motioner, subject, roster_id)
		}

		/// Close an expulsion proposal
		///
		/// The dispatch origin of this call must be _Signed_
		/// The origin will be the closer of the proposal, and can be any member of the roster
		///
		/// If conditions are met the proposal may be dismissed with prejudice
		///
		/// - `motioner`: AccountId of the member who started the expulsion vote
		/// - `subject`: AccountId of the member facing expulsion
		/// - `roster_id`: The UUID of the roster the subject could be expelled from
		///
		/// Emits `ExpulsionProposalPassed`
		/// Emits `ExpulsionProposalDismissed`
		/// Emits `ExpulsionProposalDismissedWithPrejudice`
		#[pallet::call_index(55)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn expulsion_proposal_close(
			origin: OriginFor<T>,
			motioner: T::AccountId,
			subject: T::AccountId,
			roster_id: RosterId,
		) -> DispatchResultWithPostInfo {
			let closer = ensure_signed(origin)?;
			ExpulsionCalls::<T>::close(closer, motioner, subject, roster_id)
		}
	}
}
