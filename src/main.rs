use serde_json::{json, Map, Number, Result, Value};
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;

struct KeyDiff<'a> {
    key: String,
    has: &'a str,
    misses: &'a str,
}

struct ValueDiff<'b> {
    key: String,
    value1: (&'b str, String),
    value2: (&'b str, String),
}

struct TypeDiff<'c> {
    key: String,
    type1: (&'c str, String),
    type2: (&'c str, String),
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
) -> (Vec<KeyDiff<'a>>, Vec<TypeDiff<'a>>, Vec<ValueDiff<'a>>) {
    match (a, b) {
        (Value::Null, Value::Null) => (vec![], vec![], vec![]),
        (Value::Bool(a), Value::Bool(b)) => (vec![], vec![], compare_primitives(a, b)),
        (Value::Number(a), Value::Number(b)) => (vec![], vec![], compare_primitives(a, b)),
        (Value::String(a), Value::String(b)) => (vec![], vec![], compare_primitives(a, b)),
        (Value::Array(a), Value::Array(b)) => (vec![], vec![], compare_arrays(a, b)),
        (Value::Object(a), Value::Object(b)) => compare_objects(a, b),
        (Value::Null, Value::Bool(b)) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(a.clone(), json!(b).to_owned()),
        ),
        (Value::Null, Value::Number(b)) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(a.clone(), json!(b).to_owned()),
        ),
        (Value::Null, Value::String(b)) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(a.clone(), json!(b).to_owned()),
        ),
        (Value::Null, Value::Array(b)) => (
            vec![],
            vec![],
            handle_one_element_null_arrays(a.clone(), json!(b).to_owned()),
        ),
        (Value::Null, Value::Object(b)) => {
            handle_one_element_null_objects(a.clone(), json!(b).to_owned())
        }
        (Value::Bool(a), Value::Null) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(json!(a).to_owned(), b.clone()),
        ),
        (Value::Bool(a), Value::Number(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Bool(a), Value::String(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Bool(a), Value::Array(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Bool(a), Value::Object(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Number(a), Value::Null) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(json!(a).to_owned(), b.clone()),
        ),
        (Value::Number(a), Value::Bool(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Number(a), Value::String(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Number(a), Value::Array(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Number(a), Value::Object(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::String(a), Value::Null) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(json!(a).to_owned(), b.clone()),
        ),
        (Value::String(a), Value::Bool(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::String(a), Value::Array(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::String(a), Value::Number(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::String(a), Value::Object(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Array(a), Value::Null) => (
            vec![],
            vec![],
            handle_one_element_null_arrays(json!(a).to_owned(), b.clone()),
        ),
        (Value::Array(a), Value::Bool(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Array(a), Value::Number(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Array(a), Value::String(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Array(a), Value::Object(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Object(a), Value::Null) => {
            handle_one_element_null_objects(json!(a).to_owned(), b.clone())
        }
        (Value::Object(a), Value::Bool(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Object(a), Value::Number(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Object(a), Value::String(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
        (Value::Object(a), Value::Array(b)) => (
            vec![],
            handle_different_types(json!(a).to_owned(), json!(b).to_owned()),
            vec![],
        ),
    }
}

fn handle_one_element_null_primitives<'a>(a: Value, b: Value) -> Vec<ValueDiff<'a>> {
    if a.is_null() {
        return vec![ValueDiff {
            // TODO: add key
            key: "".to_string(),
            value1: ("", "".to_string()),
            value2: ("", b.to_string()),
        }];
    } else {
        vec![ValueDiff {
            // TODO: add key
            key: "".to_string(),
            value1: ("", a.to_string()),
            value2: ("", "".to_string()),
        }]
    }
}

fn handle_one_element_null_arrays<'a>(a: Value, b: Value) -> Vec<ValueDiff<'a>> {
    let mut value_diff = vec![];

    if a.is_null() {
        // b should always be an array, because the function is called from the appropriate match arm
        for b_item in b.as_array().unwrap() {
            value_diff.push(ValueDiff {
                // TODO: key
                key: "".to_string(),
                value1: ("", "".to_string()),
                value2: ("", b_item.to_string()),
            });
        }
    } else {
        // a should always be an array, because the function is called from the appropriate match arm
        for a_item in a.as_array().unwrap() {
            value_diff.push(ValueDiff {
                // TODO: key
                key: "".to_string(),
                value1: ("", a_item.to_string()),
                value2: ("", "".to_string()),
            });
        }
    }

    value_diff
}

fn handle_one_element_null_objects<'a>(
    a: Value,
    b: Value,
) -> (Vec<KeyDiff<'a>>, Vec<TypeDiff<'a>>, Vec<ValueDiff<'a>>) {
    let mut key_diff = vec![];
    let mut type_diff = vec![];
    let mut value_diff = vec![];

    let object = if a.is_null() {
        b.as_object().unwrap()
    } else {
        a.as_object().unwrap()
    };

    for (key, value) in object.iter() {
        key_diff.push(KeyDiff {
            key: key.to_string(),
            has: if a.is_null() { "b" } else { "a" },
            misses: if a.is_null() { "a" } else { "b" },
        });

        type_diff.push(TypeDiff {
            key: key.to_string(),
            type1: ("", "".to_string()),
            type2: ("", get_type(value)),
        });

        value_diff.push(ValueDiff {
            key: key.to_string(),
            value1: if a.is_null() {
                ("", "".to_string())
            } else {
                ("", value.to_string())
            },
            value2: if a.is_null() {
                ("", value.to_string())
            } else {
                ("", "".to_string())
            },
        });
    }

    (key_diff, type_diff, value_diff)
}

fn handle_different_types<'a>(a: Value, b: Value) -> Vec<TypeDiff<'a>> {
    vec![TypeDiff {
        // TODO: key
        key: "".to_string(),
        type1: ("", get_type(&a)),
        type2: ("", get_type(&b)),
    }]
}

fn get_type(value: &Value) -> String {
    if value.is_null() {
        return "null".to_string();
    } else if value.is_boolean() {
        return "bool".to_string();
    } else if value.is_number() {
        return "number".to_string();
    } else if value.is_string() {
        return "string".to_string();
    } else if value.is_array() {
        return "array".to_string();
    } else if value.is_object() {
        return "object".to_string();
    } else {
        "unknown type".to_string()
    }
}

// TODO: This solution only works if the array are the same size and in the same order
//  It should also cover the cases when one or the other is larger or not in the same order
//  Basically this needs a complete rewrite
fn compare_arrays<'a>(a: &'a Vec<Value>, b: &'a Vec<Value>) -> Vec<ValueDiff<'a>> {
    let mut value_diff = vec![];

    // TODO: Array checking should be configurable if order matters or not or even if they should be checked
    for (i, a_item) in a.iter().enumerate() {
        let mut item_value_diff = compare_primitives(a_item, &b[i]);
        value_diff.append(&mut item_value_diff);
    }

    value_diff
}

// TODO: This is not recursive!
fn compare_objects<'a>(
    a: &'a Map<String, Value>,
    b: &'a Map<String, Value>,
) -> (Vec<KeyDiff<'a>>, Vec<TypeDiff<'a>>, Vec<ValueDiff<'a>>) {
    let mut key_diff = vec![];
    let mut type_diff = vec![]; // TODO: check for these in fields
    let mut value_diff = vec![];

    for (a_key, a_value) in a.iter() {
        //Comparing keys
        if let Some(b_value) = b.get(a_key) {
            match(a_value, b_value) {
                // Primitives of same type
                (Value::Null, Value::Null) => (vec![], vec![], vec![]),
                (Value::String(a_value), Value::String(b_value)) =>
                    (vec![], vec![], compare_primitives(a_value, b_value)),
                (Value::Number(a_value), Value::Number(b_value)) =>
                    (vec![], vec![], compare_primitives(a_value, b_value)),
                (Value::Bool(a_value), Value::Bool(b_value)) => 
                    (vec![], vec![], compare_primitives(a_value, b_value)),

                // Composites of same type
                (Value::Array(a_value), Value::Array(b_value)) =>
                    (vec![], vec![], compare_arrays(a_value, b_value)),
                (Value::Object(a_value), Value::Object(b_value)) =>
                    compare_objects(a_value, b_value),

                // One value is null primitives
                (Value::Null, Value::String(b_value)) => (
                    vec![],
                    vec![],
                    handle_one_element_null_primitives(a_value.clone(), json!(b_value).to_owned())
                ),
                (Value::Null, Value::Number(b_value)) => (
                    vec![],
                    vec![],
                    handle_one_element_null_primitives(a_value.clone(), json!(b_value).to_owned())
                ),
                (Value::Null, Value::Bool(b_value)) => (
                    vec![],
                    vec![],
                    handle_one_element_null_primitives(a_value.clone(), json!(b_value).to_owned())
                ),
                
                (Value::String(a_value), Value::Null) => (
                    vec![],
                    vec![],
                    handle_one_element_null_primitives(json!(a_value).to_owned(), b_value.clone())
                ),
                (Value::Number(a_value), Value::Null) => (
                    vec![],
                    vec![],
                    handle_one_element_null_primitives(json!(a_value).to_owned(), b_value.clone())
                ),
                (Value::Bool(a_value), Value::Null) => (
                    vec![],
                    vec![],
                    handle_one_element_null_primitives(json!(a_value).to_owned(), b_value.clone())
                ),

                // One value is null, composites
                (Value::Null, Value::Array(b_value)) => (
                    vec![],
                    vec![],
                    handle_one_element_null_arrays(a_value.clone(), json!(b_value).to_owned())
                ),
                (Value::Null, Value::Object(b_value)) => 
                    handle_one_element_null_objects(a_value.clone(), json!(b_value).to_owned()),
                
                (Value::Array(a_value), Value::Null) => (
                    vec![],
                    vec![],
                    handle_one_element_null_arrays(json!(a_value).to_owned(), b_value.clone())
                ),
                (Value::Object(a_value), Value::Null) => 
                    handle_one_element_null_objects(json!(a_value).to_owned(), b_value.clone()),

                // Type difference a: string
                (Value::String(a_value), Value::Number(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::String(a_value), Value::Bool(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::String(a_value), Value::Array(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::String(a_value), Value::Object(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                
                // Type difference a: number
                (Value::Number(a_value), Value::String(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::Number(a_value), Value::Bool(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::Number(a_value), Value::Array(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::Number(a_value), Value::Object(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
            
                // Type difference a: bool
                (Value::Bool(a_value), Value::String(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::Bool(a_value), Value::Number(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::Bool(a_value), Value::Array(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::Bool(a_value), Value::Object(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),

                // Type difference a: array
                (Value::Array(a_value), Value::String(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::Array(a_value), Value::Bool(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::Array(a_value), Value::Number(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::Array(a_value), Value::Object(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                
                // Type difference a: object
                (Value::Object(a_value), Value::String(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::Object(a_value), Value::Bool(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::Object(a_value), Value::Array(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                ),
                (Value::Object(a_value), Value::Number(b_value)) => (
                    vec![],
                    handle_different_types(json!(a_value).to_owned(), json!(b_value).to_owned()),
                    vec![]
                )
            };

            // Comparing values
            let mut item_value_diff = compare_primitives(a_value, b_value);
            value_diff.append(&mut item_value_diff);
        } else {
            key_diff.push(KeyDiff {
                key: a_key.to_string(),
                // TODO: use file names instead
                has: "a",
                misses: "b",
            });
        }
    }

    (key_diff, type_diff, value_diff)
}

fn compare_primitives<'a, T: PartialEq + Display>(a: &'a T, b: &'a T) -> Vec<ValueDiff<'a>> {
    let mut value_diff = vec![];

    if !a.eq(b) {
        value_diff.push(ValueDiff {
            // TODO: add key
            key: "".to_string(),
            value1: ("", a.to_string()),
            value2: ("", b.to_string()),
        });
    }

    value_diff
}
