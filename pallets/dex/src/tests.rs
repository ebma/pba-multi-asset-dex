use frame_support::{assert_noop, assert_ok};
use frame_system::{Config, EventRecord};
use orml_traits::MultiCurrency;
use sp_core::H256;
use sp_runtime::Permill;

use primitives::TokenSymbol;

use crate::{
	mock,
	mock::*,
	traits::{CurrencyPair, Pool},
	Error, PoolOf,
};

pub fn assert_has_event<T, F>(matcher: F)
where
	T: Config,
	F: Fn(&EventRecord<mock::Event, H256>) -> bool,
{
	assert!(System::events().iter().any(matcher));
}

pub fn assert_last_event<T, F>(matcher: F)
where
	T: Config,
	F: FnOnce(&EventRecord<mock::Event, H256>) -> bool,
{
	let events = System::events();
	let last_event = events.last().expect("events expected");
	assert!(matcher(last_event));
}

fn create_default_pool() -> PoolOf<Test> {
	let owner: AccountId = ALICE;
	let first = ASSET_1;
	let second = ASSET_2;
	let pair = CurrencyPair { token_a: first, token_b: second };

	let lp_token = ASSET_3;

	let fee = Permill::from_percent(3);

	Pool { owner, pair, lp_token, fee }
}

#[test]
fn create_pool_should_work() {
	run_test(|| {
		let pool = create_default_pool();
		Dex::create_pool(Origin::signed(ALICE), pool);

		assert_eq!(Dex::pools(0), Some(pool));

		assert_eq!(Dex::pool_count(), 1);

		assert_last_event::<Test, _>(|e| {
			matches!(e.event,
            mock::Event::Dex(crate::Event::PoolCreated { owner, pool_id, assets })
            if owner == ALICE && pool_id == 0 && assets == pool.pair)
		});
	});
}

#[test]
fn add_liquidity_should_work() {
	run_test(|| {
		let pool = create_default_pool();

		assert_ok!(Dex::create_pool(Origin::signed(ALICE), pool));

		let balance_1_pre_deposit = Tokens::free_balance(ASSET_1, &ALICE);
		let balance_2_pre_deposit = Tokens::free_balance(ASSET_2, &ALICE);

		let pool_id = 0;
		let amount = 100;
		let asset = ASSET_1;
		assert_ok!(Dex::add_liquidity(Origin::signed(ALICE), pool_id, amount, asset));

		assert_eq!(balance_1_pre_deposit - amount, Tokens::free_balance(ASSET_1, &ALICE));
		assert_eq!(balance_2_pre_deposit - amount, Tokens::free_balance(ASSET_2, &ALICE));

		// LP of initial deposit will be sqrt(amount_a*amount_b)
		let expected_minted_lp = 100u128;
		// Expect Alice to now have 100 LP tokens
		assert_eq!(Tokens::free_balance(ASSET_3, &ALICE), expected_minted_lp);

		assert_last_event::<Test, _>(|e| {
			matches!(e.event,
            mock::Event::Dex(crate::Event::LiquidityAdded {who, pool_id, amount_a, amount_b, minted_lp})
            if who == ALICE && pool_id == pool_id && amount_a == amount && amount_b == amount && minted_lp == expected_minted_lp)
		});
	});
}

#[test]
fn remove_liquidity_without_supply_should_fail() {
	run_test(|| {
		let pool = create_default_pool();

		assert_ok!(Dex::create_pool(Origin::signed(ALICE), pool));

		let pool_id = 0;
		let amount = 100;
		assert_noop!(
			Dex::remove_liquidity(Origin::signed(ALICE), pool_id, amount),
			Error::<Test>::WithdrawWithoutSupply
		);
	});
}

#[test]
fn add_and_remove_liquidity_should_work() {
	run_test(|| {
		let pool = create_default_pool();

		assert_ok!(Dex::create_pool(Origin::signed(ALICE), pool));

		let balance_1_pre_deposit = Tokens::free_balance(ASSET_1, &ALICE);
		let balance_2_pre_deposit = Tokens::free_balance(ASSET_2, &ALICE);

		let pool_id = 0;
		let amount = 100;
		let asset = ASSET_1;
		assert_ok!(Dex::add_liquidity(Origin::signed(ALICE), pool_id, amount, asset));

		assert_eq!(balance_1_pre_deposit - amount, Tokens::free_balance(ASSET_1, &ALICE));
		assert_eq!(balance_2_pre_deposit - amount, Tokens::free_balance(ASSET_2, &ALICE));

		// LP of initial deposit will be sqrt(amount_a*amount_b)
		let expected_minted_lp = 100u128;
		// Expect Alice to now have 100 LP tokens
		assert_eq!(Tokens::free_balance(ASSET_3, &ALICE), expected_minted_lp);

		assert_last_event::<Test, _>(|e| {
			matches!(e.event,
            mock::Event::Dex(crate::Event::LiquidityAdded {who, pool_id, amount_a, amount_b, minted_lp})
            if who == ALICE && pool_id == pool_id && amount_a == amount && amount_b == amount && minted_lp == expected_minted_lp)
		});

		// Withdraw all LP
		let balance_1_pre_withdraw = Tokens::free_balance(ASSET_1, &ALICE);
		let balance_2_pre_withdraw = Tokens::free_balance(ASSET_2, &ALICE);

		let amount = expected_minted_lp;
		assert_ok!(Dex::remove_liquidity(Origin::signed(ALICE), pool_id, amount));

		// The received amount of the other tokens happens to be the same as the minted LP
		let expected_amount_a = amount;
		let expected_amount_b = amount;
		assert_eq!(
			balance_1_pre_withdraw + expected_amount_a,
			Tokens::free_balance(ASSET_1, &ALICE)
		);
		assert_eq!(
			balance_2_pre_withdraw + expected_amount_b,
			Tokens::free_balance(ASSET_2, &ALICE)
		);

		assert_last_event::<Test, _>(|e| {
			matches!(e.event,
            mock::Event::Dex(crate::Event::LiquidityRemoved {who, pool_id, amount_a, amount_b, total_issuance})
            if who == ALICE && pool_id == pool_id && amount_a == expected_amount_a && amount_b == expected_amount_b && total_issuance == 0)
		});
	});
}

#[test]
pub fn swap_should_work() {
	run_test(|| {
		let pool = create_default_pool();
		assert_ok!(Dex::create_pool(Origin::signed(ALICE), pool));

		// Add liquidity to pool
		let pool_id = 0;
		let amount = 100;
		let asset = ASSET_1;
		assert_ok!(Dex::add_liquidity(Origin::signed(ALICE), pool_id, amount, asset));

		let balance_1_pre_swap = Tokens::free_balance(ASSET_1, &ALICE);
		let balance_2_pre_swap = Tokens::free_balance(ASSET_2, &ALICE);

		let pair = CurrencyPair { token_a: ASSET_1, token_b: ASSET_2 };
		let amount_to_swap = 10;
		assert_ok!(Dex::swap(Origin::signed(ALICE), pool_id, pair, amount_to_swap));

		// Expect to receive 9 tokens of token_a
		let expected_amount_a = 9;
		assert_eq!(
			balance_1_pre_swap + expected_amount_a,
			Tokens::free_balance(ASSET_1, &ALICE)
		);

		// Expect to spend 10 tokens of token_b
		let expected_amount_b = amount_to_swap;
		assert_eq!(
			balance_2_pre_swap - expected_amount_b,
			Tokens::free_balance(ASSET_2, &ALICE)
		);

		assert_last_event::<Test, _>(|e| {
			matches!(e.event,
            mock::Event::Dex(crate::Event::Swapped {who, pool_id, amount_a, amount_b, token_a, token_b, fee})
            if who == ALICE && pool_id == pool_id && amount_a == expected_amount_a && amount_b == expected_amount_b &&
			token_a == pair.token_a && token_b == pair.token_b && fee == pool.fee)
		});
	});
}
