use std::collections::HashSet;
use crate::generator::address::rand_address;

#[test]
fn test_address_format() {
    let mut rng = rand::thread_rng();
    let addr = rand_address(&mut rng);

    assert!(addr.starts_with("0x"));
    assert_eq!(addr.len(), 42); // 0x + 40 chars

    let hex_part = &addr[2..];
    assert!(hex_part.chars().all(|c| c.is_digit(16)));
}

#[test]
fn test_address_uniqueness() {
    let mut rng = rand::thread_rng();
    let mut addresses = HashSet::new();

    for _ in 0..1000 {
        let addr = rand_address(&mut rng);
        addresses.insert(addr);
    }

    assert!(addresses.len() >= 990);
}

#[test]
fn test_address_randomness() {
    let mut rng = rand::thread_rng();
    let addr1 = rand_address(&mut rng);
    let addr2 = rand_address(&mut rng);

    assert_ne!(addr1, addr2);

    assert!(addr1.starts_with("0x") && addr1.len() == 42);
    assert!(addr2.starts_with("0x") && addr2.len() == 42);
}
