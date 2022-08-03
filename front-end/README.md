# Multi-Token Dex Frontend

This project is forked from
the [substrate-front-end-template](https://github.com/substrate-developer-hub/substrate-front-end-template/tree/main).

### Installation

```bash
npm install
```

### Troubleshooting

If you encounter dependency issues, try `yarn` instead.

### Usage

You can start the template in development mode to connect to a locally running node

```bash
npm start
```

You can also build the app in production mode,

```bash
npm build
```

and open `build/index.html` in your favorite browser.

### Navigating the page

#### DEX

You can find the balances of the currently selected user at the top.
This will show the `Native`, `EURT`, and `USDC` balances, as these are the only tokens that are configured in the node
runtime.
You should first create a pool for e.g. `EURT` and `USDC` and then add liquidity to it.

#### NFT

You can create a new unique item by specifying an ID (simple u128) and some data (string of up to 255 bytes).
Afterwards you can set a price for your unique item by clicking on "Set Price".
Enter an amount and a currency (either 'native', 'EURT' or 'USDC') and click "Set Price".
You could even use the liquidity token ('EURTUSDC' or 'USDCEURT' depending on how you created the pool) to set the
price.
You should then switch to another user using the select field at the top of the page.
Now, the buttons will change and you can buy the unique item that you just set for sale on the other account.
