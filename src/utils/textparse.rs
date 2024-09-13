use std::collections::HashMap;

pub fn parse_and_store_as_vec(input: &str) -> Vec<String> {
    input.split('|').map(|s| s.trim_end().to_string()).collect()
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

pub fn format_string_as_steam_token(data: &str) -> String {
    data.replace(" ", "+")
}