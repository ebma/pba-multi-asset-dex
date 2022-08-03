import React, { useEffect, useState } from 'react'
import { Form, Grid } from 'semantic-ui-react'

import { useSubstrateState } from './substrate-lib'
import { TxButton } from './substrate-lib/components'

import UniqueItemCards from './UniqueItemCards'

const parseItem = ({ id, data, price, owner }) => ({
  id,
  data,
  price: price.toJSON(),
  owner: owner.toJSON(),
})

export default function UniqueItems(props) {
  const { api, keyring } = useSubstrateState()
  const [itemIds, setItemIds] = useState([])
  const [uniqueItems, setUniqueItems] = useState([])
  const [status, setStatus] = useState('')

  const subscribeCount = () => {
    let unsub = null

    const asyncFetch = async () => {
      unsub = await api.query.nfts.countForUniqueItems(async count => {
        const entries = await api.query.nfts.uniqueItems.entries()
        let ids = []
        entries.forEach(([key, exposure]) => {
          let id = exposure.toHuman()['id']
          ids.push(id)
        })
        setItemIds(ids)
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
      unsub = await api.query.nfts.uniqueItems.multi(itemIds, items => {
        const itemsMap = items.map(item => parseItem(item.unwrap()))
        setUniqueItems(itemsMap)
      })
    }

    asyncFetch()

    return () => {
      unsub && unsub()
    }
  }

  useEffect(subscribeCount, [api, keyring])
  useEffect(subscribeUniqueItems, [api, keyring, itemIds])

  const [newItemID, setNewItemID] = useState('')
  const [newItemData, setNewItemData] = useState('')

  return (
    <Grid.Column width={16}>
      <h1>UniqueItems</h1>
      <UniqueItemCards uniqueItems={uniqueItems} setStatus={setStatus} />
      <Form style={{ margin: '1em 0' }}>
        <Form.Group widths="equal" style={{ textAlign: 'center' }}>
          <Form.Input
            fluid
            label="ID"
            value={newItemID}
            type="number"
            onChange={e => setNewItemID(e.target.value)}
            style={{ flexGrow: 1 }}
          />
          <Form.Input
            fluid
            label="Data"
            value={newItemData}
            onChange={e => setNewItemData(e.target.value)}
            style={{ flexGrow: 1 }}
          />
          <TxButton
            label="Create Item"
            type="SIGNED-TX"
            setStatus={setStatus}
            attrs={{
              palletRpc: 'nfts',
              callable: 'createUniqueItem',
              inputParams: [newItemID, newItemData],
              paramFields: [true, true],
            }}
          />
        </Form.Group>
      </Form>
      <div style={{ overflowWrap: 'break-word' }}>{status}</div>
    </Grid.Column>
  )
}
