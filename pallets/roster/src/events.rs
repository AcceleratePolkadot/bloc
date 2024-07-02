use frame_support::pallet_macros::*;

#[pallet_section]
mod events {
    use crate::RosterStatus;

	#[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// New roster created [created by, roster title, roster id]
        NewRoster(T::AccountId, RosterTitle<T>, RosterId),
        /// Roster status has been changed [changed by, roster id, new status]
        RosterStatusChanged(T::AccountId, RosterId, RosterStatus),
        /// Roster has been removed [removed by, roster id]
        RosterRemoved(T::AccountId, RosterId),
        /// New nomination [nominator, nominee, roster id]
        NewNomination(T::AccountId, T::AccountId, RosterId),
        /// Nomination period has ended [nominator, nominee, closed by, nomination status]
        NominationClosed(T::AccountId, RosterId, T::AccountId, NominationStatus),
        /// Vote added to nomination [voter, vote value, nominee, roster id]
        Voted(T::AccountId, NominationVoteValue, T::AccountId, RosterId),
        /// Vote recanted [voter, nominee, roster id]
        VoteRecanted(T::AccountId, T::AccountId, RosterId),
        /// New member added to roster [member, roster id]
        MemberAdded(T::AccountId, RosterId),
        /// Member removed from roster [member, roster id]
        MemberRemoved(T::AccountId, RosterId),
        /// New expulsion proposal [motioner, subject, roster id, reason]
        NewExpulsionProposal(T::AccountId, T::AccountId, RosterId, ExpulsionReason<T>),
        /// Expulsion Proposal has been seconded [seconder, motioner, subject, roster id, number of seconds]
        SeconderAddedToExpulsionProposal(T::AccountId, T::AccountId, T::AccountId, RosterId, u32),
        /// Expulsion proposal opened for voting [motioner, subject, roster id]
        ExpulsionVoteOpened(T::AccountId, T::AccountId, RosterId),
        /// Vote added to expulsion proposal [voter, motioner, subject, roster id, vote value]
        ExpulsionVoteSubmitted(T::AccountId, T::AccountId, T::AccountId, RosterId, ExpulsionProposalVoteValue),
        /// Vote added to expulsion proposal [voter, motioner, subject, roster id]
        ExpulsionVoteRecanted(T::AccountId, T::AccountId, T::AccountId, RosterId),
        /// Expulsion proposal has been dismissed with prejudice [closer, motioner, subject, roster id]
        ExpulsionProposalDismissedWithPrejudice(T::AccountId, T::AccountId, T::AccountId, RosterId),
        /// Expulsion proposal has been dismissed [closer, motioner, subject, roster id]
        ExpulsionProposalDismissed(T::AccountId, T::AccountId, T::AccountId, RosterId),
        /// Expulsion proposal has passed [closer, motioner, subject, roster id]
        ExpulsionProposalPassed(T::AccountId, T::AccountId, T::AccountId, RosterId),
    }
}
