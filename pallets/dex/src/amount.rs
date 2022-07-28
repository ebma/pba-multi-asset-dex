use crate::{
	pallet::{self, Config, Error},
	types::*,
};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use primitives::TruncateFixedPointToInt;
use sp_runtime::{
	traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, UniqueSaturatedInto, Zero},
	FixedPointNumber,
};
use sp_std::{convert::TryInto, fmt::Debug};

#[cfg_attr(feature = "testing-utils", derive(Copy))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Amount<T: Config> {
	amount: BalanceOf<T>,
	currency_id: CurrencyId<T>,
}

#[cfg_attr(feature = "testing-utils", mocktopus::macros::mockable)]
impl<T: Config> Amount<T> {
	pub const fn new(amount: BalanceOf<T>, currency_id: CurrencyId<T>) -> Self {
		Self { amount, currency_id }
	}

	pub fn amount(&self) -> BalanceOf<T> {
		self.amount
	}

	pub fn currency(&self) -> CurrencyId<T> {
		self.currency_id
	}
}
