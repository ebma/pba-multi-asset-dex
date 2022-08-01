# Multi-token DEX

(Final project for Polkadot Blockchain Academy)

___

## Task

Create a simple multi-token DEX

- Create a simple multi-assets pallet (or use the existing one).
- Create a Uniswap style DEX to allow users to trustlessly exchange tokens with one another.
    - Be sure to implement Liquidity rewards.
    - Expose an API which acts as a “price oracle” based on the existing liquidity pools.
- Add a simple NFT pallet (like the kitties pallet we will do in class)
    - Allow users to mint or buy/sell new kitties with any token.

## Implementation

Liquidity rewards are implemented by the following logic:

- When a user executes a swap, he has to pay a fee. That is, the user has to pay a little more of Asset_A to receive
  Asset_B.
- Because users have to pay always a little more of one asset than they receive of the other, the existing liquidity
  providers profit for each swap that is happening on the pool.
- The fee can be set for every pool individually on creation.

Limitations:
- The fee of each pool cannot be changed.