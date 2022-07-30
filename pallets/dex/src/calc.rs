use frame_support::ensure;
use sp_runtime::{
	traits::{IntegerSquareRoot, One, Zero},
	ArithmeticError, PerThing,
};

// pub fn compute_first_deposit_lp(
// 	base_amount: u128,
// 	quote_amount: u128,
// ) -> Result<u128, ArithmeticError> {
// 	base_amount
// 		.integer_sqrt_checked()
// 		.ok_or(ArithmeticError::Overflow)?
// 		.safe_mul(&quote_amount.integer_sqrt_checked().ok_or(ArithmeticError::Overflow)?)
// }

pub fn compute_deposit_lp(
	lp_total_issuance: u128,
	base_amount: u128,
	quote_amount: u128,
	pool_base_aum: u128,
	pool_quote_aum: u128,
) -> Result<(u128, u128), ArithmeticError> {
	// let first_deposit = lp_total_issuance.is_zero();
	// if first_deposit {
	// 	let lp_to_mint = compute_first_deposit_lp(base_amount, quote_amount)?;
	// 	Ok((quote_amount, lp_to_mint))
	// } else {
	// 	let overwritten_quote_amount =
	// 		safe_multiply_by_rational(pool_quote_aum, base_amount, pool_base_aum)?;
	// 	let lp_to_mint = safe_multiply_by_rational(lp_total_issuance, base_amount, pool_base_aum)?;
	// 	Ok((overwritten_quote_amount, lp_to_mint))
	// }
	let quote_amount = 10u128;
	let lp_to_mint = 10u128;
	Ok((quote_amount, lp_to_mint))
}
