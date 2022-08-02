use crate::{AccountIdOf, Config, DataOf, ItemIdOf, PriceOf};
use codec::{Codec, Decode, Encode, FullCodec, MaxEncodedLen};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_runtime::{traits::Get, BoundedVec};

// Struct for holding unique_item information
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct UniqueItem<T: Config> {
	pub data: DataOf<T>,
	pub id: ItemIdOf<T>,
	pub owner: AccountIdOf<T>,
	// `None` assumes not for sale
	pub price: Option<PriceOf<T>>,
}
