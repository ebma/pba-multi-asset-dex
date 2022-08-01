use crate::{CurrencyId, LongSymbol, ShortSymbol, TokenSymbol};
use sp_runtime::traits::{Convert, LookupError, StaticLookup};
use sp_std::{convert::TryInto, str::from_utf8, vec::Vec};

pub struct CurrencyConversion;

fn to_look_up_error(_: &'static str) -> LookupError {
	LookupError
}

impl StaticLookup for CurrencyConversion {
	type Source = CurrencyId;
	type Target = (CurrencyId, CurrencyId);

	fn lookup(
		currency: <Self as StaticLookup>::Source,
	) -> Result<<Self as StaticLookup>::Target, LookupError> {
		if let CurrencyId::Token(TokenSymbol::Long(long_symbol)) = currency {
			let mut short_symbol_a: ShortSymbol = [0; 4];
			short_symbol_a.copy_from_slice(&long_symbol[0..4]);

			let mut short_symbol_b: ShortSymbol = [0; 4];
			short_symbol_b.copy_from_slice(&long_symbol[4..]);

			Ok((
				CurrencyId::Token(TokenSymbol::Short(short_symbol_a)),
				CurrencyId::Token(TokenSymbol::Short(short_symbol_b)),
			))
		} else {
			Err(LookupError)
		}
	}

	fn unlookup(
		(currency_a, currency_b): <Self as StaticLookup>::Target,
	) -> <Self as StaticLookup>::Source {
		if let (
			CurrencyId::Token(TokenSymbol::Short(short_symbol_a)),
			CurrencyId::Token(TokenSymbol::Short(short_symbol_b)),
		) = (currency_a, currency_b)
		{
			let mut long: LongSymbol = [0; 8];
			long[..short_symbol_a.len()].copy_from_slice(&short_symbol_a);
			long[short_symbol_a.len()..].copy_from_slice(&short_symbol_b);
			// let long_symbol: LongSymbol = *[short_symbol_a, short_symbol_b].concat().as_bytes();
			CurrencyId::Token(TokenSymbol::Long(long))
		} else {
			CurrencyId::Native
		}
		// let mut long: LongSymbol = [0; 8];
		// long[..symbol_a.len()].copy_from_slice(symbol_a.as_bytes());
		// long[symbol_a.len()..].copy_from_slice(symbol_b.as_bytes());
		//
		// CurrencyId::Token(TokenSymbol::Long(long))
	}
}
