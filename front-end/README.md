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

## Miscellaneous

- Polkadot-js API and related crypto libraries depend
  on [`BigInt`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt) that is only
  supported by modern browsers. To ensure that react-scripts properly transpile your webapp code, update
  the `package.json` file:

  ```json
  {
    "browserslist": {
      "production": [
        ">0.2%",
        "not ie <= 99",
        "not android <= 4.4.4",
        "not dead",
        "not op_mini all"
      ]
    }
  }
  ```

  Refer
  to [this doc page](https://github.com/vacp2p/docs.wakuconnect.dev/blob/develop/content/docs/guides/07_reactjs_relay.md)
  .
