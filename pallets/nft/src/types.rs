use frame_support::dispatch::DispatchError;
use sp_runtime::FixedPointNumber;

use crate::Config;

pub(crate) type BalanceOf<T> = <T as Config>::Balance;
pub(crate) type AssetIdOf<T> = <T as Config>::AssetId;
pub(crate) type PriceOf<T> = (BalanceOf<T>, AssetIdOf<T>);
pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
