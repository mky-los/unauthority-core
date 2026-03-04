// Integration test: USP-01 WrapMint proof replay protection
//
// Validates that the same bridge deposit proof cannot be used twice
// to mint wrapped tokens, preventing unbacked supply inflation.

use los_vm::usp01::{Usp01Action, Usp01Token};

const BRIDGE: &str = "LOS_BRIDGE_OPERATOR_001";
const ALICE: &str = "LOS_ALICE_TEST_ADDR_001";

#[test]
fn test_wrap_mint_replay_rejected() {
    let mut token = Usp01Token::new_wrapped(
        "Wrapped Bitcoin".to_string(),
        "wBTC".to_string(),
        8,
        "bitcoin".to_string(),
        BRIDGE.to_string(),
    )
    .unwrap();

    let proof = "btctx_unique_deposit_001".to_string();

    // First mint succeeds
    let resp = token.execute(
        BRIDGE,
        Usp01Action::WrapMint {
            to: ALICE.to_string(),
            amount: 100_000_000,
            proof: proof.clone(),
        },
    );
    assert!(resp.success);
    assert_eq!(token.balances[ALICE], 100_000_000);
    assert_eq!(token.metadata.total_supply, 100_000_000);

    // Second mint with SAME proof is rejected (replay protection)
    let resp = token.execute(
        BRIDGE,
        Usp01Action::WrapMint {
            to: ALICE.to_string(),
            amount: 100_000_000,
            proof: proof.clone(),
        },
    );
    assert!(!resp.success);
    assert!(resp.message.contains("replay rejected"));
    // Balance and supply unchanged
    assert_eq!(token.balances[ALICE], 100_000_000);
    assert_eq!(token.metadata.total_supply, 100_000_000);

    // Different proof succeeds
    let resp = token.execute(
        BRIDGE,
        Usp01Action::WrapMint {
            to: ALICE.to_string(),
            amount: 50_000_000,
            proof: "btctx_unique_deposit_002".to_string(),
        },
    );
    assert!(resp.success);
    assert_eq!(token.balances[ALICE], 150_000_000);
    assert_eq!(token.metadata.total_supply, 150_000_000);
}
