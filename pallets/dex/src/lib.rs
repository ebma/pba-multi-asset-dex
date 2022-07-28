#![cfg_attr(not(feature = "std"), no_std)]

use codec::{EncodeLike, FullCodec};
use frame_support::{dispatch::DispatchResult, traits::Get};
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

pub use amount::Amount;
pub use pallet::*;
use primitives::TruncateFixedPointToInt;
use types::*;
pub use types::{CurrencyConversion, CurrencyId};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod amount;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;

#[frame_support::pallet]
pub mod pallet {
	use std::fmt::Debug;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	use super::*;

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
			+ Debug;

		/// Native currency e.g. INTR/KINT
		#[pallet::constant]
		type GetNativeCurrencyId: Get<CurrencyId<Self>>;

		type CurrencyConversion: types::CurrencyConversion<Amount<Self>, CurrencyId<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
	}
}
