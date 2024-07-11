use crate::{mock::*, pallet, types, Error, Event, Rosters};
use frame_support::{assert_noop, assert_ok, pallet_prelude::*, traits::Currency};
use sp_runtime::{AccountId32, Percent};

// https://docs.substrate.io/test/

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([2u8; 32]);
pub const CHARLIE: AccountId32 = AccountId32::new([3u8; 32]);

// Test RosterCalls
#[test]
fn test_can_create_new_roster() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&ALICE,
			<Test as pallet::Config>::NewRosterDeposit::get() + 100,
		);

		let roster_title = "My Roster".as_bytes().to_vec();
		let bounded_title_result: Result<types::RosterTitle<Test>, Error<Test>> =
			roster_title.try_into().map_err(|_| Error::<Test>::InvalidRosterTitle);
		assert!(bounded_title_result.is_ok());
		let bounded_title = bounded_title_result.unwrap();
		let roster_id = types::RosterId::from_tuple::<Test>((&ALICE, &bounded_title.clone()));

		// Create Roster
		assert_ok!(Roster::roster_new(
			RuntimeOrigin::signed(ALICE),
			"My Roster".as_bytes().to_vec()
		));
		System::assert_last_event(
			Event::NewRoster {
				founder: ALICE,
				title: bounded_title.clone(),
				roster_id: roster_id.clone(),
			}
			.into(),
		);

		// Ensure new roster has been stored correctly
		let query_response = Rosters::<Test>::get(&roster_id);
		assert!(query_response.is_some());
		let roster = query_response.unwrap();
		assert_eq!(roster.id, roster_id);
		assert_eq!(roster.founder, ALICE);
		assert_eq!(roster.title, bounded_title);
		assert_eq!(roster.status, types::RosterStatus::Active);
		assert_eq!(roster.founded_on, 1);

		// Should start empty
		assert_eq!(roster.nominations.len(), 0);
		assert_eq!(roster.expulsion_proposals.len(), 0);

		// Founder should automatically be added as a member
		assert_eq!(roster.members.len(), 1);
		assert_eq!(roster.members[0], ALICE);
	});
}

#[test]
fn test_roster_title_unique_to_account() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&ALICE,
			<Test as pallet::Config>::NewRosterDeposit::get() * 2 + 100,
		);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&BOB,
			<Test as pallet::Config>::NewRosterDeposit::get() + 100,
		);

		let title = "My Roster".as_bytes().to_vec();
		let bounded_title: BoundedVec<_, _> = title.clone().try_into().unwrap();
		let alice_roster_id = types::RosterId::from_tuple::<Test>((&ALICE, &bounded_title.clone()));
		let bob_roster_id = types::RosterId::from_tuple::<Test>((&BOB, &bounded_title.clone()));

		// Alice creates a roster
		assert_ok!(Roster::roster_new(RuntimeOrigin::signed(ALICE), title.clone()));
		System::assert_last_event(
			Event::NewRoster {
				founder: ALICE,
				title: bounded_title.clone(),
				roster_id: alice_roster_id,
			}
			.into(),
		);

		// Bob creates a roster with the same title
		assert_ok!(Roster::roster_new(RuntimeOrigin::signed(BOB), title.clone()));
		System::assert_last_event(
			Event::NewRoster {
				founder: BOB,
				title: bounded_title.clone(),
				roster_id: bob_roster_id,
			}
			.into(),
		);

		// Alice cannot create a roster with the same title
		assert_noop!(
			Roster::roster_new(RuntimeOrigin::signed(ALICE), title.clone()),
			Error::<Test>::RosterExists
		);
	});
}

#[test]
fn test_toggle_roster_status() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&ALICE,
			<Test as pallet::Config>::NewRosterDeposit::get() + 100,
		);

		let title = "My Roster".as_bytes().to_vec();
		let bounded_title: BoundedVec<_, _> = title.clone().try_into().unwrap();
		let roster_id = types::RosterId::from_tuple::<Test>((&ALICE, &bounded_title.clone()));

		// Create roster
		assert_ok!(Roster::roster_new(RuntimeOrigin::signed(ALICE), title.clone()));
		System::assert_last_event(
			Event::NewRoster {
				founder: ALICE,
				title: bounded_title.clone(),
				roster_id: roster_id.clone(),
			}
			.into(),
		);

		// Roster should be active
		let mut roster = Rosters::<Test>::get(&roster_id).unwrap();
		assert_eq!(roster.status, types::RosterStatus::Active);

		// Set roster status to inactive
		assert_ok!(Roster::roster_deactivate(RuntimeOrigin::signed(ALICE), roster_id.clone()));
		System::assert_has_event(
			Event::RosterStatusChanged {
				changed_by: ALICE,
				roster_id: roster_id.clone(),
				new_status: types::RosterStatus::Inactive,
			}
			.into(),
		);

		run_to_block(2);

		// Roster should be inactive
		roster = Rosters::<Test>::get(&roster_id).unwrap();
		assert_eq!(roster.status, types::RosterStatus::Inactive);

		// Ensure we can toggle it back to active
		assert_ok!(Roster::roster_activate(RuntimeOrigin::signed(ALICE), roster_id.clone()));
		System::assert_has_event(
			Event::RosterStatusChanged {
				changed_by: ALICE,
				roster_id: roster_id.clone(),
				new_status: types::RosterStatus::Active,
			}
			.into(),
		);

		run_to_block(3);

		roster = Rosters::<Test>::get(&roster_id).unwrap();
		assert_eq!(roster.status, types::RosterStatus::Active);
	});
}

#[test]
fn test_cannot_activate_active_roster() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&ALICE,
			<Test as pallet::Config>::NewRosterDeposit::get() + 100,
		);

		let title = "My Roster".as_bytes().to_vec();
		let bounded_title: BoundedVec<_, _> = title.clone().try_into().unwrap();
		let roster_id = types::RosterId::from_tuple::<Test>((&ALICE, &bounded_title.clone()));

		// Create roster
		assert_ok!(Roster::roster_new(RuntimeOrigin::signed(ALICE), title.clone()));
		System::assert_last_event(
			Event::NewRoster {
				founder: ALICE,
				title: bounded_title.clone(),
				roster_id: roster_id.clone(),
			}
			.into(),
		);

		// Roster should be active
		let roster = Rosters::<Test>::get(&roster_id).unwrap();
		assert_eq!(roster.status, types::RosterStatus::Active);

		run_to_block(2);

		// Attempt to activate roster
		assert_noop!(
			Roster::roster_activate(RuntimeOrigin::signed(ALICE), roster_id.clone()),
			Error::<Test>::RosterActive
		);
	});
}

#[test]
fn test_cannot_deactivate_inactive_roster() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&ALICE,
			<Test as pallet::Config>::NewRosterDeposit::get() + 100,
		);

		let title = "My Roster".as_bytes().to_vec();
		let bounded_title: BoundedVec<_, _> = title.clone().try_into().unwrap();
		let roster_id = types::RosterId::from_tuple::<Test>((&ALICE, &bounded_title.clone()));

		// Create roster
		assert_ok!(Roster::roster_new(RuntimeOrigin::signed(ALICE), title.clone()));
		System::assert_last_event(
			Event::NewRoster {
				founder: ALICE,
				title: bounded_title.clone(),
				roster_id: roster_id.clone(),
			}
			.into(),
		);

		// Roster should be active
		let roster = Rosters::<Test>::get(&roster_id).unwrap();
		assert_eq!(roster.status, types::RosterStatus::Active);

		run_to_block(2);

		// Deactivate roster
		assert_ok!(Roster::roster_deactivate(RuntimeOrigin::signed(ALICE), roster_id.clone()));
		System::assert_has_event(
			Event::RosterStatusChanged {
				changed_by: ALICE,
				roster_id: roster_id.clone(),
				new_status: types::RosterStatus::Inactive,
			}
			.into(),
		);

		run_to_block(3);

		// Attempt to activate roster
		assert_noop!(
			Roster::roster_deactivate(RuntimeOrigin::signed(ALICE), roster_id.clone()),
			Error::<Test>::RosterNotActive
		);
	});
}

#[test]
fn test_conclude_nominations_when_roster_deactivated() {
	new_test_ext().execute_with(|| {
        System::set_block_number(1);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&ALICE,
			<Test as pallet::Config>::NewRosterDeposit::get() + <Test as pallet::Config>::NewNominationDeposit::get() + 100,
		);

        let title = "My Roster".as_bytes().to_vec();
        let bounded_title: BoundedVec<_, _> = title.clone().try_into().unwrap();
        let roster_id = types::RosterId::from_tuple::<Test>((&ALICE, &bounded_title.clone()));

        // Create roster
        assert_ok!(Roster::roster_new(RuntimeOrigin::signed(ALICE), title.clone()));
        System::assert_last_event(Event::NewRoster { founder: ALICE, title: bounded_title.clone(), roster_id: roster_id.clone() }.into());

        // Create nomination
        assert_ok!(Roster::nomination_new(RuntimeOrigin::signed(ALICE), roster_id.clone(), BOB));
        System::assert_last_event(Event::NewNomination { nominator: ALICE, nominee: BOB, roster_id: roster_id.clone() }.into());

        run_to_block(2);

        // Deactivate the roster and check the events
        assert_ok!(Roster::roster_deactivate(RuntimeOrigin::signed(ALICE), roster_id.clone()));

        // Check that the correct events were emitted
        assert_eq!(
            System::events().len(),
            2,
            "Expected 2 events to be emitted: 1 for nomination dismissal and 1 for roster deactivation"
        );

        System::assert_has_event(Event::NominationClosed {
            nominee: BOB,
            closed_by: ALICE,
            roster_id: roster_id.clone(),
            status: types::NominationStatus::Rejected
        }.into());

        System::assert_has_event(Event::RosterStatusChanged {
            changed_by: ALICE,
            roster_id: roster_id.clone(),
            new_status: types::RosterStatus::Inactive
        }.into());
    });
}

#[test]
fn test_cannot_remove_active_roster() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&ALICE,
			<Test as pallet::Config>::NewRosterDeposit::get() + 100,
		);

		let title = "My Roster".as_bytes().to_vec();
		let bounded_title: BoundedVec<_, _> = title.clone().try_into().unwrap();
		let roster_id = types::RosterId::from_tuple::<Test>((&ALICE, &bounded_title.clone()));

		// Create roster
		assert_ok!(Roster::roster_new(RuntimeOrigin::signed(ALICE), title.clone()));
		System::assert_last_event(
			Event::NewRoster {
				founder: ALICE,
				title: bounded_title.clone(),
				roster_id: roster_id.clone(),
			}
			.into(),
		);

		run_to_block(2);

		// Attempt to remove roster
		assert_noop!(
			Roster::roster_remove(RuntimeOrigin::signed(ALICE), roster_id.clone()),
			Error::<Test>::RosterActive
		);
	});
}

#[test]
fn test_can_remove_inactive_roster() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&ALICE,
			<Test as pallet::Config>::NewRosterDeposit::get() + 100,
		);

		let title = "My Roster".as_bytes().to_vec();
		let bounded_title: BoundedVec<_, _> = title.clone().try_into().unwrap();
		let roster_id = types::RosterId::from_tuple::<Test>((&ALICE, &bounded_title.clone()));

		// Create roster
		assert_ok!(Roster::roster_new(RuntimeOrigin::signed(ALICE), title.clone()));
		System::assert_last_event(
			Event::NewRoster {
				founder: ALICE,
				title: bounded_title.clone(),
				roster_id: roster_id.clone(),
			}
			.into(),
		);

		run_to_block(2);

		// Deactivate roster
		assert_ok!(Roster::roster_deactivate(RuntimeOrigin::signed(ALICE), roster_id.clone()));
		System::assert_has_event(
			Event::RosterStatusChanged {
				changed_by: ALICE,
				roster_id: roster_id.clone(),
				new_status: types::RosterStatus::Inactive,
			}
			.into(),
		);

		// Remove roster
		assert_ok!(Roster::roster_remove(RuntimeOrigin::signed(ALICE), roster_id.clone()));
		System::assert_has_event(
			Event::RosterRemoved { removed_by: ALICE, roster_id: roster_id.clone() }.into(),
		);
	});
}

#[test]
fn test_only_founder_can_deactivate_roster() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&ALICE,
			<Test as pallet::Config>::NewRosterDeposit::get() + 100,
		);

		let title = "My Roster".as_bytes().to_vec();
		let bounded_title: BoundedVec<_, _> = title.clone().try_into().unwrap();
		let roster_id = types::RosterId::from_tuple::<Test>((&ALICE, &bounded_title.clone()));

		// Create roster
		assert_ok!(Roster::roster_new(RuntimeOrigin::signed(ALICE), title.clone()));
		System::assert_last_event(
			Event::NewRoster {
				founder: ALICE,
				title: bounded_title.clone(),
				roster_id: roster_id.clone(),
			}
			.into(),
		);

		run_to_block(2);

		assert_noop!(
			Roster::roster_deactivate(RuntimeOrigin::signed(BOB), roster_id.clone()),
			Error::<Test>::PermissionDenied
		);
	});
}

#[test]
fn test_only_founder_can_activate_roster() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&ALICE,
			<Test as pallet::Config>::NewRosterDeposit::get() + 100,
		);

		let title = "My Roster".as_bytes().to_vec();
		let bounded_title: BoundedVec<_, _> = title.clone().try_into().unwrap();
		let roster_id = types::RosterId::from_tuple::<Test>((&ALICE, &bounded_title.clone()));

		// Create roster
		assert_ok!(Roster::roster_new(RuntimeOrigin::signed(ALICE), title.clone()));
		System::assert_last_event(
			Event::NewRoster {
				founder: ALICE,
				title: bounded_title.clone(),
				roster_id: roster_id.clone(),
			}
			.into(),
		);

		run_to_block(2);

		// Deactivate roster
		assert_ok!(Roster::roster_deactivate(RuntimeOrigin::signed(ALICE), roster_id.clone()));
		System::assert_has_event(
			Event::RosterStatusChanged {
				changed_by: ALICE,
				roster_id: roster_id.clone(),
				new_status: types::RosterStatus::Inactive,
			}
			.into(),
		);

		run_to_block(3);

		assert_noop!(
			Roster::roster_activate(RuntimeOrigin::signed(BOB), roster_id.clone()),
			Error::<Test>::PermissionDenied
		);
	});
}

#[test]
fn test_only_founder_can_remove_roster() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&ALICE,
			<Test as pallet::Config>::NewRosterDeposit::get() + 100,
		);

		let title = "My Roster".as_bytes().to_vec();
		let bounded_title: BoundedVec<_, _> = title.clone().try_into().unwrap();
		let roster_id = types::RosterId::from_tuple::<Test>((&ALICE, &bounded_title.clone()));

		// Create roster
		assert_ok!(Roster::roster_new(RuntimeOrigin::signed(ALICE), title.clone()));
		System::assert_last_event(
			Event::NewRoster {
				founder: ALICE,
				title: bounded_title.clone(),
				roster_id: roster_id.clone(),
			}
			.into(),
		);

		run_to_block(2);

		// Deactivate roster
		assert_ok!(Roster::roster_deactivate(RuntimeOrigin::signed(ALICE), roster_id.clone()));
		System::assert_has_event(
			Event::RosterStatusChanged {
				changed_by: ALICE,
				roster_id: roster_id.clone(),
				new_status: types::RosterStatus::Inactive,
			}
			.into(),
		);

		run_to_block(3);

		assert_noop!(
			Roster::roster_remove(RuntimeOrigin::signed(BOB), roster_id.clone()),
			Error::<Test>::PermissionDenied
		);
	});
}

#[test]
fn test_deposit_reserved_on_new_roster() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&ALICE,
			<Test as pallet::Config>::NewRosterDeposit::get() + 100,
		);

		let title = "My Roster".as_bytes().to_vec();

		assert_ok!(Roster::roster_new(RuntimeOrigin::signed(ALICE), title.clone()));
		assert_eq!(System::account(&ALICE).data.reserved, 10000000000);
		assert_eq!(System::account(&ALICE).data.free, 100);
	});
}

#[test]
fn test_deposit_returned_when_roster_removed() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&ALICE,
			<Test as pallet::Config>::NewRosterDeposit::get() + 100,
		);

		let title = "My Roster".as_bytes().to_vec();
		let bounded_title: BoundedVec<_, _> = title.clone().try_into().unwrap();
		let roster_id = types::RosterId::from_tuple::<Test>((&ALICE, &bounded_title.clone()));

		assert_ok!(Roster::roster_new(RuntimeOrigin::signed(ALICE), title.clone()));
		assert_eq!(System::account(&ALICE).data.reserved, 10000000000);
		assert_eq!(System::account(&ALICE).data.free, 100);

		run_to_block(2);

		assert_ok!(Roster::roster_deactivate(RuntimeOrigin::signed(ALICE), roster_id.clone()));

		run_to_block(3);

		assert_ok!(Roster::roster_remove(RuntimeOrigin::signed(ALICE), roster_id.clone()));
		assert_eq!(System::account(&ALICE).data.reserved, 0);
		assert_eq!(System::account(&ALICE).data.free, 10000000100);
	});
}

#[test]
fn test_deposit_slashed_when_expulsion_proposal_dismissed_with_prejudice() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let pot: AccountId32 = pallet::Pallet::<Test>::account_id();

		let roster_deposit = <Test as pallet::Config>::NewRosterDeposit::get();
		let nomination_deposit = <Test as pallet::Config>::NewNominationDeposit::get();
		let membership_dues = <Test as pallet::Config>::MembershipDues::get();
		let expulsion_proposal_deposit =
			<Test as pallet::Config>::NewExpulsionProposalDeposit::get();
		let overage = 100;
		let reparations = Percent::from_percent(
			<Test as pallet::Config>::ExpulsionProposalReparations::get()
				.try_into()
				.unwrap(),
		) * expulsion_proposal_deposit;

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&ALICE,
			roster_deposit + nomination_deposit * 2 + overage,
		);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&BOB,
			membership_dues + expulsion_proposal_deposit + overage,
		);

		<Test as pallet::Config>::Currency::make_free_balance_be(
			&CHARLIE,
			membership_dues + overage,
		);

		<Test as pallet::Config>::Currency::make_free_balance_be(&pot, overage);

		let title = "My Roster".as_bytes().to_vec();
		let bounded_title: BoundedVec<_, _> = title.clone().try_into().unwrap();
		let roster_id = types::RosterId::from_tuple::<Test>((&ALICE, &bounded_title.clone()));

		assert_ok!(Roster::roster_new(RuntimeOrigin::signed(ALICE), title.clone()));
		assert_eq!(System::account(&ALICE).data.reserved, roster_deposit);

		run_to_block(2);

		assert_ok!(Roster::nomination_new(RuntimeOrigin::signed(ALICE), roster_id.clone(), BOB));
		assert_ok!(Roster::nomination_new(
			RuntimeOrigin::signed(ALICE),
			roster_id.clone(),
			CHARLIE
		));
		assert_eq!(System::account(&ALICE).data.reserved, roster_deposit + nomination_deposit * 2);

		run_to_block(3);

		assert_ok!(Roster::nomination_vote(
			RuntimeOrigin::signed(ALICE),
			roster_id.clone(),
			BOB,
			types::NominationVoteValue::Aye
		));

		assert_ok!(Roster::nomination_close(RuntimeOrigin::signed(ALICE), roster_id.clone(), BOB));
		assert_ok!(Roster::add_member(RuntimeOrigin::signed(BOB), roster_id.clone()));

		assert_eq!(System::account(&ALICE).data.reserved, roster_deposit + nomination_deposit);
		assert_eq!(System::account(&BOB).data.reserved, membership_dues);

		run_to_block(3);

		assert_ok!(Roster::nomination_vote(
			RuntimeOrigin::signed(ALICE),
			roster_id.clone(),
			CHARLIE,
			types::NominationVoteValue::Aye
		));

		assert_ok!(Roster::nomination_vote(
			RuntimeOrigin::signed(BOB),
			roster_id.clone(),
			CHARLIE,
			types::NominationVoteValue::Aye
		));

		assert_ok!(Roster::nomination_close(
			RuntimeOrigin::signed(BOB),
			roster_id.clone(),
			CHARLIE
		));
		assert_ok!(Roster::add_member(RuntimeOrigin::signed(CHARLIE), roster_id.clone()));

		assert_eq!(System::account(&ALICE).data.reserved, roster_deposit);
		assert_eq!(System::account(&CHARLIE).data.reserved, membership_dues);
		assert_eq!(System::account(&CHARLIE).data.free, overage);

		run_to_block(4);

		let reason = "Two is a party. Three is a crowd".as_bytes().to_vec();

		assert_ok!(Roster::expulsion_proposal_new(
			RuntimeOrigin::signed(BOB),
			CHARLIE,
			roster_id.clone(),
			reason.clone()
		));
		System::assert_has_event(
			Event::NewExpulsionProposal {
				motioner: BOB,
				subject: CHARLIE,
				roster_id: roster_id.clone(),
				reason: reason.try_into().unwrap(),
			}
			.into(),
		);

		assert_eq!(
			System::account(&BOB).data.reserved,
			membership_dues + expulsion_proposal_deposit
		);
		assert_eq!(System::account(&BOB).data.free, overage);
		assert_eq!(System::account(&pot).data.free, overage);

		run_to_block(
			<Test as pallet::Config>::ExpulsionProposalAwaitingSecondPeriod::get() as u64 + 10,
		);

		assert_ok!(Roster::expulsion_proposal_close(
			RuntimeOrigin::signed(CHARLIE),
			BOB,
			CHARLIE,
			roster_id.clone()
		));
		System::assert_has_event(
			Event::ExpulsionProposalDismissedWithPrejudice {
				closer: CHARLIE,
				motioner: BOB,
				subject: CHARLIE,
				roster_id,
			}
			.into(),
		);

		assert_eq!(System::account(&BOB).data.reserved, membership_dues);
		assert_eq!(System::account(&BOB).data.free, overage);
		assert_eq!(System::account(&CHARLIE).data.free, overage + reparations);
		assert_eq!(System::account(&pot).data.free, overage + reparations);
	});
}
