use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_create() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		Balances::set_balance(RuntimeOrigin::root(), account_id, 1_000_000_000, 0).unwrap();

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id);

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"abcdabcd"));

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1);
		let kitty = KittiesModule::kitties(kitty_id);
		assert_eq!(kitty.is_some(), true);
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
		assert_eq!(KittiesModule::kitty_parents(kitty_id), None);

		crate::NextKittyId::<Test>::set(crate::KittyId::max_value());
		assert_noop!(
			KittiesModule::create(RuntimeOrigin::signed(account_id), *b"abcdabcd"),
			Error::<Test>::InvalidKittyId
		);

		// Check event
		System::assert_last_event(
			Event::KittyCreated { who: account_id, kitty_id, kitty: kitty.unwrap() }.into(),
		);
	})
}

#[test]
fn it_works_for_breed() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;

		Balances::set_balance(RuntimeOrigin::root(), account_id, 1_000_000_000, 0).unwrap();

		// parents not exist
		assert_noop!(
			KittiesModule::breed(
				RuntimeOrigin::signed(account_id),
				kitty_id,
				kitty_id + 1,
				*b"abcdabcd"
			),
			Error::<Test>::InvalidKittyId
		);

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"abcdabcd"));
		// same kitty id
		assert_noop!(
			KittiesModule::breed(
				RuntimeOrigin::signed(account_id),
				kitty_id,
				kitty_id,
				*b"abcdabcd"
			),
			Error::<Test>::SameKittyId
		);
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"abcdabcd"));
		// it_works_for_create() already tested this
		//assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 2);

		// breed works
		assert_ok!(KittiesModule::breed(
			RuntimeOrigin::signed(account_id),
			kitty_id,
			kitty_id + 1,
			*b"abcdabcd"
		));

		let breed_kitty_id = 2;
		//assert_eq!(KittiesModule::next_kitty_id(), breed_kitty_id + 1);
		let kitty = KittiesModule::kitties(breed_kitty_id);
		assert_eq!(kitty.is_some(), true);
		assert_eq!(KittiesModule::kitty_owner(breed_kitty_id), Some(account_id));
		assert_eq!(KittiesModule::kitty_parents(breed_kitty_id), Some((kitty_id, kitty_id + 1)));

		// Check event
		System::assert_last_event(
			Event::KittyBred { who: account_id, kitty_id: breed_kitty_id, kitty: kitty.unwrap() }
				.into(),
		);
	})
}

#[test]
fn it_works_for_transfer() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		let recipient = 2;

		Balances::set_balance(RuntimeOrigin::root(), account_id, 1_000_000_000, 0).unwrap();

		// kitty not exist
		assert_noop!(
			KittiesModule::transfer(RuntimeOrigin::signed(account_id), recipient, kitty_id),
			Error::<Test>::InvalidKittyId
		);

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"abcdabcd"));
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));

		//not owner
		assert_noop!(
			KittiesModule::transfer(RuntimeOrigin::signed(recipient), account_id, kitty_id),
			Error::<Test>::NotOwner
		);

		assert_ok!(KittiesModule::transfer(RuntimeOrigin::signed(account_id), recipient, kitty_id));
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(recipient));
		// Check event
		System::assert_last_event(
			Event::KittyTransferred { who: account_id, recipient, kitty_id }.into(),
		);

		assert_ok!(KittiesModule::transfer(RuntimeOrigin::signed(recipient), account_id, kitty_id));
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
		// Check event
		System::assert_last_event(
			Event::KittyTransferred { who: recipient, recipient: account_id, kitty_id }.into(),
		);
	})
}

#[test]
fn it_works_for_sale() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		let another_account_id = 2;
		Balances::set_balance(RuntimeOrigin::root(), account_id, 1_000_000_000, 0).unwrap();

		//kitty not exist
		assert_noop!(
			KittiesModule::sale(RuntimeOrigin::signed(another_account_id), kitty_id),
			Error::<Test>::InvalidKittyId
		);

		//create kitty
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"abcdabcd"));
		assert_eq!(KittiesModule::kitty_on_sale(kitty_id).is_some(), false);

		assert_noop!(
			KittiesModule::sale(RuntimeOrigin::signed(another_account_id), kitty_id),
			Error::<Test>::NotOwner
		);

		//success
		assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id));
		assert_eq!(KittiesModule::kitty_on_sale(kitty_id).is_some(), true);
		System::assert_last_event(crate::Event::KittyOnSale { who: account_id, kitty_id }.into());

		assert_noop!(
			KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id),
			Error::<Test>::AlreadyOnSale
		);
	});
}

#[test]
fn it_works_for_buy() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let seller_id = 1;
		let buyer_id = 2;
		Balances::set_balance(RuntimeOrigin::root(), seller_id, 1_000_000_000, 0).unwrap();
		Balances::set_balance(RuntimeOrigin::root(), buyer_id, 1_000_000_000, 0).unwrap();

		//kitty not exist
		assert_noop!(
			KittiesModule::buy(RuntimeOrigin::signed(seller_id), kitty_id),
			Error::<Test>::InvalidKittyId
		);

		//create kitty
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(seller_id), *b"abcdabcd"));
		assert_eq!(KittiesModule::kitty_on_sale(kitty_id).is_some(), false);

		assert_noop!(
			KittiesModule::buy(RuntimeOrigin::signed(seller_id), kitty_id),
			Error::<Test>::AlreadyOwned
		);

		assert_noop!(
			KittiesModule::buy(RuntimeOrigin::signed(buyer_id), kitty_id),
			Error::<Test>::NotOnSale
		);
		
		//success
		assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(seller_id), kitty_id));
		assert_ok!(KittiesModule::buy(RuntimeOrigin::signed(buyer_id), kitty_id));
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(buyer_id));
		assert_eq!(KittiesModule::kitty_on_sale(kitty_id).is_some(), false);
		System::assert_last_event(crate::Event::KittyBought{ who: buyer_id, kitty_id }.into());
	});
		
		
		
		
		
		
		
		
		
	
}
