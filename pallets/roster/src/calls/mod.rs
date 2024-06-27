use frame_support::pallet_macros::*;

pub mod rosters;
pub mod nominations;

#[pallet_section]
mod calls {
    use sp_std::vec::Vec;
    use crate::calls::{nominations::NominationCalls, rosters::RosterCalls};

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new roster
        ///
        /// New rosters are created as active
		///
		/// The dispatch origin of this call must be _Signed_
        /// The origin will be made the founder of the roster and added as its first member
		///
		/// - `title`: A vector of bytes representing the string title of the roster (must be smaller than `TitleMaxLength`)
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
        pub fn roster_activate(origin: OriginFor<T>,  roster_id: RosterId) -> DispatchResultWithPostInfo {
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
        #[pallet::call_index(12)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn roster_deactivate(origin: OriginFor<T>,  roster_id: RosterId) -> DispatchResultWithPostInfo {
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
        pub fn roster_remove(origin: OriginFor<T>,  roster_id: RosterId) -> DispatchResultWithPostInfo {
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
        pub fn nomination_new(origin: OriginFor<T>, roster_id: RosterId, nominee: T::AccountId) -> DispatchResultWithPostInfo {
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
        pub fn nomination_vote(origin: OriginFor<T>, roster_id: RosterId, nominee: T::AccountId, vote: NominationVoteValue) -> DispatchResultWithPostInfo {
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
        pub fn nomination_vote_recant(origin: OriginFor<T>, roster_id: RosterId, nominee: T::AccountId) -> DispatchResultWithPostInfo {
            let voter = ensure_signed(origin)?;
            NominationCalls::<T>::recant_vote(voter, roster_id, nominee)
        }

        /// Close a nomination and the voting period has ended or the voting threshold has been reached
		///
		/// The dispatch origin of this call must be _Signed_
        /// Anyone can close a nomination, the account does not need to be a roster member.
        /// This ensures nominees can close their own nominations
        ///
        /// If the vote passes nominees will be added to the roster members list
		///
		/// - `roster_id`: The UUID for the roster
        /// - `nominee`: AccountId of the account being nominated
		///
		/// Emits `NominationClosed`
        #[pallet::call_index(33)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn nomination_close(origin: OriginFor<T>, roster_id: RosterId, nominee: T::AccountId) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            NominationCalls::<T>::close(who, roster_id, nominee)
        }
    }
}