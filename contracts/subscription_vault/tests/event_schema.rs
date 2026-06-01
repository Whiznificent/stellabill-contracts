#![cfg(test)]

extern crate alloc;

use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env, IntoVal, Symbol, Val,
};
use subscription_vault::{
    SubscriptionVault, SubscriptionVaultClient, AdminRotatedEvent, NonceConsumedEvent,
    SubscriptionCreatedEvent, nonce,
};

#[test]
fn test_nonce_consumed_and_admin_rotated_event_topics_and_shapes() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone()).address();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let contract_id = env.register(SubscriptionVault, ());
    let client = SubscriptionVaultClient::new(&env, &contract_id);

    let min_topup: i128 = 1_000_000;
    let grace_period: u64 = 3600;

    client.init(&token_address, &7u32, &admin, &min_topup, &grace_period);

    // rotate_admin should consume the admin nonce and emit two events: nonce_consumed, admin_rotated
    client.rotate_admin(&admin, &new_admin, &0u64);

    let events = env.events().all();
    assert!(events.len() >= 2, "rotate_admin must emit at least two events (nonce + admin_rotated)");

    let ts = env.ledger().timestamp();

    // Check event 0: nonce_consumed
    let (addr0, topics0, data0) = events.get(0).unwrap();
    assert_eq!(addr0, contract_id);
    let expected_topics0: Val = (Symbol::new(&env, "nonce_consumed"), admin.clone(), Symbol::new(&env, "adm_rot")).into_val(&env);
    assert_eq!(topics0.into_val(&env), expected_topics0);
    let expected_data0: Val = NonceConsumedEvent { signer: admin.clone(), domain: nonce::DOMAIN_ADMIN_ROTATION, nonce: 0u64, timestamp: ts }.into_val(&env);
    assert_eq!(data0, expected_data0);

    // Check event 1: admin_rotated
    let (addr1, topics1, data1) = events.get(1).unwrap();
    assert_eq!(addr1, contract_id);
    let expected_topics1: Val = (Symbol::new(&env, "admin_rotated"),).into_val(&env);
    assert_eq!(topics1.into_val(&env), expected_topics1);
    let expected_data1: Val = AdminRotatedEvent { old_admin: admin.clone(), new_admin: new_admin.clone(), timestamp: ts }.into_val(&env);
    assert_eq!(data1, expected_data1);
}

#[test]
fn test_subscription_created_event_topic_and_shape() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone()).address();

    let admin = Address::generate(&env);
    let subscriber = Address::generate(&env);
    let merchant = Address::generate(&env);

    let contract_id = env.register(SubscriptionVault, ());
    let client = SubscriptionVaultClient::new(&env, &contract_id);

    client.init(&token_address, &7u32, &admin, &1_000_000i128, &3600u64);

    let amount: i128 = 1_000_000;
    let interval_seconds: u64 = 30 * 24 * 60 * 60;

    let subscription_id = client.create_subscription(&subscriber, &merchant, &amount, &interval_seconds, &false, &None, &None::<u64>);

    let events = env.events().all();
    let (addr, topics, data) = events.last().unwrap();

    assert_eq!(addr, contract_id);
    let expected_topics: Val = (Symbol::new(&env, "created"), subscription_id).into_val(&env);
    assert_eq!(topics.into_val(&env), expected_topics);
    let expected_data: Val = SubscriptionCreatedEvent {
        subscription_id,
        subscriber,
        merchant,
        token: token_address,
        amount,
        interval_seconds,
        lifetime_cap: None,
        expires_at: None,
        timestamp: env.ledger().timestamp(),
    }.into_val(&env);
    assert_eq!(data, expected_data);
}
