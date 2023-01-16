use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
// use near_sdk::serde_json::json;
use near_sdk_sim::ExecutionResult;
use near_sdk_sim::{call, to_yocto, view, DEFAULT_GAS};

use crate::utils::{init_with_macros as init, register_user};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalanceBounds {
    pub min: U128,
    pub max: Option<U128>,
}

#[test]
fn simulate_distribute_near() {
    let transfer_amount = to_yocto("100");
    let initial_balance = to_yocto("100000");

    let (root, _ft, bs, alice) = init(initial_balance);

    // let service_fee: U128 = view!(bs.service_fee()).unwrap_json();

    // let root_init_balance = root.account().unwrap().amount;
    let alice_init_balance = alice.account().unwrap().amount;

    // let service_res = call!(
    //     root,
    //     bs.pay_service_fee(),
    //     deposit = service_fee.0 * 2
    // );

    // service_res.assert_success();

    let distribute_res = call!(
        root,
        bs.distribute_near(
            vec![alice.account_id(), alice.account_id()],
            vec![transfer_amount.into(), transfer_amount.into()]
        ),
        transfer_amount * 2,
        DEFAULT_GAS / 2
    );

    distribute_res.assert_success();

    // let root_balance = root.account().unwrap().amount;
    let alice_balance = alice.account().unwrap().amount;
    // assert_eq!(root_init_balance - transfer_amount * 2 - distribute_res.tokens_burnt() - service_res.tokens_burnt(), root_balance);
    assert_eq!(alice_init_balance + transfer_amount * 2, alice_balance);
}

#[test]
fn simulate_distribute_ft() {
    let transfer_amount = to_yocto("100");
    let initial_balance = to_yocto("100000");

    let (root, ft, bs, alice) = init(initial_balance);

    register_user(&bs.user_account);

    // let service_fee: U128 = view!(bs.service_fee()).unwrap_json();

    // call!(
    //     root,
    //     bs.pay_service_fee(),
    //     deposit = service_fee.0 * 2
    // )
    // .assert_success();

    let res: ExecutionResult = call!(
        root,
        ft.ft_transfer_call(
            bs.account_id(),
            (transfer_amount * 2).into(),
            None,
            format!(
                "{}:{}#{}:{}",
                alice.account_id(),
                transfer_amount,
                alice.account_id(),
                transfer_amount
            )
            .into()
        ),
        1,
        DEFAULT_GAS / 2
    );

    res.assert_success();

    // println!("{:#?}", res.promise_results());

    assert_eq!(res.promise_errors().len(), 0);

    let root_balance: U128 = view!(ft.ft_balance_of(root.account_id())).unwrap_json();
    let alice_balance: U128 = view!(ft.ft_balance_of(alice.account_id())).unwrap_json();
    // println!("alice_balance: {}", alice_balance.0);
    assert_eq!(initial_balance - transfer_amount * 2, root_balance.0);
    assert_eq!(transfer_amount * 2, alice_balance.0);
}

#[test]
fn simulate_batch_storage_deposit() {
    let initial_balance = to_yocto("100000");

    let (_root, ft, bs, alice) = init(initial_balance);
    let storage_bounds: StorageBalanceBounds = view!(ft.storage_balance_bounds()).unwrap_json();

    call!(
        alice,
        bs.batch_storage_deposit(
            ft.account_id(),
            vec![bs.account_id(), alice.account_id()],
            storage_bounds.min
        ),
        storage_bounds.min.0 * 2,
        DEFAULT_GAS / 2
    )
    .assert_success();
}
