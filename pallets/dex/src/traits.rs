use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_runtime::{DispatchError, Permill};


#[derive(RuntimeDebug, Encode, Decode, MaxEncodedLen, Copy, Clone, Eq, TypeInfo)]
pub struct CurrencyPair<AssetId> {
	pub token_a: AssetId,
	pub token_b: AssetId,
}

impl<AssetId: Copy + PartialEq> CurrencyPair<AssetId> {
	pub fn swap(&self) -> Self {
		Self { token_a: self.token_b, token_b: self.token_a }
	}

	pub fn contains(&self, asset_id: AssetId) -> bool {
		self.token_a == asset_id || self.token_b == asset_id
	}
}

impl<AssetId: PartialEq> PartialEq for CurrencyPair<AssetId> {
	fn eq(&self, other: &Self) -> bool {
		(self.token_a == other.token_a && self.token_b == other.token_b) ||
			(self.token_a == other.token_b && self.token_b == other.token_a)
	}
}

#[derive(RuntimeDebug, Encode, Decode, MaxEncodedLen, Copy, Clone, PartialEq, Eq, TypeInfo)]
pub struct PoolCreationParams<AccountId, AssetId: Ord> {
	pub owner: AccountId,
	pub pair: CurrencyPair<AssetId>,
	pub fee: Permill,
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
		amount_b: Self::Balance,
	) -> Result<Self::Balance, DispatchError>;
}
