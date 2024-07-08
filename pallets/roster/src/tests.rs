use crate::{mock::*, types, Error, Event, Rosters};
use frame_support::{assert_noop, assert_ok, pallet_prelude::*};
use sp_runtime::AccountId32;

// https://docs.substrate.io/test/

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([2u8; 32]);

// Test RosterCalls
#[test]
fn test_can_create_new_roster() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

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
