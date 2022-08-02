
export function hexToAscii (hex) {
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
export function buildPrice(amount, currency)  {
  if (!currency) {
    return [amount, 'native']
  }

  let token
  if (currency.length === 4) {
    token = {
      token: {
        short: currency,
      },
    }
  } else if (currency.length === 8) {
    token = {
      token: {
        long: currency,
      },
    }
  } else {
    token = 'native'
  }
  let price = [amount, token]
  return price
}
