use rand::Rng;

// KLV keys embedded in the Growtopia binary.
const KEY1: &str = "832aac071ffbcfc15bfe1d0a7ad15221";
const KEY2: &str = "709296ddd04fc4074a7b443ecc0799aa";
const KEY3: &str = "623de1e8fff22a2b3e0d7e01593e7c22";
const KEY4: &str = "bb835e5a57e6c88e2449499ca487ced2";
const KEY5: &str = "ea76e4d6009282186063fe9465f2d9ab";

fn md5u(s: &str) -> String {
    format!("{:X}", md5::compute(s.as_bytes()))
}

/// Growtopia's custom rotate-left-5 hash with NUL terminator appended
/// (equivalent to `HashMode::NullTerminated`).
pub fn hash_string(s: &str) -> i32 {
    let mut h: u32 = 0x55555555;
    for b in s.bytes().chain(std::iter::once(0u8)) {
        h = h.rotate_left(5).wrapping_add(b as u32);
    }
    h as i32
}

/// Computes the `klv` field.
pub fn compute_klv(game_version: &str, protocol: &str, rid: &str, hash_val: i32) -> String {
    let combined = format!(
        "{}{}{}{}{}{}{}{}{}",
        md5u(&md5u(game_version)),
        KEY1,
        md5u(&md5u(&md5u(protocol))),
        KEY2,
        KEY3,
        md5u(&md5u(rid)),
        KEY4,
        md5u(&md5u(&hash_val.to_string())),
        KEY5,
    );
    md5u(&combined)
}

/// Generate `n` random uppercase hex characters.
pub fn random_hex(n: usize) -> String {
    let mut rng = rand::rng();
    (0..n).map(|_| format!("{:X}", rng.random::<u8>() & 0xF)).collect()
}

/// Generate a random locally-administered unicast MAC address.
pub fn random_mac() -> String {
    let mut rng = rand::rng();
    format!(
        "02:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        rng.random::<u8>(),
        rng.random::<u8>(),
        rng.random::<u8>(),
        rng.random::<u8>(),
        rng.random::<u8>(),
    )
}

/// Generates a 32-char uppercase hex RID derived from the current nanosecond timestamp.
pub fn generate_rid() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:032X}", nanos)
}
