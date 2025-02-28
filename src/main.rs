use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::process::exit;
use chksum::{chksum, SHA2_256};
use colored::{Colorize, ColoredString};
use json::JsonValue;

/// Compares two files by computing their SHA2-256 checksums.
fn check_for_is_same_input(file_a: File, file_b: File) -> bool {
    chksum::<SHA2_256>(file_a).unwrap().to_hex_lowercase() ==
    chksum::<SHA2_256>(file_b).unwrap().to_hex_lowercase()
}
fn print_diff_tree(json_a: &JsonValue, json_b: &JsonValue, indent: usize) {
    let indentation = "  ".repeat(indent);

    if json_a.is_object() || json_b.is_object() {
        let mut keys = HashSet::new();
        if json_a.is_object() {
            for (key, _) in json_a.entries() {
                keys.insert(key);
            }
        }
        if json_b.is_object() {
            for (key, _) in json_b.entries() {
                keys.insert(key);
            }
        }
        for key in keys {
            let a_has = json_a.has_key(key);
            let b_has = json_b.has_key(key);
            if a_has && !b_has {
                println!("{}|__{}", indentation, key.red());
            } else if !a_has && b_has {
                println!("{}|__{}", indentation, key.green());
            } else {
                let a_val = &json_a[key];
                let b_val = &json_b[key];
                if a_val != b_val {
                    println!("{}|__{}", indentation, key.blue());
                    if (a_val.is_object() && b_val.is_object())
                        || (a_val.is_array() && b_val.is_array())
                    {
                        print_diff_tree(a_val, b_val, indent + 1);
                    }
                }
            }
        }
    }
    else if json_a.is_array() || json_b.is_array() {
        let max_len = std::cmp::max(json_a.len(), json_b.len());
        for i in 0..max_len {
            let a_exists = i < json_a.len();
            let b_exists = i < json_b.len();
            if a_exists && !b_exists {
                println!("{}|__[{}]", indentation, i.to_string().red());
            } else if !a_exists && b_exists {
                println!("{}|__[{}]", indentation, i.to_string().green());
            } else {
                let a_elem = &json_a[i];
                let b_elem = &json_b[i];
                if a_elem != b_elem {
                    println!("{}|__[{}]", indentation, i.to_string().blue());
                    if (a_elem.is_object() && b_elem.is_object())
                        || (a_elem.is_array() && b_elem.is_array())
                    {
                        print_diff_tree(a_elem, b_elem, indent + 1);
                    }
                }
            }
        }
    }
}

/// Returns the (row, column) of the first occurrence of the given key in the JSON string.
fn get_position(json_str: &str, key: &str) -> Option<(usize, usize)> {
    if let Some(pos) = json_str.find(&format!("\"{}\"", key)) {
        let before = &json_str[..pos];
        let row = before.matches('\n').count() + 1;
        let column = pos - before.rfind('\n').unwrap_or(0);
        Some((row, column))
    } else {
        None
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} fileA.json fileB.json", args[0]);
        exit(1);
    }
    let file_a_path = &args[1];
    let file_b_path = &args[2];

    let file_a = File::open(file_a_path).expect("Could not open fileA");
    let file_b = File::open(file_b_path).expect("Could not open fileB");

    if check_for_is_same_input(
        file_a.try_clone().expect("Clone failed"),
        file_b.try_clone().expect("Clone failed"),
    ) {
        println!("Files are the same!");
        exit(0);
    }

    let json_string_a = std::fs::read_to_string(file_a_path).expect("Could not read fileA");
    let json_a = json::parse(&json_string_a).expect("Invalid JSON in fileA");

    let json_string_b = std::fs::read_to_string(file_b_path).expect("Could not read fileB");
    let json_b = json::parse(&json_string_b).expect("Invalid JSON in fileB");

    println!(
        "The First File is {:?} in Length ({} vs {})",
        json_string_a.len().cmp(&json_string_b.len()),
        json_string_a.len(),
        json_string_b.len()
    );

    let mut pairs = Vec::<(ColoredString, (usize, usize))>::new();
    let mut processed_keys = HashSet::new();

    // Check keys in file A.
    for (key, value) in json_a.entries() {
        processed_keys.insert(key);
        let position = get_position(&json_string_b, key).unwrap_or((0, 0));

        if json_b.has_key(key) {
            if json_b[key].to_string() != value.to_string() {
                println!("Difference found in key: {}", key.blue());
                println!("--- Diff for key '{}' ---", key);
                print_diff_tree(value, &json_b[key], 0);
                println!("--------------------------");
                pairs.push((key.blue(), position));
            } else {
                pairs.push((key.white(), position));
            }
        } else {
            pairs.push((key.red(), (0, 0)));
        }
    }

    // Check keys that exist only in file B.
    for (key, _) in json_b.entries() {
        if !processed_keys.contains(key) {
            let position = get_position(&json_string_b, key).unwrap_or((0, 0));
            pairs.push((key.green(), position));
        }
    }

    pairs.sort_by(|a, b| a.1.0.cmp(&b.1.0));
    for (i, (key, _pos)) in pairs.iter().enumerate() {
        println!("{} | {} ", i + 1, key);
    }
}
