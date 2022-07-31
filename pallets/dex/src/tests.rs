use frame_support::{assert_noop, assert_ok};
use frame_system::{Config, EventRecord};
use sp_core::H256;
use sp_runtime::Permill;
use orml_traits::MultiCurrency;

use primitives::TokenSymbol;

use crate::{
	mock,
	mock::*,
	traits::{CurrencyPair, Pool},
	Error,
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

fn create_default_pool() -> Pool<AccountId, AssetId> {
	let owner: AccountId = ALICE;
	let first = ASSET_1;
	let second = ASSET_2;
	let pair = CurrencyPair { token_a: first, token_b: second };

	let lp_token = ASSET_3;

	let fee: Permill = Permill::from_percent(3);

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
