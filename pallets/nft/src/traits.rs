use crate::{Config, PriceOf};
use codec::{Codec, Decode, Encode, FullCodec, MaxEncodedLen};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

#[cfg(feature = "std")]
use frame_support::serde::{Deserialize, Serialize};

// Struct for holding unique_item information
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct UniqueItem<T: Config> {
    // Using 16 bytes to represent a unique_item DNA
    pub dna: [u8; 16],
    // `None` assumes not for sale
    pub price: Option<PriceOf<T>>,
    pub gender: Gender,
    pub owner: T::AccountId,
}

// Set Gender type in unique_item struct
#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
// We need this to pass unique_item info for genesis configuration
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Gender {
    Male,
    Female,
}