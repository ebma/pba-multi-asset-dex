#![cfg_attr(not(feature = "std"), no_std)]

use codec::{FullCodec};
use orml_traits::MultiCurrency;
pub use pallet::*;


use sp_runtime::{traits::AtLeast32BitUnsigned, ArithmeticError, FixedPointOperand};
use sp_std::{convert::TryInto, fmt::Debug};

#[cfg(test)]
pub mod mock;

#[cfg(test)]
mod tests;

mod traits;
mod types;
use traits::UniqueItem;
use types::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency},
	};
	use frame_system::pallet_prelude::*;

	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Balance: AtLeast32BitUnsigned
			+ FixedPointOperand
			+ MaybeSerializeDeserialize
			+ FullCodec
			+ Copy
			+ Default
			+ TypeInfo
			+ MaxEncodedLen
			+ Debug;

		/// The type of assets used by the Assets handler.
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

		/// The type used to identify a unique item within a collection.
		type ItemId: Member + Parameter + MaxEncodedLen + Copy + MaybeSerializeDeserialize;

		/// The MultiCurrency handler for this pallet.
		type Assets: MultiCurrency<
			Self::AccountId,
			Balance = BalanceOf<Self>,
			CurrencyId = Self::AssetId,
		>;

		/// The maximum length of a unique_item's data stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		/// The maximum amount of unique_items a single account can own.
		#[pallet::constant]
		type MaxUniqueItemsOwned: Get<u32>;
	}

	// Errors
	#[pallet::error]
	pub enum Error<T> {
		/// An account may only own `MaxUniqueItemsOwned` unique_items.
		TooManyOwned,
		/// Trying to transfer or buy a unique_item from oneself.
		TransferToSelf,
		/// This unique_item already exists!
		DuplicateUniqueItem,
		/// This unique_item does not exist!
		NoUniqueItem,
		/// You are not the owner of this unique_item.
		NotOwner,
		/// This unique_item is not for sale.
		NotForSale,
		/// Ensures that the buying price is greater than the asking price.
		BidPriceTooLow,
		/// You need to have two cats with different gender to breed.
		CantBreed,
	}

	// Events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new unique_item was successfully created.
		Created { unique_item: ItemIdOf<T>, owner: AccountIdOf<T> },
		/// The price of a unique_item was successfully set.
		PriceSet { unique_item: ItemIdOf<T>, price: Option<PriceOf<T>> },
		/// A unique_item was successfully transferred.
		Transferred { from: AccountIdOf<T>, to: AccountIdOf<T>, unique_item: ItemIdOf<T> },
		/// A unique_item was successfully sold.
		Sold {
			seller: AccountIdOf<T>,
			buyer: AccountIdOf<T>,
			unique_item: ItemIdOf<T>,
			price: PriceOf<T>,
		},
	}

	/// Keeps track of the number of unique_items in existence.
	#[pallet::storage]
	pub(super) type CountForUniqueItems<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Maps the unique_item struct to the unique_item DNA.
	#[pallet::storage]
	pub(super) type UniqueItems<T: Config> =
		StorageMap<_, Twox64Concat, ItemIdOf<T>, UniqueItem<T>>;

	/// Track the unique_items owned by each account.
	#[pallet::storage]
	pub(super) type UniqueItemsOwned<T: Config> = StorageMap<
		_,
		Twox64Concat,
		AccountIdOf<T>,
		BoundedVec<ItemIdOf<T>, T::MaxUniqueItemsOwned>,
		ValueQuery,
	>;

	// Our pallet's genesis configuration
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub unique_items: Vec<(AccountIdOf<T>, ItemIdOf<T>, DataOf<T>)>,
	}

	// Required to implement default for GenesisConfig
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { unique_items: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// When building a unique_item from genesis config, we require the DNA and Gender to be
			// supplied
			for (account, id, data) in &self.unique_items {
				assert!(Pallet::<T>::mint(account, *id, data.clone()).is_ok());
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new unique unique_item.
		///
		/// The actual unique_item creation is done in the `mint()` function.
		#[pallet::weight(0)]
		pub fn create_unique_item(
			origin: OriginFor<T>,
			item: ItemIdOf<T>,
			data: DataOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Write new unique_item to storage by calling helper function
			Self::mint(&sender, item, data)?;

			Ok(())
		}

		/// Directly transfer a unique_item to another recipient.
		///
		/// Any account that holds a unique_item can send it to another Account. This will reset the
		/// asking price of the unique_item, marking it not for sale.
		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: AccountIdOf<T>,
			unique_item_id: ItemIdOf<T>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let from = ensure_signed(origin)?;
			let unique_item =
				UniqueItems::<T>::get(&unique_item_id).ok_or(Error::<T>::NoUniqueItem)?;
			ensure!(unique_item.owner == from, Error::<T>::NotOwner);
			Self::do_transfer(unique_item_id, to, None)?;
			Ok(())
		}

		/// Buy a unique_item for sale. The `limit_price` parameter is set as a safeguard against
		/// the possibility that the seller front-runs the transaction by setting a high price. A
		/// front-end should assume that this value is always equal to the actual price of the
		/// unique_item. The buyer will always be charged the actual price of the unique_item.
		///
		/// If successful, this dispatchable will reset the price of the unique_item to `None`,
		/// making it no longer for sale and handle the balance and unique_item transfer between the
		/// buyer and seller.
		#[pallet::weight(0)]
		pub fn buy_unique_item(
			origin: OriginFor<T>,
			unique_item_id: ItemIdOf<T>,
			limit_price: PriceOf<T>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let buyer = ensure_signed(origin)?;
			// Transfer the unique_item from seller to buyer as a sale
			Self::do_transfer(unique_item_id, buyer, Some(limit_price))?;

			Ok(())
		}

		/// Set the price for a unique_item.
		///
		/// Updates unique_item price and updates storage.
		#[pallet::weight(0)]
		pub fn set_price(
			origin: OriginFor<T>,
			unique_item_id: ItemIdOf<T>,
			new_price: Option<PriceOf<T>>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			// Ensure the unique_item exists and is called by the unique_item owner
			let mut unique_item =
				UniqueItems::<T>::get(&unique_item_id).ok_or(Error::<T>::NoUniqueItem)?;
			ensure!(unique_item.owner == sender, Error::<T>::NotOwner);

			// Set the price in storage
			unique_item.price = new_price;
			UniqueItems::<T>::insert(&unique_item_id, unique_item);

			// Deposit a "PriceSet" event.
			Self::deposit_event(Event::PriceSet { unique_item: unique_item_id, price: new_price });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn mint(owner: &AccountIdOf<T>, item: ItemIdOf<T>, data: DataOf<T>) -> DispatchResult {
			// Create a new object
			let unique_item = UniqueItem::<T> { id: item, price: None, data, owner: owner.clone() };

			// Check if the unique_item does not already exist in our storage map
			ensure!(
				!UniqueItems::<T>::contains_key(&unique_item.id),
				Error::<T>::DuplicateUniqueItem
			);

			// Performs this operation first as it may fail
			let count = CountForUniqueItems::<T>::get();
			let new_count = count.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			// Append unique_item to UniqueItemsOwned
			UniqueItemsOwned::<T>::try_append(&owner, unique_item.id)
				.map_err(|_| Error::<T>::TooManyOwned)?;

			// Write new unique_item to storage
			UniqueItems::<T>::insert(unique_item.id, unique_item);
			CountForUniqueItems::<T>::put(new_count);

			// Deposit our "Created" event.
			Self::deposit_event(Event::Created { unique_item: item, owner: owner.clone() });

			Ok(())
		}

		// Update storage to transfer unique_item
		pub fn do_transfer(
			unique_item_id: ItemIdOf<T>,
			to: AccountIdOf<T>,
			maybe_limit_price: Option<PriceOf<T>>,
		) -> DispatchResult {
			// Get the unique_item
			let mut unique_item =
				UniqueItems::<T>::get(&unique_item_id).ok_or(Error::<T>::NoUniqueItem)?;
			let from = unique_item.owner;

			ensure!(from != to, Error::<T>::TransferToSelf);
			let mut from_owned = UniqueItemsOwned::<T>::get(&from);

			// Remove unique_item from list of owned unique_items.
			if let Some(ind) = from_owned.iter().position(|&id| id == unique_item_id) {
				from_owned.swap_remove(ind);
			} else {
				return Err(Error::<T>::NoUniqueItem.into())
			}

			// Add unique_item to the list of owned unique_items.
			let mut to_owned = UniqueItemsOwned::<T>::get(&to);
			to_owned.try_push(unique_item_id).map_err(|()| Error::<T>::TooManyOwned)?;

			// Mutating state here via a balance transfer, so nothing is allowed to fail after this.
			// The buyer will always be charged the actual price. The limit_price parameter is just
			// a protection so the seller isn't able to front-run the transaction.
			if let Some(limit_price) = maybe_limit_price {
				// Current unique_item price if for sale
				if let Some((price, asset)) = unique_item.price {
					ensure!(limit_price.0 >= price, Error::<T>::BidPriceTooLow);
					// Transfer the amount from buyer to seller
					T::Assets::transfer(asset, &to, &from, price)?;
					// Deposit sold event
					Self::deposit_event(Event::Sold {
						seller: from.clone(),
						buyer: to.clone(),
						unique_item: unique_item_id,
						price: (price, asset),
					});
				} else {
					// UniqueItem price is set to `None` and is not for sale
					return Err(Error::<T>::NotForSale.into())
				}
			}

			// Transfer succeeded, update the unique_item owner and reset the price to `None`.
			unique_item.owner = to.clone();
			unique_item.price = None;

			// Write updates to storage
			UniqueItems::<T>::insert(&unique_item_id, unique_item);
			UniqueItemsOwned::<T>::insert(&to, to_owned);
			UniqueItemsOwned::<T>::insert(&from, from_owned);

			Self::deposit_event(Event::Transferred { from, to, unique_item: unique_item_id });

			Ok(())
		}
	}
}
