import React from 'react'
import {
  Button,
  Card,
  Grid,
  Message,
  Modal,
  Form,
  Label,
} from 'semantic-ui-react'

import UniqueItemAvatar from './UniqueItemAvatar'
import { useSubstrateState } from './substrate-lib'
import { TxButton } from './substrate-lib/components'
import { buildPrice, priceToString } from './lib/utils'

// --- Transfer Modal ---

const TransferModal = props => {
  const { item, setStatus } = props
  const [open, setOpen] = React.useState(false)
  const [formValue, setFormValue] = React.useState({})

  const formChange = key => (ev, el) => {
    setFormValue({ ...formValue, [key]: el.value })
  }

  const confirmAndClose = unsub => {
    setOpen(false)
    if (unsub && typeof unsub === 'function') unsub()
  }

  return (
    <Modal
      onClose={() => setOpen(false)}
      onOpen={() => setOpen(true)}
      open={open}
      trigger={
        <Button basic color="blue">
          Transfer
        </Button>
      }
    >
      <Modal.Header>UniqueItem Transfer</Modal.Header>
      <Modal.Content>
        <Form>
          <Form.Input fluid label="UniqueItem ID" readOnly value={item.id} />
          <Form.Input
            fluid
            label="Receiver"
            placeholder="Receiver Address"
            onChange={formChange('target')}
          />
        </Form>
      </Modal.Content>
      <Modal.Actions>
        <Button basic color="grey" onClick={() => setOpen(false)}>
          Cancel
        </Button>
        <TxButton
          label="Transfer"
          type="SIGNED-TX"
          setStatus={setStatus}
          onClick={confirmAndClose}
          attrs={{
            palletRpc: 'nfts',
            callable: 'transfer',
            inputParams: [formValue.target, item.id],
            paramFields: [true, true],
          }}
        />
      </Modal.Actions>
    </Modal>
  )
}

// --- Set Price ---

const SetPrice = props => {
  const { item, setStatus } = props
  const [open, setOpen] = React.useState(false)
  const [formValue, setFormValue] = React.useState({})

  const formChange = key => (ev, el) => {
    setFormValue({ ...formValue, [key]: el.value })
  }

  const confirmAndClose = unsub => {
    setOpen(false)
    if (unsub && typeof unsub === 'function') unsub()
  }

  return (
    <Modal
      onClose={() => setOpen(false)}
      onOpen={() => setOpen(true)}
      open={open}
      trigger={
        <Button basic color="blue">
          Set Price
        </Button>
      }
    >
      <Modal.Header>Set UniqueItem Price</Modal.Header>
      <Modal.Content>
        <Form>
          <Form.Input fluid label="UniqueItem ID" readOnly value={item.id} />
          <Form.Input
            fluid
            label="Price"
            placeholder="Enter Price"
            type="number"
            onChange={formChange('amount')}
          />
          <Form.Input
            fluid
            label="Currency (either 'native' or a 4 or 8 letters currency e.g. 'EURT' or 'EURTUSDC')"
            placeholder="Enter Currency"
            onChange={formChange('currency')}
          />
        </Form>
      </Modal.Content>
      <Modal.Actions>
        <Button basic color="grey" onClick={() => setOpen(false)}>
          Cancel
        </Button>
        <TxButton
          label="Set Price"
          type="SIGNED-TX"
          setStatus={setStatus}
          onClick={confirmAndClose}
          attrs={{
            palletRpc: 'nfts',
            callable: 'setPrice',
            // inputParams: [item.id, [formValue.amount, formValue.currency]],
            inputParams: [
              item.id,
              buildPrice(formValue.amount, formValue.currency),
            ],
            // inputParams: [item.id, [formValue.amount, ['token', [0, 'EURT']]]],
            paramFields: [true, true],
          }}
        />
      </Modal.Actions>
    </Modal>
  )
}

// --- Buy UniqueItem ---

const BuyUniqueItem = props => {
  const { item, setStatus } = props
  const [open, setOpen] = React.useState(false)

  const confirmAndClose = unsub => {
    setOpen(false)
    if (unsub && typeof unsub === 'function') unsub()
  }

  if (!item.price) {
    return <></>
  }

  return (
    <Modal
      onClose={() => setOpen(false)}
      onOpen={() => setOpen(true)}
      open={open}
      trigger={
        <Button basic color="green">
          Buy UniqueItem
        </Button>
      }
    >
      <Modal.Header>Buy UniqueItem</Modal.Header>
      <Modal.Content>
        <Form>
          <Form.Input fluid label="UniqueItem ID" readOnly value={item.id} />
          <Form.Input
            fluid
            label="Price"
            readOnly
            value={priceToString(item.price)}
          />
        </Form>
      </Modal.Content>
      <Modal.Actions>
        <Button basic color="grey" onClick={() => setOpen(false)}>
          Cancel
        </Button>
        <TxButton
          label="Buy UniqueItem"
          type="SIGNED-TX"
          setStatus={setStatus}
          onClick={confirmAndClose}
          attrs={{
            palletRpc: 'nfts',
            callable: 'buyUniqueItem',
            inputParams: [item.id, item.price],
            paramFields: [true, true],
          }}
        />
      </Modal.Actions>
    </Modal>
  )
}

// --- About UniqueItem Card ---

const UniqueItemCard = props => {
  const { item, setStatus } = props
  const { id = null, owner = null, data = null, price = null } = item
  const { currentAccount } = useSubstrateState()
  const isSelf = currentAccount.address === item.owner

  return (
    <Card>
      {isSelf && (
        <Label as="a" floating color="teal">
          Mine
        </Label>
      )}
      <UniqueItemAvatar id={id.toU8a()} />
      <Card.Content>
        <Card.Meta style={{ fontSize: '.9em', overflowWrap: 'break-word' }}>
          ID: {id.toHuman()}
        </Card.Meta>
        <Card.Description>
          <p style={{ overflowWrap: 'break-word' }}>Data: {data.toHuman()}</p>
          <p style={{ overflowWrap: 'break-word' }}>Owner: {owner}</p>
          <p style={{ overflowWrap: 'break-word' }}>
            Price: {price != null ? priceToString(price) : 'Not For Sale'}
          </p>
        </Card.Description>
      </Card.Content>
      <Card.Content extra style={{ textAlign: 'center' }}>
        {owner === currentAccount.address ? (
          <>
            <SetPrice item={item} setStatus={setStatus} />
            <TransferModal item={item} setStatus={setStatus} />
          </>
        ) : (
          <>
            <BuyUniqueItem item={item} setStatus={setStatus} />
          </>
        )}
      </Card.Content>
    </Card>
  )
}

const UniqueItemCards = props => {
  const { uniqueItems, setStatus } = props

  if (uniqueItems.length === 0) {
    return (
      <Message info>
        <Message.Header>
          No UniqueItem found here... Create one now!&nbsp;
          <span role="img" aria-label="point-down">
            👇
          </span>
        </Message.Header>
      </Message>
    )
  }

  return (
    <Grid columns={3}>
      {uniqueItems.map((item, i) => (
        <Grid.Column key={`item-${i}`}>
          <UniqueItemCard item={item} setStatus={setStatus} />
        </Grid.Column>
      ))}
    </Grid>
  )
}

export default UniqueItemCards
