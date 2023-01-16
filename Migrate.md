# Migrate smart contract

This docs provide the steps for migrating the contract after upgrades

## Re-deploy contract on testnet and mainnet

Check out the env in [`config.js`](./scripts/config.js) to ensure that the `CONTRACT_ACCOUNT_ID` matches with the account id that current contract was deployed on testnet and mainnet. Also, the `ORACLE_ACCOUNT_ID` and `ORACLE_PROVIDER_ID` account should match with account id in [`oracle docs`](https://docs.fluxprotocol.org/docs/live-data-feeds/fpo-live-networks-and-pairs#near).

```js
const env = {
    WASM_PATH: "./target/res/bulk_sender.wasm",
    testnet: {
        CONTRACT_ACCOUNT_ID: "nearsend.testnet",
        ORACLE_ACCOUNT_ID: "fpo.opfilabs.testnet",
        ORACLE_PROVIDER_ID: "opfilabs.testnet",
    },
    mainnet: {
        CONTRACT_ACCOUNT_ID: "bulksender.near",
        ORACLE_ACCOUNT_ID: "fpo.opfilabs.mainnet",
        ORACLE_PROVIDER_ID: "opfilabs.mainnet",
    }
};
```

Make sure your account is logged in through your near-cli

```bash
near login
```

Run the js script [`migrate.testnet.js`](./scripts/migrate.testnet.js) and [`migrate.mainnet.js`](./scripts/migrate.mainnet.js) for contract migration on testnet and mainnet accordingly. In the script, the following actions will be executed.
* Build the smart contract.
* Re-deploy the compiled contract on the current contract account id. 
* Call the `migrate` method to update the state of the contract.
* Call the `set_oracle` method to update `oracle_account_id` and `oracle_provider_id` in contract state.

For migration on testnet, run command below.

```bash
node scripts/migrate.testnet.js
```

For migration on mainnet, run command below.

```bash
node scripts/migrate.mainnet.js
```

### Nearsend Fees

Nearsend charges $0.05 per address.

```rust,no_run
const FEE_USD_PER_ADDRESS_DECIMAL_OFFSET: u32 = 1;
const FEE_USD_PER_ADDRESS_VALUE: u128 = 1;
```

To edit the service fee per address, we need to edit two variables `FEE_USD_PER_ADDRESS_DECIMAL_OFFSET` and `FEE_USD_PER_ADDRESS_VALUE`. So the USD fee per address will be calculated as follow.

```
FEE_USD_PER_ADDRESS_VALUE * 10 ^ (FLUX_USD_DECIMAL - FEE_USD_PER_ADDRESS_DECIMAL_OFFSET)
```

The `FLUX_USD_DECIMAL` is the base decimal for USD, check [`oracle docs`](https://docs.fluxprotocol.org/docs/live-data-feeds/fpo-live-networks-and-pairs#near) for more information. For example, if we want to have the service fee equal `$0.05` per address, we just need to change `FEE_USD_PER_ADDRESS_DECIMAL_OFFSET` value to `2`

```rust,no_run
const FEE_USD_PER_ADDRESS_DECIMAL_OFFSET: u32 = 2;
const FEE_USD_PER_ADDRESS_VALUE: u128 = 5;
```

Then the USD fee will be calculated as below:

```
usd_per_address = FEE_USD_PER_ADDRESS_VALUE * 10 ^ (FLUX_USD_DECIMAL - FEE_USD_PER_ADDRESS_DECIMAL_OFFSET)
                = 5 * 10 ^ (8 - 2)
                = 5 * 10 ^ 6 (~ $0.05)
```