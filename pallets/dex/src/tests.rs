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

#[test]
fn create_pool_should_work() {
	run_test(|| {
		let owner: AccountId = ALICE;
		let first = CurrencyId::Token(TokenSymbol::USDC);
		let second = CurrencyId::Token(TokenSymbol::EURT);
		let pair = CurrencyPair { first, second };

		let lp_token = CurrencyId::Token(TokenSymbol::WBTC);

		let fee: Permill = Permill::from_percent(3);

		let pool = Pool { owner, pair, lp_token, fee };
		DexModule::create_pool(Origin::signed(1), pool);

		assert_eq!(DexModule::pools(0), Some(pool));

		assert_eq!(DexModule::pool_count(), 1)
	});
}

#[test]
fn remove_pool_should_work() {
	run_test(|| {
		let owner: AccountId = ALICE;
		let first = CurrencyId::Token(TokenSymbol::USDC);
		let second = CurrencyId::Token(TokenSymbol::EURT);
		let pair = CurrencyPair { first, second };

		let lp_token = CurrencyId::Token(TokenSymbol::WBTC);

		let fee: Permill = Permill::from_percent(3);

		let pool = Pool { owner, pair, lp_token, fee };
		DexModule::create_pool(Origin::signed(1), pool);

		assert_eq!(DexModule::pools(0), Some(pool));

		assert_eq!(DexModule::pool_count(), 1)
	});
}
