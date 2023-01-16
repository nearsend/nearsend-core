# Nearsend

Contract code for [Nearsend](https://nearsend.io/)

### Prerequisites

1. Make sure Rust is installed per the prerequisites in [`near-sdk-rs`](https://github.com/near/near-sdk-rs#pre-requisites)
2. Ensure `near-cli` is installed by running `near --version`. If not installed, install with: `npm install -g near-cli`

### Build contract

To build run:
```=bash
bash shell-script/build.sh
```

### Deployment Steps

This smart contract will get deployed to your NEAR account. For this example, please create a new NEAR account. Because NEAR allows the ability to upgrade contracts on the same account, initialization functions must be cleared. If you'd like to run this contract on a NEAR account that has had prior contracts deployed, please use the `near-cli` command `near delete`, and then recreate it in Wallet. To create (or recreate) an account, please follow the directions on [NEAR Wallet](https://wallet.near.org/).

Switch to `mainnet`. You can skip this step to use `testnet` as a default network.

```=bash
export NEAR_ENV=mainnet
```

In the project root, log in to your newly created account  with `near-cli` by following the instructions after this command:

```=bash
near login
```

To make this tutorial easier to copy/paste, we're going to set an environment variable for your account id. In the below command, replace `MY_ACCOUNT_NAME` with the account name you just logged in with, including the `.near`:

```=bash
ID=MY_ACCOUNT_NAME
```

You can tell if the environment variable is set correctly if your command line prints the account name after this command:

```=bash
echo $ID
```

Now we can deploy the compiled contract in this example to your account:

```=bash
near deploy --wasmFile target/res/bulk_sender.wasm --accountId $ID
```

Contract should be initialized before usage. To initialize the contract, change the oracle_id to your near account and use this command:

```=bash
near call $ID new '{"oracle_id": "YOUR_ORACLE_ACCOUNT_ID_HERE"}' --accountId $ID
```

### Test

Run all tests with this command:

```bash
cargo test
```

### Function Explanation

#### Call Methods

```rust,no_run
    /// A payable method that transfers near from `env::predecessor_account_id` to `receivers`.
    /// 
    /// Requirements:
    /// - The attached deposit must equal the total amount sent to all receivers.
    /// - Caller must pay service fee to get enough quota to transfer near.
    /// - The length of `receivers` and `amount` arguments must be the same.
    /// 
    /// Arguments:
    /// - `receivers` - a vec of all receivers' account ID.
    /// - `amount` - a vec of the amount of near sent to each receiver corresponding.
    pub fn distribute_near(&mut self, receivers: Vec<AccountId>, amount: Vec<U128>);

    /// Transfers positive `amount` of tokens from the `env::predecessor_account_id` to `receivers`.
    /// This function is executed when user call `ft_transfer_call` on a fungible token contracts. 
    /// See [ft_transfer_call](https://github.com/near/NEPs/blob/master/neps/nep-0141.md#ft_transfer_call).
    /// 
    /// Requirements:
    /// - `msg` argument must follow this format `"msg": "bob.testnet:20#alice.testnet:50"`. This means
    /// `bob.testnet` receive 20 tokens and `alice.testnet` receive 50 tokens.
    /// - Both `bob` and `alice` must register storage for token contract in advance.
    /// - `sender_id` balance must be greater or equal to the total amount sent to each receiver. 
    /// - `sender_id` must pay service fee to get enough quota to transfer near.
    /// 
    /// Arguments:
    /// - `sender_id`: the account id of sender.
    /// - `amount`: the amount of token that sender transfer to this contract by calling `ft_transfer_call`.
    /// - `msg` - a string message that includes receivers and amount.
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;

    /// A payable method that helps pay token storage fee for multiple accounts. 
    /// 
    /// Requirements:
    /// - The attached deposit must be equal to `min_fee * account_ids.len()`.
    /// - `min_fee` must be equal to the token contract's `StorageBalanceBounds.min`.
    /// - all of the `account_ids` must haven't paid for the token storage fee.
    /// 
    /// Arguments: 
    /// - `token_id`: token contract account id.
    /// - `account_ids`: the vec of all account that needs to pay token storage fee.
    /// - `min_fee`: the minimum fee needed to cover token storage for each account.
    pub fn batch_storage_deposit(
        &mut self,
        token_id: AccountId,
        account_ids: Vec<AccountId>,
        min_fee: U128,
    );

    /// A payable method to pay service fee. This method will increase
    /// the number of user quota to transfer near and tokens.
    /// 
    /// Requirements:
    /// - The difference between `estimated_fee` and the fee calculated 
    /// by the near price fetched from the oracle.
    /// 
    /// Arguments:
    /// - `estimated_fee` is the amount near equals 0.05 USD.
    #[payable]
    pub fn pay_service_fee(&mut self, estimated_fee: U128);

    /// Set the oracle account id and  oracle provider id.
    /// 
    /// Requirements:
    /// - The caller must be contract owners.
    /// - `oracle_account_id` and `oracle_provider_id` must be a valid near account.
    /// 
    /// Arguments:
    /// - `oracle_account_id`: the account id of oracle contract to query token price on-chain.
    /// - `oracle_provider_id`: the account id of provider that update the token price on oracle contract.
    pub fn set_oracle(
        &mut self,
        oracle_account_id: AccountId,
        oracle_provider_id: AccountId,
    ) -> (AccountId, AccountId)
```

#### View Methods

```rust,no_run
    /// Return the `owner_id` of the contract. When contract is initialized `owner_id` is set to 
    /// the account id that is contract is deployed on.
    pub fn owner_id(&self) -> AccountId;

    /// Return the number of quota for `account_id`. This quota is the total number of account that user can 
    /// transfer near and tokens to. For each successful transfer the total quota will be decreased by 1.
    pub fn get_account_quota(&self, account_id: &AccountId) -> u128;

    /// Return the oracle account id and oracle provider id. For more information, refers to FLux oracle docs.
    pub fn oracle(&self) -> (AccountId, AccountId);
```