use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

pub fn parse_config(path: &str) -> Result<HashMap<String, Vec<String>>, String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(why) => return Err(format!("Failed to open config file: {}", why)),
    };

    let mut config = HashMap::new();

    let mut multi_line_value = None;
    let mut multi_line_key = None;

    for line in BufReader::new(file).lines() {
        let mut line = match line {
            Ok(line) => line,
            Err(why) => return Err(format!("Failed to read line: {}", why)),
        };

        line = line.trim().to_string();

        // Ignore empty lines and comments
        if line == "" {
            continue;
        } else if line.starts_with("#") {
            continue;
        }

        let mut final_values = Vec::new();
        let mut key = String::new();

        // No multi line set beforehand, multi line character present
        if multi_line_value.is_none() && line.ends_with("\\") {
            let (key, value) = get_parts(&line).expect("Unable to split line");
            multi_line_value = Some(Vec::new());
            multi_line_key = Some(key);

            multi_line_value.as_mut().unwrap().push(value.to_string());
            continue;
        // No multi line set beforehand, no multi line character
        } else if multi_line_value.is_none() {
            let (_key, value) = get_parts(&line).expect("Unable to split line");
            key = _key;
            final_values.push(value);
        // Multi line set beforehand, multi line character present
        } else if multi_line_value.is_some() && line.ends_with("\\") {
            multi_line_value
                .as_mut()
                .unwrap()
                .push(line.replace("\\", "").trim().to_string());
            continue;
        // Multi line set beforehand, multi line character not present
        } else if multi_line_value.is_some() {
            multi_line_value.as_mut().unwrap().push(line);

            final_values = multi_line_value.unwrap().clone();
            key = multi_line_key.unwrap();

            multi_line_key = None;
            multi_line_value = None;
        }

        config.insert(key.to_string(), final_values);
    }

    Ok(config)
}

fn get_parts(line: &str) -> Result<(String, String), String> {
    let mut parts = line.splitn(2, '=');
    let key = match parts.next() {
        Some(key) => key,
        None => return Err(format!("Failed to parse key from line: {}", line)),
    };
    let mut value = match parts.next() {
        Some(value) => value.to_string(),
        None => return Err(format!("Failed to parse value from line: {}", line)),
    };

    if value.ends_with("\\") {
        value = value.replace("\\", "");
    }

    // Trim them to remove any excess whitespace
    Ok((key.trim().to_string(), value.trim().to_string()))
}
