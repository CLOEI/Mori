use std::{
    fs::File,
    io::{self, Read},
};

use md5;
use sha2::{Digest, Sha256};

pub fn generate_klv(protocol: &str, version: &str, rid: &str) -> String {
    let salts = [
        "e9fc40ec08f9ea6393f59c65e37f750aacddf68490c4f92d0d2523a5bc02ea63",
        "c85df9056ee603b849a93e1ebab5dd5f66e1fb8b2f4a8caef8d13b9f9e013fa4",
        "3ca373dffbf463bb337e0fd768a2f395b8e417475438916506c721551f32038d",
        "73eff5914c61a20a71ada81a6fc7780700fb1c0285659b4899bc172a24c14fc1",
    ];

    let constant_values = [
        hash_sha256(&hash_md5(&hash_sha256(protocol))),
        hash_sha256(&hash_sha256(version)),
        hash_sha256(&(hash_sha256(protocol) + salts[3])),
    ];

    let result = hash_sha256(&format!(
        "{}{}{}{}{}{}{}",
        constant_values[0],
        salts[0],
        constant_values[1],
        salts[1],
        hash_sha256(&hash_md5(&hash_sha256(rid))),
        salts[2],
        constant_values[2]
    ));

    result
}

pub fn hash_sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn hash_md5(input: &str) -> String {
    let hasher = md5::compute(input);
    hex::encode(hasher.as_ref())
}

pub fn hash_string(input: &str) -> u32 {
    if input.is_empty() {
        return 0;
    }

    let mut acc: u32 = 0x55555555;
    for byte in input.as_bytes() {
        acc = (acc >> 27) + (acc << 5) + (*byte as u32);
    }
    acc
}

pub fn hash_file(file_name: &str) -> io::Result<u32> {
    let mut file = File::open(file_name)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut hash: u32 = 0x55555555;
    for byte in buffer {
        // Use wrapping_add to avoid overflow during addition
        hash = (hash >> 27).wrapping_add(hash << 5).wrapping_add(byte as u32);
    }

    Ok(hash)
}
