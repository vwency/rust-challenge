use rand::Rng;

pub fn rand_address<R: Rng + ?Sized>(rng: &mut R) -> String {
    let bytes: [u8; 20] = rng.random();

    let hex_str = bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    format!("0x{}", hex_str)
}
