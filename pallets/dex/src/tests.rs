use frame_support::{assert_noop, assert_ok};
use frame_system::{Config, EventRecord};
use orml_traits::MultiCurrency;
use sp_arithmetic::FixedPointNumber;
use sp_core::H256;
use sp_runtime::{FixedU128, Permill};

use primitives::{CurrencyId, TokenSymbol};

use crate::{
	mock,
	mock::*,
	traits::{CurrencyPair, Pool, PoolCreationParams},
	AssetIdOf, Error, PoolCreationParamsOf, PoolOf,
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

fn create_default_pool_params() -> PoolCreationParamsOf<Test> {
	let owner: AccountId = ALICE;
	let first = ASSET_1;
	let second = ASSET_2;
	let pair = CurrencyPair { token_a: first, token_b: second };

	let fee = Permill::from_percent(3);

	PoolCreationParams { owner, pair, fee }
}

/// Default value for deviation of computation error
const DEFAULT_EPSILON: u128 = 5;

fn assert_with_computation_error(expected: u128, value: u128, epsilon: u128) -> Result<(), ()> {
	let lower = expected.saturating_sub(epsilon);
	let upper = expected.saturating_add(epsilon);

	if lower <= value && value <= upper {
		Ok(())
	} else {
		Err(())
	}
}

#[test]
fn create_pool_should_work() {
	run_test(|| {
		let pool_params = create_default_pool_params();
		Dex::create_pool(Origin::signed(ALICE), pool_params);

		let pool: PoolOf<Test> = Dex::pools(0).unwrap();
		assert_eq!(pool.pair, pool_params.pair);
		assert_eq!(pool.owner, pool_params.owner);
		assert_eq!(pool.fee, pool_params.fee);

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
		let pool = create_default_pool_params();

		assert_ok!(Dex::create_pool(Origin::signed(ALICE), pool));

		let pool_id = 0;
		let pool = Dex::pools(pool_id).unwrap();

		let user_balance_1_pre_deposit = Tokens::free_balance(ASSET_1, &ALICE);
		let user_balance_2_pre_deposit = Tokens::free_balance(ASSET_2, &ALICE);

		let pool_account = Dex::pool_accounts(pool_id).unwrap();
		let pool_balance_1_pre_deposit = Tokens::free_balance(ASSET_1, &pool_account);
		let pool_balance_2_pre_deposit = Tokens::free_balance(ASSET_2, &pool_account);

		let amount = 100;
		let asset = ASSET_1;
		// Add liquidity to pool
		assert_ok!(Dex::add_liquidity(Origin::signed(ALICE), pool_id, amount, asset));

		// Expect user balance to be reduced by amount
		assert_eq!(user_balance_1_pre_deposit - amount, Tokens::free_balance(ASSET_1, &ALICE));
		assert_eq!(user_balance_2_pre_deposit - amount, Tokens::free_balance(ASSET_2, &ALICE));
		// Expect pool balance to be increased by amount
		assert_eq!(
			pool_balance_1_pre_deposit + amount,
			Tokens::free_balance(ASSET_1, &pool_account)
		);
		assert_eq!(
			pool_balance_2_pre_deposit + amount,
			Tokens::free_balance(ASSET_2, &pool_account)
		);

		// LP of initial deposit will be sqrt(amount_a*amount_b)
		let expected_minted_lp = 100u128;
		// Expect Alice to now have 100 LP tokens
		assert_eq!(Tokens::free_balance(pool.lp_token, &ALICE), expected_minted_lp);

		assert_last_event::<Test, _>(|e| {
			matches!(e.event,
            mock::Event::Dex(crate::Event::LiquidityAdded {who, pool_id, amount_a, amount_b, minted_lp})
            if who == ALICE && pool_id == pool_id && amount_a == amount && amount_b == amount && minted_lp == expected_minted_lp)
		});
	});
}

#[test]
fn add_liquidity_should_fail_with_invalid_asset() {
	run_test(|| {
		let pool_params = create_default_pool_params();

		assert_ok!(Dex::create_pool(Origin::signed(ALICE), pool_params));
		let pool_id = 0;
		let amount = 100;
		let invalid_asset: AssetIdOf<Test> = CurrencyId::Token(TokenSymbol::Short([u8::MAX; 4]));
		assert_noop!(
			Dex::add_liquidity(Origin::signed(ALICE), pool_id, amount, invalid_asset),
			Error::<Test>::InvalidAsset
		);
	})
}

#[test]
fn add_liquidity_should_fail_without_balance() {
	run_test(|| {
		let mut pool_params = create_default_pool_params();
		// Use asset in pair that the user has no balance for
		let asset_without_balance: AssetIdOf<Test> =
			CurrencyId::Token(TokenSymbol::Short([u8::MAX; 4]));
		pool_params.pair.token_a = asset_without_balance;

		assert_ok!(Dex::create_pool(Origin::signed(ALICE), pool_params));
		let pool_id = 0;
		let amount = 100;
		assert_noop!(
			Dex::add_liquidity(Origin::signed(ALICE), pool_id, amount, asset_without_balance),
			Error::<Test>::InsufficientBalance
		);
	})
}

#[test]
fn remove_liquidity_with_too_high_amount_should_fail() {
	run_test(|| {
		let pool = create_default_pool_params();

		assert_ok!(Dex::create_pool(Origin::signed(ALICE), pool));
		let pool_id = 0;
		let amount = 100;
		let asset = ASSET_1;
		assert_ok!(Dex::add_liquidity(Origin::signed(ALICE), pool_id, amount, asset));

		let pool_id = 0;
		let amount = 10_000;
		assert_noop!(
			Dex::remove_liquidity(Origin::signed(ALICE), pool_id, amount),
			Error::<Test>::InsufficientLiquidityBalance
		);
	});
}
#[test]
fn remove_liquidity_without_supply_should_fail() {
	run_test(|| {
		let pool = create_default_pool_params();

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
		let pool_params = create_default_pool_params();

		assert_ok!(Dex::create_pool(Origin::signed(ALICE), pool_params));
		// pool_id will be 0
		let pool_id = 0;

		let pool: PoolOf<Test> = Dex::pools(pool_id).unwrap();

		let user_balance_1_pre_deposit = Tokens::free_balance(ASSET_1, &ALICE);
		let user_balance_2_pre_deposit = Tokens::free_balance(ASSET_2, &ALICE);

		let pool_account = Dex::pool_accounts(pool_id).unwrap();
		let pool_balance_1_pre_deposit = Tokens::free_balance(ASSET_1, &pool_account);
		let pool_balance_2_pre_deposit = Tokens::free_balance(ASSET_2, &pool_account);

		let amount = 100;
		let asset = ASSET_1;
		assert_ok!(Dex::add_liquidity(Origin::signed(ALICE), pool_id, amount, asset));

		// Expect user balance to be reduced by amount
		assert_eq!(user_balance_1_pre_deposit - amount, Tokens::free_balance(ASSET_1, &ALICE));
		assert_eq!(user_balance_2_pre_deposit - amount, Tokens::free_balance(ASSET_2, &ALICE));
		// Expect pool balance to be increased by amount
		assert_eq!(
			pool_balance_1_pre_deposit + amount,
			Tokens::free_balance(ASSET_1, &pool_account)
		);
		assert_eq!(
			pool_balance_2_pre_deposit + amount,
			Tokens::free_balance(ASSET_2, &pool_account)
		);

		// LP of initial deposit will be sqrt(amount_a*amount_b) -> sqrt(100*100) = 100
		let expected_minted_lp = 100u128;
		// Expect Alice to now have 100 LP tokens
		assert_eq!(Tokens::free_balance(pool.lp_token, &ALICE), expected_minted_lp);

		assert_last_event::<Test, _>(|e| {
			matches!(e.event,
            mock::Event::Dex(crate::Event::LiquidityAdded {who, pool_id, amount_a, amount_b, minted_lp})
            if who == ALICE && pool_id == pool_id && amount_a == amount && amount_b == amount && minted_lp == expected_minted_lp)
		});

		// Withdraw all LP
		let user_balance_1_pre_withdraw = Tokens::free_balance(ASSET_1, &ALICE);
		let user_balance_2_pre_withdraw = Tokens::free_balance(ASSET_2, &ALICE);

		let pool_balance_1_pre_withdraw = Tokens::free_balance(ASSET_1, &pool_account);
		let pool_balance_2_pre_withdraw = Tokens::free_balance(ASSET_2, &pool_account);

		let amount = expected_minted_lp;
		assert_ok!(Dex::remove_liquidity(Origin::signed(ALICE), pool_id, amount));

		// The received amount of the other tokens happens to be the same as the minted LP
		let expected_amount_a = amount;
		let expected_amount_b = amount;
		assert_eq!(
			user_balance_1_pre_withdraw + expected_amount_a,
			Tokens::free_balance(ASSET_1, &ALICE)
		);
		assert_eq!(
			user_balance_2_pre_withdraw + expected_amount_b,
			Tokens::free_balance(ASSET_2, &ALICE)
		);
		// Expect pools balance to be reduced by amount
		assert_eq!(
			pool_balance_1_pre_withdraw - expected_amount_a,
			Tokens::free_balance(ASSET_1, &pool_account)
		);
		assert_eq!(
			pool_balance_2_pre_withdraw - expected_amount_b,
			Tokens::free_balance(ASSET_2, &pool_account)
		);

		assert_last_event::<Test, _>(|e| {
			matches!(e.event,
            mock::Event::Dex(crate::Event::LiquidityRemoved {who, pool_id, amount_a, amount_b, total_issuance})
            if who == ALICE && pool_id == pool_id && amount_a == expected_amount_a && amount_b == expected_amount_b && total_issuance == 0)
		});
	});
}

#[test]
fn sell_should_work() {
	run_test(|| {
		let pool_params = create_default_pool_params();
		assert_ok!(Dex::create_pool(Origin::signed(ALICE), pool_params));

		// Add liquidity to pool
		let pool_id = 0;
		let amount = 100_000;
		let asset = ASSET_1;
		assert_ok!(Dex::add_liquidity(Origin::signed(ALICE), pool_id, amount, asset));

		let balance_1_pre_swap = Tokens::free_balance(ASSET_1, &ALICE);
		let balance_2_pre_swap = Tokens::free_balance(ASSET_2, &ALICE);

		let asset = ASSET_1;
		let amount_to_sell = 100;
		assert_ok!(Dex::sell(Origin::signed(ALICE), pool_id, asset, amount_to_sell));

		// Expect to spend `amount_to_sell` tokens of token_a
		assert_eq!(balance_1_pre_swap - amount_to_sell, Tokens::free_balance(ASSET_1, &ALICE));

		// Expect to receive roughly `amount_to_sell` - fee tokens of token_b
		let amount_to_receive: u128 = amount_to_sell - pool_params.fee.mul_ceil(amount_to_sell);
		assert_ok!(assert_with_computation_error(
			balance_2_pre_swap + amount_to_receive,
			Tokens::free_balance(ASSET_2, &ALICE),
			DEFAULT_EPSILON,
		));

		// Expect the pool to have more of ASSET_1 because the user swapped ASSET_1 for ASSET_2
		let pool_account = Dex::pool_accounts(pool_id).unwrap();
		assert!(
			Tokens::free_balance(ASSET_1, &pool_account) >
				Tokens::free_balance(ASSET_2, &pool_account)
		);

		assert_last_event::<Test, _>(|e| {
			matches!(e.event,
            mock::Event::Dex(crate::Event::Swapped {who, pool_id, amount_a, amount_b, token_a, token_b, fee})
            if who == ALICE && pool_id == pool_id && amount_b == amount_to_sell &&
			token_a == ASSET_2 && token_b == ASSET_1 && fee == pool_params.fee)
		});
	});
}

#[test]
fn buy_should_work() {
	run_test(|| {
		let pool_params = create_default_pool_params();
		assert_ok!(Dex::create_pool(Origin::signed(ALICE), pool_params));

		// Add liquidity to pool
		let pool_id = 0;
		let amount = 100_000;
		let asset = ASSET_1;
		assert_ok!(Dex::add_liquidity(Origin::signed(ALICE), pool_id, amount, asset));

		let balance_1_pre_swap = Tokens::free_balance(ASSET_1, &ALICE);
		let balance_2_pre_swap = Tokens::free_balance(ASSET_2, &ALICE);

		let asset = ASSET_1;
		let amount_to_receive = 100;
		assert_ok!(Dex::buy(Origin::signed(ALICE), pool_id, asset, amount_to_receive));

		// Expect to receive `amount_to_receive` tokens of token_a
		assert_eq!(balance_1_pre_swap + amount_to_receive, Tokens::free_balance(ASSET_1, &ALICE));

		// Expect to spend roughly `amount_to_sell` + fee tokens of token_b
		let amount_to_sell: u128 = amount_to_receive + pool_params.fee.mul_ceil(amount_to_receive);
		assert_ok!(assert_with_computation_error(
			balance_2_pre_swap - amount_to_sell,
			Tokens::free_balance(ASSET_2, &ALICE),
			DEFAULT_EPSILON
		));

		// Expect the pool to have more of ASSET_2 because the user swapped ASSET_2 for ASSET_1
		let pool_account = Dex::pool_accounts(pool_id).unwrap();
		assert!(
			Tokens::free_balance(ASSET_2, &pool_account) >
				Tokens::free_balance(ASSET_1, &pool_account)
		);

		assert_last_event::<Test, _>(|e| {
			matches!(e.event,
            mock::Event::Dex(crate::Event::Swapped {who, pool_id, amount_a, amount_b, token_a, token_b, fee})
            if who == ALICE && pool_id == pool_id && amount_a == amount_to_receive &&
			token_a == ASSET_1 && token_b == ASSET_2 && fee == pool_params.fee)
		});
	});
}
