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

  let token = buildToken(tokenSymbol)
  let price = [amount, token]
  return price
}

export function buildToken(tokenSymbol) {
  let token
  if (tokenSymbol.length === 4) {
    token = {
      token: {
        short: tokenSymbol,
      },
    }
  } else if (tokenSymbol.length === 8) {
    token = {
      token: {
        long: tokenSymbol,
      },
    }
  } else {
    token = 'native'
  }
  return token
}

export function priceToString(price) {
  if (price) {
    return `${price[0]} ${tokenToString(price[1].token)}`
  }
}

export function tokenToString(token) {
  if (token) {
    if (token.short) {
      return hexToAscii(token.short)
    } else {
      console.log("in else block of", token.long)
      return hexToAscii(token.long)
    }
  } else {
    return 'Native'
  }
}
