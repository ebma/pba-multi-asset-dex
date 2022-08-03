export function hexToAscii(hex) {
  if (!(typeof hex === 'number' || typeof hex == 'string')) {
    return ''
  }

  hex = hex.toString().replace(/\s+/gi, '')
  const stack = []

  for (var i = 0; i < hex.length; i += 2) {
    const code = parseInt(hex.substr(i, 2), 16)
    if (!isNaN(code) && code !== 0) {
      stack.push(String.fromCharCode(code))
    }
  }

  return stack.join('')
}

// build a price that is compatible with the runtime
// if the given currency is invalid, just use 'native'
export function buildPrice(amount, tokenSymbol) {
  if (!tokenSymbol) {
    return [amount, 'native']
  }

  let token = buildCurrency(tokenSymbol)
  let price = [amount, token]
  return price
}

export function buildCurrency(tokenSymbol) {
  let currency
  if (tokenSymbol.length === 4) {
    currency = {
      token: {
        short: tokenSymbol,
      },
    }
  } else if (tokenSymbol.length === 8) {
    currency = {
      token: {
        long: tokenSymbol,
      },
    }
  } else {
    currency = 'native'
  }
  return currency
}

export function priceToString(price) {
  if (price) {
    return `${price[0]} ${currencyToString(price[1])}`
  }
}

export function currencyToString(currency) {
  if (currency?.token) {
    if (currency.token.short) {
      return hexToAscii(currency.token.short)
    } else {
      return hexToAscii(currency.token.long)
    }
  } else {
    return 'Native'
  }
}
