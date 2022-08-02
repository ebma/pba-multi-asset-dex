use crate::Config;
use frame_support::dispatch::DispatchError;
use sp_runtime::{BoundedVec, FixedPointNumber};

pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub(crate) type AssetIdOf<T> = <T as Config>::AssetId;
pub(crate) type BalanceOf<T> = <T as Config>::Balance;
pub(crate) type DataOf<T> = BoundedVec<u8, <T as Config>::StringLimit>;
pub(crate) type ItemIdOf<T> = <T as Config>::ItemId;
pub(crate) type PriceOf<T> = (BalanceOf<T>, AssetIdOf<T>);
