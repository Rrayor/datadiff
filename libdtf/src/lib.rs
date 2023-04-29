use serde_json::{Map, Result, Value};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;

mod compare_field;
pub mod diff_types;

use compare_field::compare_field;
use diff_types::{
    ArrayDiff, ArrayDiffDesc, ComparisionResult, KeyDiff, TypeDiff, ValueDiff, ValueType,
    WorkingContext,
};

pub fn read_json_file(file_path: &str) -> Result<Map<String, Value>> {
    let file = File::open(file_path).expect(&format!("Could not open file {}", file_path));
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)?;
    Ok(result)
}

pub fn compare_objects<'a>(
    key_in: &'a str,
    a: &'a Map<String, Value>,
    b: &'a Map<String, Value>,
    working_context: &WorkingContext,
) -> ComparisionResult {
    let mut key_diff = vec![];
    let mut type_diff = vec![];
    let mut value_diff = vec![];
    let mut array_diff = vec![];

    // create a map of all keys in `b`
    let mut b_keys = HashMap::new();
    for b_key in b.keys() {
        b_keys.insert(b_key, ());
    }

    // iterate over `a`, checking if each key is in `b`
    for (a_key, a_value) in a.into_iter() {
        let key = if key_in.is_empty() {
            a_key.to_string()
        } else {
            format!("{}.{}", key_in, a_key)
        };

        if let Some(b_value) = b.get(a_key) {
            // remove the key from `b_keys` if it is in `b`
            b_keys.remove(a_key);

            let (
                mut field_key_diff,
                mut field_type_diff,
                mut field_value_diff,
                mut field_array_diff,
            ) = compare_field(key.as_str(), a_value, b_value, working_context);
            {
                key_diff.append(&mut field_key_diff);
                type_diff.append(&mut field_type_diff);
                value_diff.append(&mut field_value_diff);
                array_diff.append(&mut field_array_diff);
            }
        } else {
            key_diff.push(KeyDiff {
                key: key,
                has: working_context.file_a.name.clone(),
                misses: working_context.file_b.name.clone(),
            });
        }
    }

    // add any keys remaining in `b_keys` to the `key_diff` vector
    for (b_key, _) in b_keys {
        let key = if key_in.is_empty() {
            b_key.to_string()
        } else {
            format!("{}.{}", key_in, b_key)
        };
        key_diff.push(KeyDiff {
            key: key,
            has: working_context.file_b.name.clone(),
            misses: working_context.file_a.name.clone(),
        });
    }

    (key_diff, type_diff, value_diff, array_diff)
}

fn compare_arrays<'a>(
    key: &'a str,
    a: &'a Vec<Value>,
    b: &'a Vec<Value>,
    working_context: &WorkingContext,
) -> ComparisionResult {
    let mut key_diff = vec![];
    let mut type_diff = vec![];
    let mut value_diff = vec![];
    let mut array_diff: Vec<ArrayDiff> = vec![];
    let same_order = false; // TODO: this should be configurable

    if a.len() == b.len() {
        if same_order {
            for (i, a_item) in a.iter().enumerate() {
                let (
                    mut item_key_diff,
                    mut item_type_diff,
                    mut item_value_diff,
                    mut item_array_diff,
                ) = compare_field(
                    format!("{}[{}]", key.to_string(), i.to_string()).as_str(),
                    a_item,
                    &b[i],
                    working_context,
                );

                key_diff.append(&mut item_key_diff);
                type_diff.append(&mut item_type_diff);
                value_diff.append(&mut item_value_diff);
                array_diff.append(&mut item_array_diff);
            }
        } else {
            array_diff = handle_different_order_arrays(a, b, key.to_string());
        }
    } else {
        array_diff = handle_different_order_arrays(a, b, key.to_string());
    }

    (key_diff, type_diff, value_diff, array_diff)
}

fn handle_different_order_arrays(a: &[Value], b: &[Value], key: String) -> Vec<ArrayDiff> {
    let mut array_diff = Vec::new();
    let (a_has, a_misses, b_has, b_misses) = fill_diff_vectors(a, b);

    for (value, desc) in a_has
        .iter()
        .map(|v| (v, ArrayDiffDesc::AHas))
        .chain(a_misses.iter().map(|v| (v, ArrayDiffDesc::AMisses)))
        .chain(b_has.iter().map(|v| (v, ArrayDiffDesc::BHas)))
        .chain(b_misses.iter().map(|v| (v, ArrayDiffDesc::BMisses)))
    {
        array_diff.push(ArrayDiff {
            key: key.clone(),
            descriptor: desc,
            value: value.to_string(),
        });
    }

    array_diff
}

fn fill_diff_vectors<'a, T: PartialEq + Display>(
    a: &'a [T],
    b: &'a [T],
) -> (Vec<&'a T>, Vec<&'a T>, Vec<&'a T>, Vec<&'a T>) {
    let a_has = a.iter().filter(|&x| !b.contains(x)).collect::<Vec<&T>>();
    let b_has = b.iter().filter(|&x| !a.contains(x)).collect::<Vec<&T>>();
    let a_misses = b.iter().filter(|&x| !a.contains(x)).collect::<Vec<&T>>();
    let b_misses = a.iter().filter(|&x| a.contains(x)).collect::<Vec<&T>>();

    (a_has, a_misses, b_has, b_misses)
}

fn compare_primitives<'a, T: PartialEq + Display>(
    key: &'a str,
    a: &'a T,
    b: &'a T,
) -> Vec<ValueDiff> {
    let mut diffs = vec![];

    if a != b {
        diffs.push(ValueDiff {
            key: key.to_string(),
            value1: a.to_string(),
            value2: b.to_string(),
        });
    }

    diffs
}

fn handle_different_types<'a>(key: &'a str, a: Value, b: Value) -> Vec<TypeDiff> {
    vec![TypeDiff {
        key: key.to_string(),
        type1: get_type(&a).to_string(),
        type2: get_type(&b).to_string(),
    }]
}

// One item is null

fn handle_one_element_null_primitives<'a>(key: &'a str, a: Value, b: Value) -> Vec<ValueDiff> {
    if a.is_null() {
        return vec![ValueDiff {
            key: key.to_string(),
            value1: "".to_string(),
            value2: b.to_string(),
        }];
    } else {
        vec![ValueDiff {
            key: key.to_string(),
            value1: a.to_string(),
            value2: "".to_string(),
        }]
    }
}

fn handle_one_element_null_arrays<'a>(key: &'a str, a: Value, b: Value) -> Vec<ArrayDiff> {
    match (a, b) {
        (Value::Null, Value::Array(b_array)) => b_array
            .iter()
            .map(|b_item| ArrayDiff {
                key: key.to_string(),
                descriptor: ArrayDiffDesc::BHas,
                value: b_item.to_string(),
            })
            .collect(),
        (Value::Array(a_array), Value::Null) => a_array
            .iter()
            .map(|a_item| ArrayDiff {
                key: key.to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: a_item.to_string(),
            })
            .collect(),
        _ => vec![],
    }
}

fn handle_one_element_null_objects<'a>(
    parent_key: &'a str,
    a: Value,
    b: Value,
    working_context: &WorkingContext,
) -> ComparisionResult {
    if (a.is_null() && b.is_null()) || (!a.is_null() && !b.is_null()) {
        panic!(
            "handle_one_element_null_objects called with wrong parameters: {} {}",
            a, b
        );
    }

    let mut key_diff = vec![];
    let mut type_diff = vec![];
    let mut value_diff = vec![];

    let object = if a.is_null() {
        b.as_object().unwrap()
    } else {
        a.as_object().unwrap()
    };

    for (key, value) in object.iter() {
        let full_key = if parent_key.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", parent_key, key)
        };
        key_diff.push(KeyDiff {
            key: full_key.clone(),
            has: if a.is_null() {
                working_context.file_b.name.clone()
            } else {
                working_context.file_a.name.clone()
            },
            misses: if a.is_null() {
                working_context.file_a.name.clone()
            } else {
                working_context.file_b.name.clone()
            },
        });

        type_diff.push(TypeDiff {
            key: full_key.clone(),
            type1: if a.is_null() {
                "".to_string()
            } else {
                get_type(value).to_string()
            },
            type2: if a.is_null() {
                get_type(value).to_string()
            } else {
                "".to_string()
            },
        });

        value_diff.push(ValueDiff {
            key: full_key.to_string(),
            value1: if a.is_null() {
                "".to_string()
            } else {
                value.as_str().unwrap().to_string()
            },
            value2: if a.is_null() {
                value.as_str().unwrap().to_string()
            } else {
                "".to_string()
            },
        });
    }

    (key_diff, type_diff, value_diff, vec![]) // TODO: handle arrays here?
}

// Util

fn get_type(value: &Value) -> ValueType {
    match value {
        Value::Null => ValueType::Null,
        Value::Bool(_) => ValueType::Boolean,
        Value::Number(_) => ValueType::Number,
        Value::String(_) => ValueType::String,
        Value::Array(_) => ValueType::Array,
        Value::Object(_) => ValueType::Object,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        diff_types::{WorkingContext, WorkingFile},
        handle_one_element_null_objects,
    };

    #[test]
    #[should_panic]
    fn test_handle_one_element_null_objects_panics_if_both_null() {
        let a = json!(null);
        let b = json!(null);
        let working_context = create_test_working_context();
        handle_one_element_null_objects("parent_key", a, b, &working_context);
    }

    #[test]
    #[should_panic]

    fn test_handle_one_element_null_objects_panics_neither_is_null() {
        let a = json!({ "key": "something" });
        let b = json!({ "key": "anything" });
        let working_context = create_test_working_context();
        handle_one_element_null_objects("parent_key", a, b, &working_context);
    }

    #[test]
    fn test_handle_one_element_null_objects_return_a_if_b_is_null() {
        let a = json!({ "key": "something" });
        let b = json!(null);
        let working_context = create_test_working_context();
        let (key_diff, type_diff, value_diff, array_diff) =
            handle_one_element_null_objects("parent_key", a, b, &working_context);

        assert_eq!(key_diff[0].key, "parent_key.key");
        assert_eq!(key_diff[0].has, "test1.json");
        assert_eq!(key_diff[0].misses, "test2.json");

        assert_eq!(type_diff[0].key, "parent_key.key");
        assert_eq!(type_diff[0].type1, "string");
        assert_eq!(type_diff[0].type2, "");

        assert_eq!(value_diff[0].key, "parent_key.key");
        assert_eq!(value_diff[0].value1, "something");
        assert_eq!(value_diff[0].value2, "");

        assert_eq!(array_diff.len(), 0);
    }

    #[test]
    fn test_handle_one_element_null_objects_return_b_if_a_is_null() {
        let a = json!(null);
        let b = json!({ "key": "something" });
        let working_context = create_test_working_context();
        let (key_diff, type_diff, value_diff, array_diff) =
            handle_one_element_null_objects("parent_key", a, b, &working_context);

        assert_eq!(key_diff[0].key, "parent_key.key");
        assert_eq!(key_diff[0].has, "test2.json");
        assert_eq!(key_diff[0].misses, "test1.json");

        assert_eq!(type_diff[0].key, "parent_key.key");
        assert_eq!(type_diff[0].type1, "");
        assert_eq!(type_diff[0].type2, "string");

        assert_eq!(value_diff[0].key, "parent_key.key");
        assert_eq!(value_diff[0].value1, "");
        assert_eq!(value_diff[0].value2, "something");

        assert_eq!(array_diff.len(), 0);
    }

    fn create_test_working_context() -> WorkingContext {
        WorkingContext {
            file_a: WorkingFile {
                name: "test1.json".to_string(),
            },
            file_b: WorkingFile {
                name: "test2.json".to_string(),
            },
        }
    }
}
