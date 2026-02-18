use super::*;

#[test]
fn verify_pkce_challenge_accepts_valid_pair() {
    // Example from RFC 7636 (Section 4.2).
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

    assert!(verify_pkce_challenge(challenge, verifier));
}

#[test]
fn verify_pkce_challenge_rejects_invalid_pair() {
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let challenge = "invalid-challenge";

    assert!(!verify_pkce_challenge(challenge, verifier));
}
