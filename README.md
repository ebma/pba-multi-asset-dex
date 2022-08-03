# Multi-token DEX

(Project for the Polkadot Blockchain Academy)

___

## Task

Create a simple multi-token DEX

- Create a simple multi-assets pallet (or use the existing one).
- Create a Uniswap style DEX to allow users to trustlessly exchange tokens with one another.
    - Be sure to implement Liquidity rewards.
    - Expose an API which acts as a “price oracle” based on the existing liquidity pools.
- Add a simple NFT pallet (like the kitties pallet we will do in class)
    - Allow users to mint or buy/sell new kitties with any token.

## Frontend User Interface

This project contains a React-based frontend user interface.
The frontend is forked from
the [substrate-front-end-template](https://github.com/substrate-developer-hub/substrate-front-end-template/tree/main).
Components for interacting with the DEX and NFT pallet were added to it.
For instructions on how to run it, check out the [README.md](/front-end/README.md) file.

## Node Template

This project contains a standalone Substrate node.
It is forked from
the [substrate-node-template](https://github.com/substrate-developer-hub/substrate-node-template/tree/main).

It is configured to use `CurrencyID::Native` for paying the transaction fees.
It also limits the data a unique item can hold to 255 bytes.

To run the node in development mode use:

```
cargo run --release -- -- dev
```

## Tests

To run the tests use:

```
cargo test
```

or to speed up compilation time and only run the tests of the pallets use:

```
cargo test --package pallet-dex
cargo test --package pallet-nft
```

## Implementation

### Multi Assets pallet

The pallet used for multi-asset support is [orml_tokens](https://docs.rs/orml-tokens/latest/orml_tokens/).

### DEX pallet

The `CurrencyId` used in both the pallet mocks and the node-runtime is defined
in [primitives/src/lib.rs](/primitives/src/lib.rs).
The `CurrencyId` is either the `Native` variant or a `Token` that itself can be either `Short` or `Long`.
The idea is that the `Native` variant is used as the native currency of the chain (e.g. to pay fees) and the `Token`
variant is used for other assets.
There is a `Long` variant that is double the size of the `Short` variant so that it can be used for deriving the
liquidity token of a pool by simply concatenating the IDs of the short tokens.

The DEX offers methods to create a new pool for a given pair of assets, provide/remove liquidity to a pool, and
buy/sell/swap assets on a pool.
Buy and sell can be considered convenience functions as they also use the swap function under-the-hood but offer an
easier interface for a user.

#### Automated Market Maker

The calculations of the AMM are based on the [Uniswap v2](https://docs.uniswap.org/protocol/V2/introduction) smart
contracts.

#### Liquidity rewards

Liquidity rewards are implemented by the following logic:

- When a user executes a swap, he has to pay a fee. That is, the user has to pay a little more of one asset to receive
  the other asset than the exchange rate would dictate.
- Because users always have to pay a little more of one asset, the existing liquidity
  providers profit for each swap that is happening on the pool, as the pool always grows after a swap happened.
- The fee can be set for every pool individually on creation.

#### Price Oracle

The pallet generates a unique account ID for every pool that is created.
This account ID is stored on-chain in the `PoolAccounts` map of the pallet.
The account ID can be used to check the balances/reserves of a pool by querying the balances with the `orml_tokens`
pallet.
The price/exchange rate of assets in a pool can be derived from these reserves.

#### Limitations / Considerations

- The fee of each pool cannot be changed.
- Multiple pools can be created for the same asset pair
    - This is not ideal, but I did not want to iterate over the whole map of pools to find if there already is a pool
      for the given asset pair because this is quite costly.
    - Because of how the liquidity token is derived, there can be the same LP token for multiple pools. But this is more
      a limitation of how the LP token derivation was configured in the runtime.
      This derivation is passed into the pallet so this issue can be mitigated by choosing different CurrencyIDs and
      conversion functions without having to touch the implementation of the pallet itself.
    - It's also possible to create a pool that holds the 'native' asset as one token, but the liquidity token derivation
      also does not really work in this case, so this is discouraged.

### NFT pallet

The NFT pallet is a simple pallet that allows users to mint or buy/sell unique items with any token.
It is based on
the [kitties pallet](https://github.com/substrate-developer-hub/substrate-front-end-template/blob/tutorials/solutions/kitties/src/Kitties.js)
but instead of generating random items and breeding them, the user can mint new items with custom data.
The length of the data associated to a unique item is limited.
This can be configured by changing `StringLimit` parameter of the pallet's Config.

#### Limitations / Considerations

- The user has to manually specify the ID of an item when minting it. This is because the ItemID is passed to the pallet
  and I want to allow it to be lots of different things, which makes it harder for the pallet to auto-generate an ID.
  The reason for this
  is that it should be up for the runtime to choose the type of the ItemID because it might want to give it a special
  meaning.
- Only the id of an item has to be unique, there can be multiple items holding the same associated data. This also was
  an deliberate choice because there might be cases where a user wants to buy/sell multiple items with the same data.
  The ID makes it unique anyways.

