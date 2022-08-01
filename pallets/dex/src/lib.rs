#![cfg_attr(not(feature = "std"), no_std)]

use codec::{EncodeLike, FullCodec};
use frame_support::{dispatch::DispatchResult, traits::Get, transactional};
use num_integer::{sqrt, Roots};
use orml_traits::{arithmetic::CheckedAdd, MultiCurrency, MultiReservableCurrency};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{
		AccountIdConversion, AtLeast32BitUnsigned, CheckedDiv, MaybeSerializeDeserialize, One, Zero,
	},
	ArithmeticError, FixedPointNumber, FixedPointOperand,
};
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
};

pub use pallet::*;
use primitives::TruncateFixedPointToInt;
pub use types::CurrencyId;
use types::*;

mod calc;
mod traits;
mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, PalletId};
	use frame_system::pallet_prelude::*;
	use sp_arithmetic::{PerThing, Permill, UpperOf};
	use sp_runtime::traits::{CheckedAdd, CheckedMul, CheckedSub, Convert, Saturating};

	use crate::traits::{Amm, CurrencyPair, Pool};

	use super::*;

	pub(crate) type AssetIdOf<T> = <T as Config>::AssetId;
	pub(crate) type BalanceOf<T> = <T as Config>::Balance;
	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type PoolOf<T> = Pool<AccountIdOf<T>, AssetIdOf<T>>;
	pub(crate) type PoolIdOf<T> = <T as Config>::PoolId;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Balance: AtLeast32BitUnsigned
			+ FixedPointOperand
			+ MaybeSerializeDeserialize
			+ FullCodec
			+ Copy
			+ Default
			+ TypeInfo
			+ Debug;

		type AssetId: FullCodec
			+ MaxEncodedLen
			+ Eq
			+ PartialEq
			+ Copy
			+ Clone
			+ MaybeSerializeDeserialize
			+ Debug
			+ Default
			+ TypeInfo
			+ Ord;

		type PoolId: FullCodec
			+ MaxEncodedLen
			+ Default
			+ Debug
			+ TypeInfo
			+ Eq
			+ PartialEq
			+ Ord
			+ Copy
			+ Zero
			+ CheckedAdd
			+ One;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type Assets: MultiCurrency<
			Self::AccountId,
			Balance = BalanceOf<Self>,
			CurrencyId = Self::AssetId,
		>;

		type Convert: Convert<u128, BalanceOf<Self>> + Convert<BalanceOf<Self>, u128>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn pool_count)]
	#[allow(clippy::disallowed_types)]
	pub type PoolCount<T: Config> = StorageValue<_, T::PoolId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub type Pools<T: Config> = StorageMap<_, Blake2_128Concat, T::PoolId, PoolOf<T>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		PoolCreated {
			pool_id: T::PoolId,
			owner: T::AccountId,
			assets: CurrencyPair<AssetIdOf<T>>,
		},
		LiquidityAdded {
			who: T::AccountId,
			pool_id: T::PoolId,
			amount_a: BalanceOf<T>,
			amount_b: BalanceOf<T>,
			minted_lp: BalanceOf<T>,
		},
		LiquidityRemoved {
			who: T::AccountId,
			pool_id: PoolIdOf<T>,
			amount_a: BalanceOf<T>,
			amount_b: BalanceOf<T>,
			total_issuance: BalanceOf<T>,
		},
		Swapped {
			who: T::AccountId,
			pool_id: PoolIdOf<T>,
			token_a: AssetIdOf<T>,
			token_b: AssetIdOf<T>,
			amount_a: BalanceOf<T>,
			amount_b: BalanceOf<T>,
			/// Charged fees.
			fee: Permill,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		PairMismatch,
		PoolNotFound,
		MaximumPoolCountReached,
		InvalidAmount,
		InvalidAsset,
		InsufficientBalance,
		InsufficientInputAmount,
		InsufficientOutputAmount,
		InsufficientLiquidity,
		InsufficientLiquidityBalance,
		InvalidExchangeValue,
		WithdrawWithoutSupply,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create pool
		///
		/// Emits `PoolCreated` event when successful.
		#[pallet::weight(10_000)]
		pub fn create_pool(origin: OriginFor<T>, pool: PoolOf<T>) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let pool_id = Self::_create_pool(pool.clone())?;

			Self::deposit_event(Event::<T>::PoolCreated {
				owner: pool.owner,
				pool_id,
				assets: pool.pair,
			});

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			amount: BalanceOf<T>,
			asset: AssetIdOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as Amm>::add_liquidity(&sender, pool_id, amount, asset)?;

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as Amm>::remove_liquidity(&sender, pool_id, amount)?;

			Ok(())
		}

		/// Execute a swap
		/// The user will send amount_b of token_b to receive the corresponding amount_a of token_a.
		#[pallet::weight(10_000)]
		pub fn swap(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			pair: CurrencyPair<AssetIdOf<T>>,
			amount_b: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<Self as Amm>::swap(&who, pool_id, pair, amount_b)?;
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn buy(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			asset_id: AssetIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<Self as Amm>::buy(&who, pool_id, asset_id, amount)?;
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn sell(
			origin: OriginFor<T>,
			pool_id: PoolIdOf<T>,
			asset_id: AssetIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			<Self as Amm>::sell(&who, pool_id, asset_id, amount)?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn get_pool(pool_id: PoolIdOf<T>) -> Result<PoolOf<T>, DispatchError> {
			Pools::<T>::get(pool_id).ok_or_else(|| Error::<T>::PoolNotFound.into())
		}

		pub(crate) fn account_id(pool_id: &PoolIdOf<T>) -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(pool_id)
		}

		fn _create_pool(pool: PoolOf<T>) -> Result<PoolIdOf<T>, DispatchError> {
			let pool_id =
				PoolCount::<T>::try_mutate(|pool_count| -> Result<T::PoolId, DispatchError> {
					let pool_id = *pool_count;
					Pools::<T>::insert(pool_id, pool.clone());
					*pool_count = pool_id
						.checked_add(&T::PoolId::one())
						.ok_or(Error::<T>::MaximumPoolCountReached)?;
					Ok(pool_id)
				})?;
			Ok(pool_id)
		}

		// Adapted from https://github.com/Uniswap/v2-periphery/blob/master/contracts/libraries/UniswapV2Library.sol
		pub fn get_amount_in(
			amount_out: u128,
			reserve_in: u128,
			reserve_out: u128,
			fee: Permill,
		) -> Result<u128, Error<T>> {
			ensure!(amount_out > 0, Error::<T>::InsufficientOutputAmount);
			ensure!(reserve_in > 0 && reserve_out > 0, Error::<T>::InsufficientLiquidity);

			let multiplier: u128 = 1000;
			let fee_multiplier: u128 = 1000u128.saturating_sub(fee.mul_floor(1000));

			let numerator = reserve_in.saturating_mul(amount_out).saturating_mul(multiplier);
			let denominator = fee_multiplier.saturating_mul(reserve_out.saturating_sub(amount_out));
			let result =
				numerator.checked_div(denominator).and_then(|x| x.checked_add(1)).unwrap_or(0);

			Ok(result)
		}

		// Adapted from https://github.com/Uniswap/v2-periphery/blob/master/contracts/libraries/UniswapV2Library.sol
		fn get_amount_out(
			amount_in: u128,
			reserve_in: u128,
			reserve_out: u128,
			fee: Permill,
		) -> Result<u128, DispatchError> {
			ensure!(amount_in > 0, Error::<T>::InsufficientInputAmount);
			ensure!(reserve_in > 0 && reserve_out > 0, Error::<T>::InsufficientLiquidity);

			let multiplier: u128 = 1000;
			let fee_multiplier: u128 = 1000u128.saturating_sub(fee.mul_floor(1000));

			// Subtract fee from amount_in
			let amount_in_with_fee = amount_in.saturating_mul(fee_multiplier);
			let numerator = amount_in_with_fee.saturating_mul(reserve_out);
			let denominator =
				reserve_in.saturating_mul(multiplier).saturating_add(amount_in_with_fee);
			let result = numerator.checked_div(denominator).unwrap_or(0);
			Ok(result)
		}
	}

	impl<T: Config> Amm for Pallet<T> {
		type AssetId = T::AssetId;
		type Balance = BalanceOf<T>;
		type AccountId = T::AccountId;
		type PoolId = T::PoolId;

		fn pool_exists(pool_id: Self::PoolId) -> bool {
			Pools::<T>::contains_key(pool_id)
		}

		fn currency_pair(
			pool_id: Self::PoolId,
		) -> Result<CurrencyPair<Self::AssetId>, DispatchError> {
			let pool = Self::get_pool(pool_id)?;
			Ok(pool.pair)
		}

		fn lp_token(pool_id: Self::PoolId) -> Result<Self::AssetId, DispatchError> {
			let pool = Self::get_pool(pool_id)?;
			Ok(pool.lp_token)
		}

		/// Return the balances of the assets in the pool.
		fn pool_reserves(
			pool_id: Self::PoolId,
		) -> Result<(Self::Balance, Self::Balance), DispatchError> {
			let pool = Self::get_pool(pool_id)?;
			let pool_account = Self::account_id(&pool_id);

			let pair = pool.pair;
			let reserve_a = T::Assets::free_balance(pair.token_a, &pool_account);
			let reserve_b = T::Assets::free_balance(pair.token_b, &pool_account);
			Ok((reserve_a, reserve_b))
		}

		/// Calculate the value of a given asset in a given pool based on the other asset in the
		/// currency pair.
		fn get_exchange_value(
			pool_id: Self::PoolId,
			asset_id: Self::AssetId,
			amount: Self::Balance,
		) -> Result<Self::Balance, DispatchError> {
			let pool = Self::get_pool(pool_id)?;
			let pool_account = Self::account_id(&pool_id);
			let pair = pool.pair;
			let (reserve_a, reserve_b) = Self::pool_reserves(pool_id)?;

			let quote = if pair.token_a == asset_id {
				amount.checked_mul(&reserve_b).and_then(|x| x.checked_div(&reserve_a))
			} else if pair.token_b == asset_id {
				amount.checked_mul(&reserve_a).and_then(|x| x.checked_div(&reserve_b))
			} else {
				return Err(Error::<T>::InvalidAsset.into())
			};
			quote.ok_or(Error::<T>::InvalidAmount.into())
		}

		#[transactional]
		fn buy(
			who: &Self::AccountId,
			pool_id: Self::PoolId,
			asset_id: Self::AssetId,
			amount: Self::Balance,
		) -> Result<Self::Balance, DispatchError> {
			let pool = Self::get_pool(pool_id)?;
			let pair = if asset_id == pool.pair.token_a { pool.pair } else { pool.pair.swap() };

			// Compute how much user has to pay to buy the given amount of the given asset.
			let (reserve_a, reserve_b) = Self::pool_reserves(pool_id)?;
			let (reserve_a, reserve_b) =
				(T::Convert::convert(reserve_a), T::Convert::convert(reserve_b));
			let amount = T::Convert::convert(amount);

			let sell_amount = Self::get_amount_in(amount, reserve_a, reserve_b, pool.fee)?;
			let sell_amount = T::Convert::convert(sell_amount);
			<Self as Amm>::swap(who, pool_id, pair, sell_amount)
		}

		#[transactional]
		fn sell(
			who: &Self::AccountId,
			pool_id: Self::PoolId,
			asset_id: Self::AssetId,
			amount: Self::Balance,
		) -> Result<Self::Balance, DispatchError> {
			let pool = Self::get_pool(pool_id)?;
			let pair = if asset_id == pool.pair.token_a { pool.pair.swap() } else { pool.pair };
			<Self as Amm>::swap(who, pool_id, pair, amount)
		}

		#[transactional]
		fn add_liquidity(
			who: &Self::AccountId,
			pool_id: Self::PoolId,
			amount: Self::Balance,
			asset: Self::AssetId,
		) -> Result<(), DispatchError> {
			let pool = Self::get_pool(pool_id)?;
			let pool_account = Self::account_id(&pool_id);

			let (reserve_a, reserve_b) = Self::pool_reserves(pool_id)?;
			let (amount_a, amount_b) = if reserve_a.is_zero() && reserve_b.is_zero() {
				(amount, amount)
			} else if pool.pair.token_a == asset {
				let other_amount =
					<Self as Amm>::get_exchange_value(pool_id, pool.pair.token_b, amount)?;
				(amount, other_amount)
			} else if pool.pair.token_b == asset {
				let other_amount =
					<Self as Amm>::get_exchange_value(pool_id, pool.pair.token_a, amount)?;
				(other_amount, amount)
			} else {
				return Err(Error::<T>::InvalidAsset.into())
			};

			let user_balance_a = T::Assets::free_balance(pool.pair.token_a, who);
			if user_balance_a < amount_a {
				return Err(Error::<T>::InsufficientBalance.into())
			}

			let user_balance_b = T::Assets::free_balance(pool.pair.token_b, who);
			if user_balance_b < amount_b {
				return Err(Error::<T>::InsufficientBalance.into())
			}

			// Convert to u128 for calculations
			let (amount_a, amount_b) =
				(T::Convert::convert(amount_a), T::Convert::convert(amount_b));

			let lp_total_issuance = T::Convert::convert(T::Assets::total_issuance(pool.lp_token));
			let amount_of_lp_token_to_mint =
				if lp_total_issuance == 0 || (reserve_a.is_zero() && reserve_b.is_zero()) {
					let product = amount_a.saturating_mul(amount_b);
					sqrt(product)
				} else {
					core::cmp::min(
						amount_a
							.saturating_mul(lp_total_issuance)
							.checked_div(T::Convert::convert(reserve_a))
							.ok_or(ArithmeticError::DivisionByZero)?,
						amount_b
							.saturating_mul(lp_total_issuance)
							.checked_div(T::Convert::convert(reserve_b))
							.ok_or(ArithmeticError::DivisionByZero)?,
					)
				};

			// Convert back to balances
			let (amount_a, amount_b) =
				(T::Convert::convert(amount_a), T::Convert::convert(amount_b));
			let amount_of_lp_token_to_mint = T::Convert::convert(amount_of_lp_token_to_mint);

			T::Assets::transfer(pool.pair.token_a, who, &pool_account, amount_a)?;
			T::Assets::transfer(pool.pair.token_b, who, &pool_account, amount_b)?;
			T::Assets::deposit(pool.lp_token, who, amount_of_lp_token_to_mint)?;

			Self::deposit_event(Event::<T>::LiquidityAdded {
				who: who.clone(),
				pool_id,
				amount_a,
				amount_b,
				minted_lp: amount_of_lp_token_to_mint,
			});
			Ok(())
		}

		#[transactional]
		fn remove_liquidity(
			who: &Self::AccountId,
			pool_id: Self::PoolId,
			amount: Self::Balance,
		) -> Result<(), DispatchError> {
			let pool = Self::get_pool(pool_id)?;
			let pool_account = Self::account_id(&pool_id);
			let total_issuance = T::Assets::total_issuance(pool.lp_token);
			ensure!(!total_issuance.is_zero(), Error::<T>::WithdrawWithoutSupply);

			let user_lp_balance = T::Assets::free_balance(pool.lp_token, who);
			ensure!(user_lp_balance >= amount, Error::<T>::InsufficientLiquidityBalance);

			let (reserve_a, reserve_b) = Self::pool_reserves(pool_id)?;
			// Convert to u128 for calculations
			let amount = T::Convert::convert(amount);
			let total_issuance = T::Convert::convert(total_issuance);
			let (reserve_a, reserve_b) =
				(T::Convert::convert(reserve_a), T::Convert::convert(reserve_b));

			/// Calculate the amounts of tokens the user will receive for removing liquidity
			let amount_a = amount.checked_mul(reserve_a).and_then(|x| x.checked_div(total_issuance));
			let amount_b =
				amount.checked_mul(reserve_b).and_then(|x| x.checked_div(total_issuance));

			ensure!(amount_a.is_some() && amount_b.is_some(), Error::<T>::InvalidAmount);

			// Unwrap and convert to Balance
			let (amount_a, amount_b) = (
				T::Convert::convert(amount_a.expect("Checked to be Some; qed")),
				T::Convert::convert(amount_b.expect("Checked to be Some; qed")),
			);
			let amount = T::Convert::convert(amount);

			ensure!(!amount_a.is_zero() && !amount_b.is_zero(), Error::<T>::InvalidAmount);

			T::Assets::transfer(pool.pair.token_a, &pool_account, who, amount_a)?;
			T::Assets::transfer(pool.pair.token_b, &pool_account, who, amount_b)?;
			T::Assets::withdraw(pool.lp_token, who, amount)?;

			let total_issuance = T::Assets::total_issuance(pool.lp_token);

			Self::deposit_event(Event::<T>::LiquidityRemoved {
				who: who.clone(),
				pool_id,
				amount_a,
				amount_b,
				total_issuance,
			});

			Ok(())
		}

		/// Execute a swap and return the amount of tokens received by executing the swap
		/// The order of assets in the CurrencyPair will decide how the swap is executed
		/// The user will spend `amount_b_in` tokens of token b to receive some tokens of token_a
		#[transactional]
		fn swap(
			who: &Self::AccountId,
			pool_id: Self::PoolId,
			pair: CurrencyPair<Self::AssetId>,
			amount_b_in: Self::Balance,
		) -> Result<Self::Balance, DispatchError> {
			let pool = Self::get_pool(pool_id)?;
			let pool_account = Self::account_id(&pool_id);

			ensure!(pair == pool.pair, Error::<T>::PairMismatch);

			let (reserve_a, reserve_b) = Self::pool_reserves(pool_id)?;

			// Convert to u128 for calculations
			let amount_b = T::Convert::convert(amount_b_in);
			let (reserve_a, reserve_b) =
				(T::Convert::convert(reserve_a), T::Convert::convert(reserve_b));

			let amount_a = Self::get_amount_out(amount_b, reserve_b, reserve_a, pool.fee)?;
			ensure!(amount_a > 0, Error::<T>::InvalidAmount);

			// Convert back to balances
			let amount_a = T::Convert::convert(amount_a);
			let amount_b = T::Convert::convert(amount_b);

			T::Assets::transfer(pair.token_b, who, &pool_account, amount_b)?;
			T::Assets::transfer(pair.token_a, &pool_account, who, amount_a)?;

			Self::deposit_event(Event::<T>::Swapped {
				pool_id,
				who: who.clone(),
				token_a: pair.token_a,
				token_b: pair.token_b,
				amount_a,
				amount_b,
				fee: pool.fee,
			});
			Ok(amount_a)
		}
	}
}
