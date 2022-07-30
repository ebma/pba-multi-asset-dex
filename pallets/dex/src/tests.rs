use frame_support::{assert_noop, assert_ok};
use sp_runtime::Permill;

use primitives::TokenSymbol;

use crate::{
	mock::*,
	pool::{CurrencyPair, Pool},
	Error,
};

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		// assert_noop!(DexModule::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
	});
}

fn create_default_pool() -> Pool<AccountId, AssetId> {
	let owner: AccountId = ALICE;
	let first = ASSET_1;
	let second = ASSET_2;
	let pair = CurrencyPair { first, second };

	let lp_token = ASSET_3;

	let fee: Permill = Permill::from_percent(3);

	Pool { owner, pair, lp_token, fee }
}

#[test]
fn create_pool_should_work() {
	run_test(|| {
		let pool = create_default_pool();
		DexModule::create_pool(Origin::signed(ALICE), pool);

		assert_eq!(DexModule::pools(0), Some(pool));

		assert_eq!(DexModule::pool_count(), 1)
	});
}

#[test]
fn remove_pool_should_work() {
	run_test(|| {
		let pool = create_default_pool();
		DexModule::create_pool(Origin::signed(ALICE), pool);

		assert_eq!(DexModule::pools(0), Some(pool));

		assert_eq!(DexModule::pool_count(), 1)
	});
}

#[test]
fn add_liquidity_should_work() {
	run_test(|| {
		let pool = create_default_pool();

		DexModule::create_pool(Origin::signed(ALICE), pool);
		DexModule::add_liquidity(Origin::signed(ALICE), 0, 100, 100);
		println!("{:?}", DexModule::pools(0));
	});
}
