use crate::{Config, Error};
use frame_support::ensure;
use sp_arithmetic::Permill;
use sp_runtime::{
	PerThing,
};

pub(crate) fn get_amount_in<T: Config>(
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
	let result = numerator.checked_div(denominator).and_then(|x| x.checked_add(1)).unwrap_or(0);

	Ok(result)
}

pub(crate) fn get_amount_out<T: Config>(
	amount_in: u128,
	reserve_in: u128,
	reserve_out: u128,
	fee: Permill,
) -> Result<u128, Error<T>> {
	ensure!(amount_in > 0, Error::<T>::InsufficientInputAmount);
	ensure!(reserve_in > 0 && reserve_out > 0, Error::<T>::InsufficientLiquidity);

	let multiplier: u128 = 1000;
	let fee_multiplier: u128 = 1000u128.saturating_sub(fee.mul_floor(1000));

	// Subtract fee from amount_in
	let amount_in_with_fee = amount_in.saturating_mul(fee_multiplier);
	let numerator = amount_in_with_fee.saturating_mul(reserve_out);
	let denominator = reserve_in.saturating_mul(multiplier).saturating_add(amount_in_with_fee);
	let result = numerator.checked_div(denominator).unwrap_or(0);
	Ok(result)
}
