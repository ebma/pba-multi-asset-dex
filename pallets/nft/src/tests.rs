#![cfg(test)]

use crate::{mock::*, pallet::Error, *};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::BoundedVec;

// This function checks that unique_item ownership is set correctly in storage.
// This will panic if things are not correct.
fn assert_ownership(owner: u64, unique_item_id: ItemIdOf<Test>) {
	// For a unique_item to be owned it should exist.
	let unique_item = UniqueItems::<Test>::get(unique_item_id).unwrap();
	// The unique_item's owner is set correctly.
	assert_eq!(unique_item.owner, owner);

	for (check_owner, owned) in UniqueItemsOwned::<Test>::iter() {
		if owner == check_owner {
			// Owner should have this unique_item.
			assert!(owned.contains(&unique_item_id));
		} else {
			// Everyone else should not.
			assert!(!owned.contains(&unique_item_id));
		}
	}
}

#[test]
fn should_build_genesis_unique_items() {
	new_test_ext(vec![
		(1, *b"1234567890123456", b"test".to_vec()),
		(2, *b"123456789012345a", b"othertest".to_vec()),
	])
	.execute_with(|| {
		// Check we have 2 unique_items, as specified in genesis
		assert_eq!(CountForUniqueItems::<Test>::get(), 2);

		// Check owners own the correct amount of unique_items
		let unique_items_owned_by_1 = UniqueItemsOwned::<Test>::get(1);
		assert_eq!(unique_items_owned_by_1.len(), 1);

		let unique_items_owned_by_2 = UniqueItemsOwned::<Test>::get(2);
		assert_eq!(unique_items_owned_by_2.len(), 1);

		// Check that unique_items are owned by the correct owners
		let unique_item_1 = unique_items_owned_by_1[0];
		assert_ownership(1, unique_item_1);

		let unique_item_2 = unique_items_owned_by_2[0];
		assert_ownership(2, unique_item_2);
	});
}

#[test]
fn create_unique_item_should_work() {
	new_test_ext(vec![]).execute_with(|| {
		// Create a unique_item with account #10
		let unique_item_id = *b"1234567890123456";
		let data = BoundedVec::truncate_from(b"test".to_vec());
		assert_ok!(Nfts::create_unique_item(Origin::signed(10), unique_item_id, data.clone()));

		// Check that now 3 unique_items exists
		assert_eq!(CountForUniqueItems::<Test>::get(), 1);

		// Check that account #10 owns 1 unique_item
		let unique_items_owned = UniqueItemsOwned::<Test>::get(10);
		assert_eq!(unique_items_owned.len(), 1);
		let id = unique_items_owned.last().unwrap();
		assert_ownership(10, *id);

		// Check that multiple create_unique_item calls work in the same block.
		// Increment extrinsic index to add entropy for DNA
		frame_system::Pallet::<Test>::set_extrinsic_index(1);
		let other_unique_item_id = *b"1234567890123450";
		assert_ok!(Nfts::create_unique_item(
			Origin::signed(10),
			other_unique_item_id,
			data.clone()
		));
	});
}

#[test]
fn create_unique_item_fails() {
	// Check that create_unique_item fails when user owns too many unique_items.
	new_test_ext(vec![]).execute_with(|| {
		// Data used for all items
		let data = BoundedVec::truncate_from(b"test".to_vec());

		// Create `MaxUniqueItemsOwned` unique_items with account #10
		for _i in 0..<Test as Config>::MaxUniqueItemsOwned::get() {
			let unique_item_id: [u8; 16] = [_i as u8; 16];
			assert_ok!(Nfts::create_unique_item(Origin::signed(10), unique_item_id, data.clone()));
			// We do this because the hash of the unique_item depends on this for seed,
			// so changing this allows you to have a different unique_item id
			System::set_block_number(System::block_number() + 1);
		}

		// Can't create 1 more
		let unique_item_id: [u8; 16] = [u8::MAX; 16];
		assert_noop!(
			Nfts::create_unique_item(Origin::signed(10), unique_item_id, data.clone()),
			Error::<Test>::TooManyOwned
		);

		// Minting a unique_item with DNA that already exists should fail
		let id = [u8::MAX; 16];

		// Mint new unique_item with `id`
		assert_ok!(Nfts::mint(&1, id, data.clone()));

		// Mint another unique_item with the same `id` should fail
		assert_noop!(Nfts::mint(&1, id, data.clone()), Error::<Test>::DuplicateUniqueItem);
	});
}

#[test]
fn transfer_unique_item_should_work() {
	new_test_ext(vec![]).execute_with(|| {
		// Account 10 creates a unique_item
		let unique_item_id: [u8; 16] = [0u8; 16];
		let data = BoundedVec::truncate_from(b"test".to_vec());
		assert_ok!(Nfts::create_unique_item(Origin::signed(10), unique_item_id, data));
		let id = UniqueItemsOwned::<Test>::get(10)[0];

		// and sends it to account 3
		assert_ok!(Nfts::transfer(Origin::signed(10), 3, id));

		// Check that account 10 now has nothing
		assert_eq!(UniqueItemsOwned::<Test>::get(10).len(), 0);

		// but account 3 does
		assert_eq!(UniqueItemsOwned::<Test>::get(3).len(), 1);
		assert_ownership(3, id);
	});
}

#[test]
fn transfer_unique_item_should_fail() {
	new_test_ext(vec![
		(1, *b"1234567890123456", b"test".to_vec()),
		(2, *b"123456789012345a", b"othertest".to_vec()),
	])
	.execute_with(|| {
		// Get the DNA of some unique_item
		let id = UniqueItemsOwned::<Test>::get(1)[0];

		// Account 9 cannot transfer a unique_item with this DNA.
		assert_noop!(Nfts::transfer(Origin::signed(9), 2, id), Error::<Test>::NotOwner);

		// Check transfer fails when transferring to self
		assert_noop!(Nfts::transfer(Origin::signed(1), 1, id), Error::<Test>::TransferToSelf);

		// Check transfer fails when no unique_item exists
		let random_id = [0u8; 16];

		assert_noop!(Nfts::transfer(Origin::signed(2), 1, random_id), Error::<Test>::NoUniqueItem);

		// Check that transfer fails when max unique_item is reached
		// Create `MaxUniqueItemsOwned` unique_items for account #10
		for _i in 0..<Test as Config>::MaxUniqueItemsOwned::get() {
			let unique_item_id: [u8; 16] = [_i as u8; 16];
			let data = BoundedVec::truncate_from(b"test".to_vec());
			assert_ok!(Nfts::create_unique_item(Origin::signed(10), unique_item_id, data));
			System::set_block_number(System::block_number() + 1);
		}

		// Account #10 should not be able to receive a new unique_item
		assert_noop!(Nfts::transfer(Origin::signed(1), 10, id), Error::<Test>::TooManyOwned);
	});
}

#[test]
fn buy_unique_item_works() {
	new_test_ext(vec![
		(1, *b"1234567890123456", b"test".to_vec()),
		(2, *b"123456789012345a", b"test".to_vec()),
		(3, *b"1234567890123451", b"test".to_vec()),
	])
	.execute_with(|| {
		// Check buy_unique_item works as expected
		let id = UniqueItemsOwned::<Test>::get(2)[0];
		let set_price: PriceOf<Test> = (4, ASSET_1);
		let balance_1_before = Tokens::free_balance(ASSET_1, &1);
		let balance_2_before = Tokens::free_balance(ASSET_1, &2);

		// Account #2 sets a price of 4 for their unique_item
		assert_ok!(Nfts::set_price(Origin::signed(2), id, Some(set_price)));

		// Account #1 can buy account #2's unique_item, specifying some limit_price
		let limit_price: PriceOf<Test> = (6, ASSET_1);
		assert_ok!(Nfts::buy_unique_item(Origin::signed(1), id, limit_price));

		// Check balance transfer works as expected
		let balance_1_after = Tokens::free_balance(ASSET_1, &1);
		let balance_2_after = Tokens::free_balance(ASSET_1, &2);

		// We use set_price as this is the amount actually being charged
		assert_eq!(balance_1_before - set_price.0, balance_1_after);
		assert_eq!(balance_2_before + set_price.0, balance_2_after);

		// Now this unique_item is not for sale, even from an account who can afford it
		assert_noop!(
			Nfts::buy_unique_item(Origin::signed(3), id, set_price),
			Error::<Test>::NotForSale
		);
	});
}

#[test]
fn buy_unique_item_fails() {
	new_test_ext(vec![
		(1, *b"1234567890123456", b"test".to_vec()),
		(2, *b"123456789012345a", b"test".to_vec()),
		(10, *b"1234567890123410", b"test".to_vec()),
	])
	.execute_with(|| {
		// Check buy_unique_item fails when unique_item is not for sale
		let id = UniqueItemsOwned::<Test>::get(1)[0];
		// UniqueItem is not for sale
		let price: PriceOf<Test> = (2, ASSET_1);
		assert_noop!(
			Nfts::buy_unique_item(Origin::signed(2), id, price),
			Error::<Test>::NotForSale
		);

		// Check buy_unique_item fails when bid price is too low
		// New price is set to 4
		let id = UniqueItemsOwned::<Test>::get(2)[0];
		let set_price: PriceOf<Test> = (4, ASSET_1);
		assert_ok!(Nfts::set_price(Origin::signed(2), id, Some(set_price)));

		// Account #10 can't buy this unique_item for half the asking price
		assert_noop!(
			Nfts::buy_unique_item(Origin::signed(10), id, (set_price.0 / 2, set_price.1)),
			Error::<Test>::BidPriceTooLow
		);

		// Check buy_unique_item fails when balance is too low
		// Get the balance of account 10
		let balance_of_account_10 = Tokens::free_balance(ASSET_1, &10);

		// Reset the price to something higher than account 10's balance
		assert_ok!(Nfts::set_price(
			Origin::signed(2),
			id,
			Some((balance_of_account_10 * 10, ASSET_1))
		));

		// Account 10 can't buy a unique_item they can't afford
		assert_noop!(
			Nfts::buy_unique_item(Origin::signed(10), id, (balance_of_account_10 * 10, ASSET_1)),
			orml_tokens::Error::<Test>::BalanceTooLow
		);
	});
}

#[test]
fn set_price_works() {
	new_test_ext(vec![
		(1, *b"1234567890123456", b"test".to_vec()),
		(2, *b"123456789012345a", b"test".to_vec()),
	])
	.execute_with(|| {
		// Check set_price works as expected
		let id = UniqueItemsOwned::<Test>::get(2)[0];
		let set_price: PriceOf<Test> = (4, ASSET_1);
		assert_ok!(Nfts::set_price(Origin::signed(2), id, Some(set_price)));

		// Only owner can set price
		assert_noop!(
			Nfts::set_price(Origin::signed(1), id, Some(set_price)),
			Error::<Test>::NotOwner
		);

		// UniqueItem must exist too
		let non_dna = [2u8; 16];
		assert_noop!(
			Nfts::set_price(Origin::signed(1), non_dna, Some(set_price)),
			Error::<Test>::NoUniqueItem
		);
	});
}
