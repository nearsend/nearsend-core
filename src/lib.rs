use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk::{
    env, ext_contract, log, near_bindgen, serde_json, AccountId, Balance, BorshStorageKey, Gas,
    PanicOnDefault, Promise, PromiseOrValue, PromiseResult, StorageUsage,
};

pub use crate::events::*;

mod events;

const NO_DEPOSIT: Balance = 0;
const ONE_NEAR: Balance = 1_000_000_000_000_000_000_000_000;
const GAS_FOR_TRANSFER_NEAR_CALLBACK: Gas = Gas(5_000_000_000_000);
const GAS_FOR_FT_TRANSFER: Gas = Gas(2_000_000_000_000);
const GAS_FOR_FT_TRANSFER_CALLBACK: Gas = Gas(3_000_000_000_000);
const GAS_FOR_STORAGE_DEPOSIT_CALLBACK: Gas = Gas(3_000_000_000_000);
const GAS_FOR_STORAGE_DEPOSIT: Gas = Gas(2_000_000_000_000);
const FEE_USD_PER_ADDRESS_DECIMAL_OFFSET: u32 = 2;
const FEE_USD_PER_ADDRESS_VALUE: u128 = 5;

#[ext_contract(ext_ft)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance;
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct OracleEntry {
    pub price: U128,
    pub decimals: u32,
    pub last_update: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    pub total: U128,
    pub available: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalanceBounds {
    pub min: U128,
    pub max: Option<U128>,
}

#[ext_contract(ext_ts)]
pub trait StorageManagement {
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance;

    fn storage_balance_of(&mut self, account_id: AccountId) -> Option<StorageBalance>;
}

#[ext_contract(ext_self)]
pub trait Handler {
    fn callback_storage_deposit(&self, account_ids: Vec<AccountId>, min_fee: U128);
    fn callback_transfer_near(&self, receivers: Vec<AccountId>, amount: Vec<U128>);
    fn callback_ft_transfer(&self, account_ids: Vec<AccountId>, amount: Vec<U128>) -> U128;
    fn callback_get_entry(&self, estimated_fee: U128, amount: Balance);
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldContract {
    pub owner_id: AccountId,  // admin Id
    pub oracle_id: AccountId, // oracleId
    pub balances: LookupMap<AccountId, u128>,
    pub account_storage_usage: StorageUsage,
    pub service_fee: U128,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    BalanceData,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId, // admin Id
    pub balances: LookupMap<AccountId, u128>,
    pub oracle_account_id: AccountId,
    pub oracle_provider_id: AccountId,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(oracle_account_id: AccountId, oracle_provider_id: AccountId) -> Self {
        let this = Self {
            owner_id: env::signer_account_id(),
            balances: LookupMap::new(StorageKey::BalanceData),
            oracle_account_id,
            oracle_provider_id,
        };
        this
    }

    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let old_state: OldContract = env::state_read().expect("failed");

        // Check the caller is authorized to update the code
        assert!(
            env::predecessor_account_id() == old_state.owner_id,
            "Only owner can update the code"
        );

        Self {
            owner_id: old_state.owner_id, // admin Id
            balances: old_state.balances,
            oracle_account_id: AccountId::new_unchecked("oracle_account_id".to_string()),
            oracle_provider_id: AccountId::new_unchecked("oracle_provider_id".to_string()),
        }
    }

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
    ) -> (AccountId, AccountId) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "only contract owner can set oracle"
        );
        self.oracle_account_id = oracle_account_id;
        self.oracle_provider_id = oracle_provider_id;
        (
            self.oracle_account_id.clone(),
            self.oracle_provider_id.clone(),
        )
    }

    /// Return the oracle account id and oracle provider id. For more information, refers to FLux oracle docs.
    pub fn oracle(&self) -> (AccountId, AccountId) {
        (
            self.oracle_account_id.clone(),
            self.oracle_provider_id.clone(),
        )
    }

    /// Return the `owner_id` of the contract. When contract is initialized `owner_id` is set to
    /// the account id that is contract is deployed on.
    pub fn owner_id(&self) -> AccountId {
        self.owner_id.clone()
    }

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
    #[payable]
    pub fn distribute_near(&mut self, receivers: Vec<AccountId>, amount: Vec<U128>) {
        let total_amount: Balance = amount.iter().map(|x| x.0).sum();
        assert_eq!(receivers.len(), amount.len(), "invalid parameters");
        assert_eq!(
            total_amount,
            env::attached_deposit(),
            "Not enough Near attached"
        );
        assert!(
            self.get_account_quota(&env::signer_account_id()) >= receivers.len() as u128,
            "Not enough quota for user"
        );
        let mut transfer_promise = Promise::new(receivers[0].clone()).transfer(amount[0].0);
        for i in 1..receivers.len() {
            transfer_promise =
                transfer_promise.and(Promise::new(receivers[i].clone()).transfer(amount[i].0));
        }
        self.decrease_account_quota(&env::signer_account_id(), receivers.len() as u128);
        transfer_promise.then(ext_self::callback_transfer_near(
            receivers,
            amount,
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_TRANSFER_NEAR_CALLBACK,
        ));
    }

    #[private]
    pub fn callback_transfer_near(&mut self, receivers: Vec<AccountId>, amount: Vec<U128>) {
        assert_eq!(env::promise_results_count(), receivers.len() as u64);
        let mut refund: U128 = 0.into();
        let mut total_failed_transfer: u128 = 0;
        for i in 0..receivers.len() {
            match env::promise_result(i as u64) {
                PromiseResult::NotReady => {
                    log!(" Transfer not ready ");
                }
                PromiseResult::Failed => {
                    refund = (refund.0 + amount[i].0).into();
                    total_failed_transfer += 1;
                }
                PromiseResult::Successful(_) => {}
            };
        }
        if total_failed_transfer > 0 {
            let refund_log: EventLog = EventLog {
                standard: EVENT_STANDARD_NAME.to_string(),
                version: EVENT_VERSION.to_string(),
                event: EventLogVariant::RefundNear(RefundNearLog {
                    refund_amount: refund.0.to_string(),
                    user_id: env::signer_account_id().to_string(),
                }),
            };
            env::log_str(&refund_log.to_string());
            Promise::new(env::signer_account_id()).transfer(refund.0);
            self.increase_account_quota(&env::signer_account_id(), total_failed_transfer);
        }
    }

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
    ) -> PromiseOrValue<U128> {
        if env::signer_account_id() != sender_id {
            env::panic_str("sender_id is not signer");
        }
        let receivers_info: Vec<&str> = msg.split("#").collect();
        let mut account_id_arr: Vec<AccountId> = Vec::new();
        let mut amount_arr: Vec<U128> = Vec::new();
        for (_, receiver_info) in receivers_info.iter().enumerate() {
            let mut info = receiver_info.split(":");
            let account_id = info.next().unwrap();
            let transfer_amount: &str = info.next().unwrap();
            account_id_arr.push(AccountId::new_unchecked(String::from(account_id)));
            amount_arr.push(transfer_amount.parse::<u128>().unwrap().into());
        }
        let total_amount: U128 = amount_arr.iter().map(|x| x.0).sum::<u128>().into();
        assert_eq!(total_amount, amount, "Not enough amount of token sent");
        assert!(
            self.get_account_quota(&sender_id) >= account_id_arr.len() as u128,
            "Not enough quota for user"
        );

        let mut ft_transfer_promise = ext_ft::ft_transfer(
            account_id_arr[0].clone(),
            amount_arr[0].clone(),
            Some(String::from("")),
            env::predecessor_account_id(), // contract account id
            1,                             // yocto NEAR to attach
            GAS_FOR_FT_TRANSFER,
        );
        for i in 1..amount_arr.len() {
            ft_transfer_promise = ft_transfer_promise.and(ext_ft::ft_transfer(
                account_id_arr[i].clone(),
                amount_arr[i].clone(),
                Some(String::from("")),
                env::predecessor_account_id(), // contract account id
                1,                             // yocto NEAR to attach
                GAS_FOR_FT_TRANSFER,
            ));
        }
        self.decrease_account_quota(&env::signer_account_id(), account_id_arr.len() as u128);
        ft_transfer_promise
            .then(ext_self::callback_ft_transfer(
                account_id_arr,
                amount_arr,
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_FOR_FT_TRANSFER_CALLBACK,
            ))
            .into()
    }

    #[private]
    pub fn callback_ft_transfer(&mut self, account_ids: Vec<AccountId>, amount: Vec<U128>) -> U128 {
        assert_eq!(env::promise_results_count(), account_ids.len() as u64);
        let mut refund: U128 = 0.into();
        let mut total_failed_transfer: u128 = 0;
        for i in 0..account_ids.len() {
            match env::promise_result(i as u64) {
                PromiseResult::NotReady => {
                    log!(" Transfer not ready ");
                }
                PromiseResult::Failed => {
                    refund = (refund.0 + amount[i].0).into();
                    total_failed_transfer += 1;
                }
                PromiseResult::Successful(_) => {}
            };
        }
        if total_failed_transfer > 0 {
            self.increase_account_quota(&env::signer_account_id(), total_failed_transfer);
        }
        refund
    }

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
    #[payable]
    pub fn batch_storage_deposit(
        &mut self,
        token_id: AccountId,
        account_ids: Vec<AccountId>,
        min_fee: U128,
    ) {
        assert_eq!(
            min_fee.0 * account_ids.len() as u128,
            env::attached_deposit(),
            "Not enough Near attached"
        );
        let mut storage_deposit_promise = ext_ft::storage_deposit(
            Option::from(account_ids[0].clone()),
            None,
            token_id.clone(),
            min_fee.0,
            GAS_FOR_STORAGE_DEPOSIT,
        );
        for i in 1..account_ids.len() {
            storage_deposit_promise = storage_deposit_promise.and(ext_ft::storage_deposit(
                Option::from(account_ids[i].clone()),
                None,
                token_id.clone(),
                min_fee.0,
                GAS_FOR_STORAGE_DEPOSIT,
            ));
        }
        storage_deposit_promise.then(ext_self::callback_storage_deposit(
            account_ids,
            min_fee,
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_STORAGE_DEPOSIT_CALLBACK,
        ));
    }

    #[private]
    pub fn callback_storage_deposit(&self, account_ids: Vec<AccountId>, min_fee: U128) {
        assert_eq!(env::promise_results_count(), account_ids.len() as u64);
        let mut refund: U128 = 0.into();
        for i in 0..account_ids.len() {
            match env::promise_result(i as u64) {
                PromiseResult::NotReady => {
                    log!(" Transfer not ready ");
                }
                PromiseResult::Failed => {
                    log!("storage registration for {} failed", account_ids[i]);
                    refund = (refund.0 + min_fee.0).into();
                }
                PromiseResult::Successful(_result) => {}
            };
        }
        if refund.0 > 0 {
            let refund_log: EventLog = EventLog {
                standard: EVENT_STANDARD_NAME.to_string(),
                version: EVENT_VERSION.to_string(),
                event: EventLogVariant::RefundNear(RefundNearLog {
                    refund_amount: refund.0.to_string(),
                    user_id: env::signer_account_id().to_string(),
                }),
            };
            env::log_str(&refund_log.to_string());
            Promise::new(env::signer_account_id()).transfer(refund.0);
        }
    }

    /// A payable method to pay service fee. This method will increase
    /// the number of user quota to transfer near and tokens.
    #[payable]
    pub fn pay_service_fee(&mut self, estimated_fee: U128) {
        Promise::new(AccountId::new_unchecked(self.oracle_account_id.to_string()))
            .function_call(
                "get_entry".to_string(),
                serde_json::to_vec(&json!({
                    "pair": "NEAR/USD".to_string(),
                    "provider": self.oracle_provider_id.to_string()
                }))
                .unwrap(),
                NO_DEPOSIT,
                Gas(5_000_000_000_000),
            )
            .then(ext_self::callback_get_entry(
                estimated_fee,
                env::attached_deposit(),
                env::current_account_id(),
                NO_DEPOSIT,
                GAS_FOR_STORAGE_DEPOSIT_CALLBACK,
            ));
    }

    #[private]
    pub fn callback_get_entry(&mut self, estimated_fee: U128, amount: Balance) {
        assert_eq!(env::promise_results_count(), 1, "This is a callback method");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => {}
            PromiseResult::Successful(result) => {
                let res = near_sdk::serde_json::from_slice::<OracleEntry>(&result).unwrap();
                let oracle_fee: U128 = (ONE_NEAR
                    * FEE_USD_PER_ADDRESS_VALUE
                    * 10u128.pow(res.decimals - FEE_USD_PER_ADDRESS_DECIMAL_OFFSET)
                    / res.price.0)
                    .into();
                log!(
                    "diff: {}, rate: {}, oracle_fee: {}, estimated_fee: {}",
                    oracle_fee.0.abs_diff(estimated_fee.0),
                    oracle_fee.0 / 10u128,
                    oracle_fee.0,
                    estimated_fee.0
                );
                if oracle_fee.0.abs_diff(estimated_fee.0) > oracle_fee.0 / 10u128 {
                    let refund_log: EventLog = EventLog {
                        standard: EVENT_STANDARD_NAME.to_string(),
                        version: EVENT_VERSION.to_string(),
                        event: EventLogVariant::RefundNear(RefundNearLog {
                            refund_amount: amount.to_string(),
                            user_id: env::signer_account_id().to_string(),
                        }),
                    };
                    env::log_str(&refund_log.to_string());
                    Promise::new(env::signer_account_id()).transfer(amount);
                } else {
                    let redundant_coin = amount % estimated_fee.0;
                    if redundant_coin != 0 {
                        Promise::new(env::signer_account_id()).transfer(redundant_coin);
                    }

                    let num_addr: u128 = amount / estimated_fee.0;
                    let current_quota: u128 = self.get_account_quota(&env::signer_account_id());
                    let pay_fee_log: EventLog = EventLog {
                        standard: EVENT_STANDARD_NAME.to_string(),
                        version: EVENT_VERSION.to_string(),
                        event: EventLogVariant::PayFee(PayFeeLog {
                            amount: amount.to_string(),
                            refund: redundant_coin.to_string(),
                            user_id: env::signer_account_id().to_string(),
                            old_quota: current_quota.to_string(),
                            new_quota: (current_quota + num_addr).to_string(),
                        }),
                    };
                    self.increase_account_quota(&env::signer_account_id(), num_addr);
                    env::log_str(&pay_fee_log.to_string());
                }
            }
        }
    }

    fn increase_account_quota(&mut self, account_id: &AccountId, num: u128) {
        let value = self.get_account_quota(account_id) + num;
        self.balances.insert(&account_id, &value);
    }

    fn decrease_account_quota(&mut self, account_id: &AccountId, num: u128) {
        let value = self.get_account_quota(account_id) - num;
        self.balances.insert(&account_id, &value);
    }

    /// Return the number of quota for `account_id`. This quota is the total number of account that user can
    /// transfer near and tokens to. For each successful transfer the total quota will be decreased by 1.
    pub fn get_account_quota(&self, account_id: &AccountId) -> u128 {
        self.balances.get(account_id).unwrap_or_default()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    // use near_sdk::MockedBlockchain;
    use near_sdk::testing_env;

    use super::*;

    // const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new(
            AccountId::new_unchecked("id".to_string()),
            AccountId::new_unchecked("id".to_string()),
        );
        testing_env!(context.is_view(true).build());
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_increase_account_quota() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(
            AccountId::new_unchecked("id".to_string()),
            AccountId::new_unchecked("id".to_string()),
        ); // testing_env!(context.is_view(true).build());
        let service_fee: U128 = 1u128.into();
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(service_fee.0)
            .predecessor_account_id(accounts(2))
            .build());
        // contract.pay_service_fee();
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(service_fee.0)
            .predecessor_account_id(accounts(2))
            .build());
        contract.increase_account_quota(&accounts(2), 1);
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.get_account_quota(&accounts(2)), 2);
    }

    #[test]
    fn test_decrease_account_quota() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(
            AccountId::new_unchecked("id".to_string()),
            AccountId::new_unchecked("id".to_string()),
        ); // testing_env!(context.is_view(true).build());
        let service_fee: U128 = 1u128.into();
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(service_fee.0)
            .predecessor_account_id(accounts(2))
            .build());
        // contract.pay_service_fee();
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(service_fee.0)
            .predecessor_account_id(accounts(2))
            .build());
        contract.decrease_account_quota(&accounts(2), 1);
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.get_account_quota(&accounts(2)), 0);
    }
}