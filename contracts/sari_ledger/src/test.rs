#![cfg(test)]
use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::Address as _,
    Address, Env,
};

/// Spin up a clean env + contract client, ready for the owner to act.
fn setup() -> (Env, SariLedgerContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, SariLedgerContract);
    let client = SariLedgerContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    client.initialize(&owner);
    (env, client, owner)
}

// ---------------------------------------------------------------------------
// Test 1 (Happy path): MVP transaction — restock then sell — runs end-to-end.
// ---------------------------------------------------------------------------
#[test]
fn test_happy_path_restock_and_sale() {
    let (_env, client, _owner) = setup();

    let rice = symbol_short!("RICE");
    client.restock(&rice, &50, &45);        // 50 units @ ₱45 cost
    client.record_sale(&rice, &10, &55);    // sold 10 @ ₱55

    let inv = client.get_inventory(&rice);
    assert_eq!(inv.quantity, 40);
    assert_eq!(client.get_revenue(), 550);  // 10 * 55
}

// ---------------------------------------------------------------------------
// Test 2 (Edge case): selling more than is stocked must panic and not
// corrupt inventory or revenue.
// ---------------------------------------------------------------------------
#[test]
#[should_panic(expected = "insufficient inventory")]
fn test_insufficient_inventory() {
    let (_env, client, _owner) = setup();

    let soap = symbol_short!("SOAP");
    client.restock(&soap, &5, &20);
    client.record_sale(&soap, &10, &25);    // should panic
}

// ---------------------------------------------------------------------------
// Test 3 (State verification): after several restocks and sales, inventory
// and revenue match the ledger exactly.
// ---------------------------------------------------------------------------
#[test]
fn test_state_after_multiple_sales() {
    let (_env, client, _owner) = setup();

    let noodles = symbol_short!("NOODLES");
    client.restock(&noodles, &100, &8);
    client.record_sale(&noodles, &5, &12);
    client.record_sale(&noodles, &7, &12);
    client.record_sale(&noodles, &3, &15);

    let inv = client.get_inventory(&noodles);
    assert_eq!(inv.quantity, 85);           // 100 - 5 - 7 - 3
    // 5*12 + 7*12 + 3*15 = 60 + 84 + 45 = 189
    assert_eq!(client.get_revenue(), 189);
}

// ---------------------------------------------------------------------------
// Test 4 (Loan happy path): a request within the 30% ratio is approved,
// returns a loan id, and the loan record is retrievable.
// ---------------------------------------------------------------------------
#[test]
fn test_loan_within_limit() {
    let (_env, client, _owner) = setup();

    let soap = symbol_short!("SOAP");
    client.restock(&soap, &1000, &20);
    client.record_sale(&soap, &500, &25);   // revenue = 12,500

    // 30% cap = 3,750; request 3,000 → approved
    let loan_id = client.request_loan(&3000);
    assert_eq!(loan_id, 0);

    let loan = client.get_loan(&loan_id);
    assert_eq!(loan.amount, 3000);
    assert_eq!(loan.repaid, 0);
}

// ---------------------------------------------------------------------------
// Test 5 (Loan edge case): a request exceeding the 30% ratio must panic.
// ---------------------------------------------------------------------------
#[test]
#[should_panic(expected = "loan exceeds eligible amount")]
fn test_loan_exceeds_limit() {
    let (_env, client, _owner) = setup();

    let soap = symbol_short!("SOAP");
    client.restock(&soap, &100, &20);
    client.record_sale(&soap, &10, &25);    // revenue = 250

    // 30% cap = 75; request 200 → panic
    client.request_loan(&200);
}