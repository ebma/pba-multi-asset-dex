use codec::{Codec, Decode, Encode, FullCodec, MaxEncodedLen};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, Permill};
use sp_std::cmp::Ordering;

#[derive(RuntimeDebug, Encode, Decode, MaxEncodedLen, Copy, Clone, PartialEq, Eq, TypeInfo)]
pub struct CurrencyPair<AssetId> {
	pub token_a: AssetId,
	pub token_b: AssetId,
}

#[derive(RuntimeDebug, Encode, Decode, MaxEncodedLen, Copy, Clone, PartialEq, Eq, TypeInfo)]
pub struct Pool<AccountId, AssetId: Ord> {
	pub owner: AccountId,
	pub pair: CurrencyPair<AssetId>,
	pub lp_token: AssetId,
	pub fee: Permill,
}

pub trait Amm {
	type AssetId;
	type Balance;
	type AccountId;
	type PoolId;

	fn pool_exists(pool_id: Self::PoolId) -> bool;

	fn currency_pair(pool_id: Self::PoolId) -> Result<CurrencyPair<Self::AssetId>, DispatchError>;

	fn lp_token(pool_id: Self::PoolId) -> Result<Self::AssetId, DispatchError>;

	fn pool_reserves(
		pool_id: Self::PoolId,
	) -> Result<(Self::Balance, Self::Balance), DispatchError>;

	fn get_exchange_value(
		pool_id: Self::PoolId,
		asset_id: Self::AssetId,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError>;

	fn buy(
		who: &Self::AccountId,
		pool_id: Self::PoolId,
		asset_id: Self::AssetId,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError>;

	fn sell(
		who: &Self::AccountId,
		pool_id: Self::PoolId,
		asset_id: Self::AssetId,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError>;

	fn add_liquidity(
		who: &Self::AccountId,
		pool_id: Self::PoolId,
		amount: Self::Balance,
		asset: Self::AssetId,
	) -> Result<(), DispatchError>;

	fn remove_liquidity(
		who: &Self::AccountId,
		pool_id: Self::PoolId,
		lp_amount: Self::Balance,
	) -> Result<(), DispatchError>;

	fn swap(
		who: &Self::AccountId,
		pool_id: Self::PoolId,
		pair: CurrencyPair<Self::AssetId>,
		quote_amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError>;
}
