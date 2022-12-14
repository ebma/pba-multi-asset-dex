import React, { useEffect, useState } from 'react'
import { Card, Divider, Form, Grid } from 'semantic-ui-react'
import { buildCurrency, currencyToString } from './lib/utils'

import { useSubstrateState } from './substrate-lib'
import { TxButton } from './substrate-lib/components'

const parseItem = ({ owner, pair, lpToken, fee }) => ({
  owner,
  lpToken: lpToken.toJSON(),
  fee,
  pair: pair.toJSON(),
})

function Pool(props) {
  const { id, account, tokenQuery } = props
  const { owner, pair, lpToken, fee } = props.pool
  const [balanceMap, setBalanceMap] = useState([])
  const [status, setStatus] = useState('')

  const subscribePoolBalances = () => {
    let unsub = null

    if (!account) return

    const asyncFetch = async () => {
      unsub = await tokenQuery.multi(
        [
          [account, pair.tokenA],
          [account, pair.tokenB],
        ],
        items => {
          const itemsMap = items.map(item => item.toJSON())
          setBalanceMap(itemsMap)
        }
      )
    }

    asyncFetch()

    return () => {
      unsub && unsub()
    }
  }

  useEffect(subscribePoolBalances, [account, pair, tokenQuery])

  const balanceA = balanceMap.at(0) ? balanceMap[0].free : 0
  const balanceB = balanceMap.at(1) ? balanceMap[1].free : 0

  const [addLiquidityCurrency, setAddLiquidityCurrency] = useState('')
  const [addLiquidityAmount, setAddLiquidityAmount] = useState('')
  const [removeLiquidityAmount, setRemoveLiquidityAmount] = useState('')
  const [swapCurrency, setSwapCurrency] = useState('')
  const [swapAmount, setSwapAmount] = useState('')

  return (
    <Card fluid>
      <Card.Content>
        <Card.Header>Pool #{id}</Card.Header>
        <Card.Meta>
          <Card.Description>Account: {account}</Card.Description>
        </Card.Meta>
        <Card.Description>
          Pair: {balanceA} {currencyToString(pair.tokenA)} - {balanceB}{' '}
          {currencyToString(pair.tokenB)}
        </Card.Description>
        <Card.Description>
          LP Token: {currencyToString(lpToken)}
        </Card.Description>
        <Card.Description>Fee: {fee.toHuman()}</Card.Description>
        <Card.Description>Owner: {owner.toHuman()}</Card.Description>

        <Divider horizontal>Add Liquidity</Divider>
        <Form style={{ margin: '1em 0' }}>
          <Form.Group widths="equal" style={{ textAlign: 'center' }}>
            <Form.Input
              fluid
              label="Amount"
              style={{ flexGrow: 1 }}
              value={addLiquidityAmount}
              onChange={e => setAddLiquidityAmount(e.target.value)}
            />
            <Form.Input
              fluid
              label="Token"
              placeholder="USDC"
              style={{ flexGrow: 1 }}
              value={addLiquidityCurrency}
              onChange={e => setAddLiquidityCurrency(e.target.value)}
            />
            <TxButton
              label="Add Liquidity"
              type="SIGNED-TX"
              setStatus={setStatus}
              attrs={{
                palletRpc: 'dex',
                callable: 'addLiquidity',
                inputParams: [
                  id,
                  addLiquidityAmount,
                  buildCurrency(addLiquidityCurrency),
                ],
                paramFields: [true, true, true],
              }}
            />
          </Form.Group>
        </Form>

        <Divider horizontal>Remove Liquidity</Divider>
        <Form style={{ margin: '1em 0' }}>
          <Form.Group widths="equal" style={{ textAlign: 'center' }}>
            <Form.Input
              fluid
              label="Amount"
              style={{ flexGrow: 1 }}
              value={removeLiquidityAmount}
              onChange={e => setRemoveLiquidityAmount(e.target.value)}
            />
            <TxButton
              label="Remove Liquidity"
              type="SIGNED-TX"
              setStatus={setStatus}
              attrs={{
                palletRpc: 'dex',
                callable: 'removeLiquidity',
                inputParams: [
                  id,
                  removeLiquidityAmount,
                ],
                paramFields: [true, true],
              }}
            />
          </Form.Group>
        </Form>

        <Divider horizontal>Swap</Divider>
        <Form style={{ margin: '1em 0' }}>
          <Form.Group widths="equal" style={{ textAlign: 'center' }}>
            <Form.Input
              fluid
              label="Amount"
              style={{ flexGrow: 1 }}
              value={swapAmount}
              onChange={e => setSwapAmount(e.target.value)}
            />
            <Form.Input
              fluid
              label="Token"
              placeholder="USDC"
              style={{ flexGrow: 1 }}
              value={swapCurrency}
              onChange={e => setSwapCurrency(e.target.value)}
            />
            <TxButton
              label="Buy"
              type="SIGNED-TX"
              setStatus={setStatus}
              attrs={{
                palletRpc: 'dex',
                callable: 'buy',
                inputParams: [id, buildCurrency(swapCurrency), swapAmount],
                paramFields: [true, true, true],
              }}
            />
            <TxButton
              label="Sell"
              type="SIGNED-TX"
              setStatus={setStatus}
              attrs={{
                palletRpc: 'dex',
                callable: 'sell',
                inputParams: [id, buildCurrency(swapCurrency), swapAmount],
                paramFields: [true, true, true],
              }}
            />
          </Form.Group>
        </Form>
        <div style={{ overflowWrap: 'break-word' }}>{status}</div>
      </Card.Content>
    </Card>
  )
}

export default function Dex(props) {
  const { api, currentAccount, keyring } = useSubstrateState()
  const [poolIds, setPoolIds] = useState([])
  const [poolAccounts, setPoolAccounts] = useState([])
  const [pools, setPools] = useState([])
  const [status, setStatus] = useState('')

  const tokenQuery = React.useMemo(
    () => api.query.tokens.accounts,
    [api.query.tokens]
  )

  const subscribeCount = () => {
    let unsub = null

    const asyncFetch = async () => {
      unsub = await api.query.dex.poolCount(async count => {
        // fetch pool ids
        let entries = await api.query.dex.pools.entries()
        let ids = []
        entries.forEach(([key, exposure]) => {
          let id = key.toHuman()
          ids.push(id)
        })
        setPoolIds(ids)

        // fetch pool accounts
        entries = await api.query.dex.poolAccounts.entries()
        let accounts = []
        entries.forEach(([key, exposure]) => {
          let accountID = exposure.toHuman()
          accounts.push(accountID)
        })
        setPoolAccounts(accounts)
      })
    }

    asyncFetch()

    return () => {
      unsub && unsub()
    }
  }

  const subscribeUniqueItems = () => {
    let unsub = null

    const asyncFetch = async () => {
      unsub = await api.query.dex.pools.multi(poolIds, items => {
        const itemsMap = items.map(item => parseItem(item.unwrap()))
        setPools(itemsMap)
      })
    }

    asyncFetch()

    return () => {
      unsub && unsub()
    }
  }

  useEffect(subscribeCount, [api, keyring])
  useEffect(subscribeUniqueItems, [api, keyring, poolIds])

  const [tokenA, setTokenA] = useState('')
  const [tokenB, setTokenB] = useState('')
  const [fee, setFee] = useState('')

  const buildPoolCreationParams = () => {
    let owner = currentAccount?.address
    let pair = {
      token_a: buildCurrency(tokenA),
      token_b: buildCurrency(tokenB),
    }
    return {
      owner,
      pair,
      fee,
    }
  }

  return (
    <Grid.Column width={16}>
      <h1>Dex</h1>
      {poolIds.length === 0 && <span>No pools yet</span>}
      {pools.map((pool, index) => (
        <Pool
          account={poolAccounts[index]}
          id={poolIds[index]}
          pool={pool}
          tokenQuery={tokenQuery}
        />
      ))}
      <Form style={{ margin: '1em 0' }}>
        <Form.Group widths="equal" style={{ textAlign: 'center' }}>
          <Form.Input
            fluid
            label="Token A"
            value={tokenA}
            placeholder="EURT"
            onChange={e => setTokenA(e.target.value)}
            style={{ flexGrow: 1 }}
          />
          <Form.Input
            fluid
            label="Token B"
            value={tokenB}
            placeholder="USDC"
            onChange={e => setTokenB(e.target.value)}
            style={{ flexGrow: 1 }}
          />
          <Form.Input
            fluid
            label="Fee in Permill"
            placeholder="30000 -> 3%"
            value={fee}
            onChange={e => setFee(e.target.value)}
            style={{ flexGrow: 1 }}
          />
          <TxButton
            label="Create Pool"
            type="SIGNED-TX"
            setStatus={setStatus}
            attrs={{
              palletRpc: 'dex',
              callable: 'createPool',
              inputParams: [buildPoolCreationParams()],
              paramFields: [true],
            }}
          />
        </Form.Group>
      </Form>
      <div style={{ overflowWrap: 'break-word' }}>{status}</div>
    </Grid.Column>
  )
}
