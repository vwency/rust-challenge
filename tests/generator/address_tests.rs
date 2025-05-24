use std::collections::HashSet;
use token_transfers::generator::address::rand_address;

#[test]
fn test_address_format() {
    let mut rng = rand::thread_rng();
    let addr = rand_address(&mut rng);

    assert!(addr.starts_with("0x"));
    assert_eq!(addr.len(), 42); // 0x + 40 chars

    let hex_part = &addr[2..];
    assert!(hex_part.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn test_address_uniqueness() {
    let mut rng = rand::thread_rng();
    let mut addresses = HashSet::new();

    // Generate 1000 addresses and check for uniqueness
    for _ in 0..1000 {
        let addr = rand_address(&mut rng);
        addresses.insert(addr);
    }

    // With random generation, we should have close to 1000 unique addresses
    // Allow for very small chance of collision
    assert!(addresses.len() >= 990);
}

#[test]
fn test_address_randomness() {
    let mut rng = rand::thread_rng();
    let addr1 = rand_address(&mut rng);
    let addr2 = rand_address(&mut rng);

    // Addresses should be different
    assert_ne!(addr1, addr2);

    // Both should have correct format
    assert!(addr1.starts_with("0x") && addr1.len() == 42);
    assert!(addr2.starts_with("0x") && addr2.len() == 42);
}
