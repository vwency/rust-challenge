use rand::{distributions::Alphanumeric, Rng};

pub fn rand_address(rng: &mut impl Rng) -> String {
    let suffix: String = rng
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect();
    format!("0x{}", suffix)
}
