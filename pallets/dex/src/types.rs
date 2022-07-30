use frame_support::dispatch::DispatchError;
use sp_runtime::FixedPointNumber;

use crate::Config;

pub type CurrencyId<T> = <T as orml_tokens::Config>::CurrencyId;

pub(crate) type BalanceOf<T> = <T as Config>::Balance;
