use frame_support::{assert_noop, assert_ok};
use frame_system::{Config, EventRecord};
use sp_core::H256;
use sp_runtime::Permill;

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
	assert!(matcher(System::events().last().expect("events expected")));
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
fn remove_pool_should_work() {
	run_test(|| {
		let pool = create_default_pool();
		Dex::create_pool(Origin::signed(ALICE), pool);

		assert_eq!(Dex::pools(0), Some(pool));

		assert_eq!(Dex::pool_count(), 1)
	});
}

#[test]
fn add_liquidity_should_work() {
	run_test(|| {
		let pool = create_default_pool();

		Dex::create_pool(Origin::signed(ALICE), pool);
		Dex::add_liquidity(Origin::signed(ALICE), 0, 100, 100);
		println!("{:?}", Dex::pools(0));
	});
}
