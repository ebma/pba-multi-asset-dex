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
        let ids = [];
        entries.forEach(([key, exposure]) => {
          console.log('     exposure:', exposure.toHuman());
          let id = exposure.toHuman()["id"];
          ids.push(id);
        });
        setItemIds(ids)
      })
    }

    asyncFetch()

    return () => {
      unsub && unsub()
    }
  }

  console.log("uniqueItems", uniqueItems)

  const subscribeUniqueItems = () => {
    let unsub = null

    const asyncFetch = async () => {
      unsub = await api.query.nfts.uniqueItems.multi(
        itemIds,
        items => {
          const itemsMap = items.map(item => parseItem(item.unwrap()))
          setUniqueItems(itemsMap)
        }
      )
    }

    asyncFetch()

    return () => {
      unsub && unsub()
    }
  }

  useEffect(subscribeCount, [api, keyring])
  useEffect(subscribeUniqueItems, [api, keyring, itemIds])

  return (
    <Grid.Column width={16}>
      <h1>UniqueItems</h1>
      <UniqueItemCards uniqueItems={uniqueItems} setStatus={setStatus} />
      <Form style={{ margin: '1em 0' }}>
        <Form.Field style={{ textAlign: 'center' }}>
          <TxButton
            label="Create Kitty"
            type="SIGNED-TX"
            setStatus={setStatus}
            attrs={{
              palletRpc: 'nfts',
              callable: 'createUniqueItem',
              inputParams: [],
              paramFields: [],
            }}
          />
        </Form.Field>
      </Form>
      <div style={{ overflowWrap: 'break-word' }}>{status}</div>
    </Grid.Column>
  )
}
