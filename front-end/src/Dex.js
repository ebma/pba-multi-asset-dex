import React, { useEffect, useState } from 'react'
import { Card, Form, Grid } from 'semantic-ui-react'
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
  const { id, account} = props
  const { owner, pair, lpToken, fee } = props.pool


  return (
    <Card fluid>
      <Card.Content>
        <Card.Header>Pool #{id}</Card.Header>
        <Card.Meta>
          <span>TODO</span>
        </Card.Meta>
        <Card.Description>Pair: {currencyToString(pair.tokenA)} - {currencyToString(pair.tokenB)}</Card.Description>
        <Card.Description>LP Token: {currencyToString(lpToken)}</Card.Description>
        <Card.Description>Fee: {fee.toHuman()}</Card.Description>
        <Card.Description>Owner: {owner.toHuman()}</Card.Description>
        <Card.Description>Pool Account: {account}</Card.Description>
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

  console.log("api", api.query.dex)
  console.log("poolAccounts", poolAccounts)

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
    let owner = currentAccount?.address;
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
        <Pool pool={pool} id={poolIds[index]} account={poolAccounts[index]}/>
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
