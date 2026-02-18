use super::*;
use crate::error::Error;

#[test]
fn hashed_password_round_trip_verifies() {
    let hash = HashedPassword::new("correct-horse-battery-staple").expect("hash should be created");

    let parsed = HashedPassword::from_hash(hash.as_str()).expect("hash should parse");

    assert!(parsed
        .verify("correct-horse-battery-staple")
        .expect("verify should succeed"));
    assert!(!parsed
        .verify("wrong-password")
        .expect("verify should succeed"));
}

#[test]
fn hashed_password_rejects_invalid_hash() {
    let err = HashedPassword::from_hash("not-a-valid-hash").expect_err("invalid hash should fail");
    assert!(matches!(err, Error::Unexpected(_)));
}
