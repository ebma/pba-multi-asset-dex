#![cfg_attr(not(feature = "std"), no_std)]

use codec::{EncodeLike, FullCodec};
use frame_support::{dispatch::DispatchResult, traits::Get, transactional};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedDiv, MaybeSerializeDeserialize},
	FixedPointNumber, FixedPointOperand,
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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;

mod pool;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, PalletId};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AccountIdConversion, One, Zero};

	use crate::pool::{Amm, CurrencyPair, Pool};

	use super::*;

	pub(crate) type AssetIdOf<T> = <T as Config>::AssetId;
	pub(crate) type BalanceOf<T> = <T as Config>::Balance;
	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	type PoolIdOf<T> = <T as Config>::PoolId;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + orml_tokens::Config<Balance = BalanceOf<Self>>
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type UnsignedFixedPoint: FixedPointNumber<Inner = BalanceOf<Self>>
			+ TruncateFixedPointToInt
			+ Encode
			+ EncodeLike
			+ Decode
			+ MaybeSerializeDeserialize
			+ TypeInfo;

		type SignedInner: Debug
			+ CheckedDiv
			+ TryFrom<BalanceOf<Self>>
			+ TryInto<BalanceOf<Self>>
			+ MaybeSerializeDeserialize;

		type SignedFixedPoint: FixedPointNumber<Inner = SignedInner<Self>>
			+ TruncateFixedPointToInt
			+ Encode
			+ EncodeLike
			+ Decode
			+ MaybeSerializeDeserialize;

		type Balance: AtLeast32BitUnsigned
			+ FixedPointOperand
			+ MaybeSerializeDeserialize
			+ FullCodec
			+ Copy
			+ Default
			+ TypeInfo
			+ Debug;

		#[pallet::constant]
		type GetNativeCurrencyId: Get<CurrencyId<Self>>;

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
			+ One;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type Assets: MultiCurrency<Self::AccountId>;
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
	pub type Pools<T: Config> =
		StorageMap<_, Blake2_128Concat, T::PoolId, Pool<T::AccountId, T::AssetId>>;

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
			first_amount: BalanceOf<T>,
			second_amount: BalanceOf<T>,
			minted_lp: BalanceOf<T>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		PoolNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create pool
		///
		/// Emits `PoolCreated` event when successful.
		#[pallet::weight(10_000)]
		pub fn create_pool(
			origin: OriginFor<T>,
			pool: Pool<T::AccountId, T::AssetId>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let pool_id =
				PoolCount::<T>::try_mutate(|pool_count| -> Result<T::PoolId, DispatchError> {
					let pool_id = *pool_count;
					Pools::<T>::insert(pool_id, pool.clone());
					*pool_count = pool_id + T::PoolId::one();
					Ok(pool_id)
				})?;

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
			first_amount: BalanceOf<T>,
			second_amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Self as Amm>::add_liquidity(&sender, pool_id, first_amount, second_amount)?;

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn get_pool(
			pool_id: T::PoolId,
		) -> Result<Pool<T::AccountId, T::AssetId>, DispatchError> {
			Pools::<T>::get(pool_id).ok_or_else(|| Error::<T>::PoolNotFound.into())
		}

		pub(crate) fn account_id(pool_id: &T::PoolId) -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(pool_id)
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

		fn get_exchange_value(
			pool_id: Self::PoolId,
			asset_id: Self::AssetId,
			amount: Self::Balance,
		) -> Result<Self::Balance, DispatchError> {
			todo!();
			Ok(Self::Balance::zero())
		}

		#[transactional]
		fn buy(
			who: &Self::AccountId,
			pool_id: Self::PoolId,
			asset_id: Self::AssetId,
			amount: Self::Balance,
		) -> Result<Self::Balance, DispatchError> {
			todo!();
			Ok(Self::Balance::zero())
		}

		#[transactional]
		fn sell(
			who: &Self::AccountId,
			pool_id: Self::PoolId,
			asset_id: Self::AssetId,
			amount: Self::Balance,
		) -> Result<Self::Balance, DispatchError> {
			todo!();
			Ok(Self::Balance::zero())
		}

		#[transactional]
		fn add_liquidity(
			who: &Self::AccountId,
			pool_id: Self::PoolId,
			first_amount: Self::Balance,
			second_amount: Self::Balance,
		) -> Result<(), DispatchError> {
			let pool = Self::get_pool(pool_id)?;
			let pool_account = Self::account_id(&pool_id);
			// TODO
			let first_amount: BalanceOf<T> = Self::Balance::zero();
			let second_amount: BalanceOf<T> = Self::Balance::zero();
			let minted_lp: BalanceOf<T> = Self::Balance::zero();
			Self::deposit_event(Event::<T>::LiquidityAdded {
				who: who.clone(),
				pool_id,
				first_amount,
				second_amount,
				minted_lp,
			});
			Ok(())
		}

		#[transactional]
		fn remove_liquidity(
			who: &Self::AccountId,
			pool_id: Self::PoolId,
			lp_amount: Self::Balance,
		) -> Result<(), DispatchError> {
			todo!();
			Ok(())
		}

		#[transactional]
		fn swap(
			who: &Self::AccountId,
			pool_id: Self::PoolId,
			pair: CurrencyPair<Self::AssetId>,
			quote_amount: Self::Balance,
		) -> Result<Self::Balance, DispatchError> {
			todo!();
			Ok(Self::Balance::zero())
		}
	}
}
