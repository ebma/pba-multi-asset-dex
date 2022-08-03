import React, { useEffect, useState } from 'react'
import { Card, Grid } from 'semantic-ui-react'
import { buildCurrency } from './lib/utils'
import { useSubstrateState } from './substrate-lib'

export default function Main(props) {
  const { api, currentAccount } = useSubstrateState()
  const [balances, setBalances] = useState([])

  const balanceNative = balances.at(0) ? balances[0].free : 0
  const balanceUSDC = balances.at(1) ? balances[1].free : 0
  const balanceEURT = balances.at(2) ? balances[2].free : 0
  const balanceEURTUSDC = balances.at(3) ? balances[3].free : 0
  const balanceUSDCEURT = balances.at(4) ? balances[4].free : 0

  useEffect(() => {
    let unsubscribeAll = null

    if (!currentAccount) return

    api.query.tokens.accounts
      .multi(
        [
          [currentAccount.address, buildCurrency('native')],
          [currentAccount.address, buildCurrency('USDC')],
          [currentAccount.address, buildCurrency('EURT')],
          [currentAccount.address, buildCurrency('EURTUSDC')],
          [currentAccount.address, buildCurrency('USDCEURT')],
        ],
        balances => {
          let balancesMap = balances.map(balance => balance.toHuman())
          setBalances(balancesMap)
        }
      )
      .then(unsub => {
        unsubscribeAll = unsub
      })
      .catch(console.error)

    return () => unsubscribeAll && unsubscribeAll()
  }, [api, currentAccount, setBalances])

  return (
    <Grid.Column>
      <Card fluid>
        <Card.Content>
          <Card.Header>Balances</Card.Header>
          <Card.Meta>
            <Card.Description>
              Account: {currentAccount?.meta?.name} ({currentAccount?.address})
            </Card.Description>
          </Card.Meta>
          <Card.Description>Native: {balanceNative}</Card.Description>
          <Card.Description>EURT: {balanceEURT}</Card.Description>
          <Card.Description>USDC: {balanceUSDC}</Card.Description>
          {balanceEURTUSDC != 0 && (
            <Card.Description>EURTUSDC: {balanceEURTUSDC}</Card.Description>
          )}
          {balanceUSDCEURT != 0 && (
            <Card.Description>USDCEURT: {balanceUSDCEURT}</Card.Description>
          )}
        </Card.Content>
      </Card>
    </Grid.Column>
  )
}
