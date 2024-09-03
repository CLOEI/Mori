use std::collections::HashMap;

pub fn parse_and_store_as_vec(input: &str) -> Vec<&str> {
    input.split('|').collect()
}

pub fn parse_and_store_as_map(input: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in input.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 2 {
            let key = parts[0].to_string();
            let value = parts[1..].join("|");
            map.insert(key, value);
        }
    }
    map
}

pub fn format_byte_as_steam_token(data: Vec<u8>) -> String {
    data
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<String>>()
        .join("+")
}