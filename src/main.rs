use std::{fs::File, collections::HashMap};
use std::io::BufReader;
use serde_json::{Result, Value};

struct KeyDiff<'a> {
    key: &'a str,
    has: &'a str,
    misses: &'a str
}

struct ValueDiff<'b> {
    key: &'b str,
    value1: (&'b str, &'b str),
    value2: (&'b str, &'b str)
}

fn main() -> Result<()> {
    let data1 = read_json_file("test_data/person1.json")?;
    let data2 = read_json_file("test_data/person2.json")?;
    println!("data1: {:?}", data1);
    println!("data2: {:?}", data2);
    Ok(())
}

fn read_json_file(file_path: &str) -> Result<Value> {
    let file = File::open(file_path)
        .expect(&format!("Could not open file {}", file_path));
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)?;
    Ok(result)
}

fn compare_objects(a: &Value, b: &Value) -> (Vec<KeyDiff>, Vec<ValueDiff>) {
    match(a, b) {
        (Value::Null, Value::Null) => (vec![], vec![]),
        (Value::Bool(a), Value::Bool(b)) => {
            let key_diff = vec![];
            let mut value_diff: Vec<ValueDiff> = vec![];

            if a != b {
                value_diff = vec![
                    ValueDiff {
                        key: "",
                        value1: ("", a.to_string().as_str()),
                        value2: ("", b.to_string().as_str())
                    }
                ];
            }

            return (key_diff, value_diff);
        } 
    }
}
