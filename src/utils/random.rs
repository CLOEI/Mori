use rand::Rng;

pub fn random_hex(length: u32, upper: bool) -> String {
    let chars = if upper {
        "0123456789ABCDEF"
    } else {
        "0123456789abcdef"
    };
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..chars.len());
            chars.chars().nth(idx).unwrap()
        })
        .collect()
}

pub fn random_mac_address() -> String {
    let mut mac = random_hex(2, false);
    for _ in 0..5 {
        mac.push_str(&format!(":{}", random_hex(2, false)));
    }
    mac
}
