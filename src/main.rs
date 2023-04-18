use serde::Serialize;
use serde_json::{Map, Number, Result, Value};
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;

#[derive(Clone)]
struct KeyDiff<'a> {
    key: &'a str,
    has: &'a str,
    misses: &'a str,
}

#[derive(Clone)]
struct ValueDiff<'b> {
    key: &'b str,
    value1: (&'b str, &'b str),
    value2: (&'b str, &'b str),
}

fn main() -> Result<()> {
    let data1 = read_json_file("test_data/person1.json")?;
    let data2 = read_json_file("test_data/person2.json")?;
    println!("data1: {:?}", data1);
    println!("data2: {:?}", data2);
    Ok(())
}

fn read_json_file(file_path: &str) -> Result<Value> {
    let file = File::open(file_path).expect(&format!("Could not open file {}", file_path));
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)?;
    Ok(result)
}

fn compare_json<'a>(
    a: &'a Value,
    b: &'a Value,
) -> (Box<Vec<KeyDiff<'a>>>, Box<Vec<ValueDiff<'a>>>) {
    match (a, b) {
        (Value::Null, Value::Null) => (Box::new(vec![]), Box::new(vec![])),
        (Value::Bool(a), Value::Bool(b)) => compare_primitives(a, b),
        (Value::Number(a), Value::Number(b)) => compare_primitives(a, b),
        (Value::String(a), Value::String(b)) => compare_primitives(a, b),
        (Value::Array(a), Value::Array(b)) => compare_arrays(a, b),
        (Value::Object(a), Value::Object(b)) => compare_objects(a, b),
    }
}

fn compare_arrays<'a>(
    a: &'a Vec<Value>,
    b: &'a Vec<Value>,
) -> (Box<Vec<KeyDiff<'a>>>, Box<Vec<ValueDiff<'a>>>) {
    let mut key_diff = Box::new(vec![]);
    let mut value_diff = Box::new(vec![]);

    // TODO: Array checking should be configurable if order matters or not or even if they should be checked
    for (i, a_item) in a.iter().enumerate() {
        let (mut item_key_diff, mut item_value_diff) = compare_primitives(a_item, &b[i]);
        key_diff.append(&mut item_key_diff);
        value_diff.append(&mut item_value_diff);
    }

    (key_diff, value_diff)
}

// TODO: This is not recursive!
fn compare_objects<'a>(
    a: &'a Map<String, Value>,
    b: &'a Map<String, Value>,
) -> (Box<Vec<KeyDiff<'a>>>, Box<Vec<ValueDiff<'a>>>) {
    let mut key_diff = Box::new(vec![]);
    let mut value_diff = Box::new(vec![]);

    // TODO: Array checking should be configurable if order matters or not or even if they should be checked
    for (key, value) in a.iter() {
        //Comparing keys
        if let Some(b_value) = b.get(key) {
            // Comparing values
            let (mut item_key_diff, mut item_value_diff) =
                compare_primitives(value, b.get(key).unwrap());
            key_diff.append(&mut item_key_diff);
            value_diff.append(&mut item_value_diff);
        } else {
            key_diff.push(KeyDiff {
                key: key,
                // TODO: use file names instead
                has: "a",
                misses: "b",
            });
        }
    }

    (key_diff, value_diff)
}

fn compare_primitives<'a, T: PartialEq + Serialize + Display>(
    a: &'a T,
    b: &'a T,
) -> (Box<Vec<KeyDiff<'a>>>, Box<Vec<ValueDiff<'a>>>) {
    // TODO: remove key_diff, not needed with primitives
    let key_diff = Box::new(vec![]);
    let mut value_diff = Box::new(vec![]);

    if !a.eq(b) {
        value_diff.push(ValueDiff {
            key: "",
            value1: ("", a.to_string().as_str()),
            value2: ("", b.to_string().as_str()),
        });
    }

    (key_diff, value_diff)
}
