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
    array_same_order: bool, // TODO: Config object
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
            ) = compare_field(
                key.as_str(),
                a_value,
                b_value,
                working_context,
                array_same_order,
            );
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
    same_order: bool, // TODO: should have a config object
) -> ComparisionResult {
    let mut key_diff = vec![];
    let mut type_diff = vec![];
    let mut value_diff = vec![];
    let mut array_diff: Vec<ArrayDiff> = vec![];

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
                    same_order,
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
    let b_misses = a.iter().filter(|&x| !b.contains(x)).collect::<Vec<&T>>();

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

fn handle_different_types<'a>(key: &'a str, a: &Value, b: &Value) -> Vec<TypeDiff> {
    let type_a = get_type(&a);
    let type_b = get_type(&b);

    elements_different_types_guard_debug("handle_different_types", key, &type_a, &type_b);

    vec![TypeDiff {
        key: key.to_string(),
        type1: type_a.to_string(),
        type2: type_b.to_string(),
    }]
}

// One item is null

fn handle_one_element_null_primitives<'a>(key: &'a str, a: &Value, b: &Value) -> Vec<ValueDiff> {
    one_element_null_guard_debug("handle_one_element_null_primitives", &a, &b);

    if a.is_null() {
        return vec![ValueDiff {
            key: key.to_string(),
            value1: "".to_string(),
            value2: b.as_str().unwrap().to_string(),
        }];
    } else {
        vec![ValueDiff {
            key: key.to_string(),
            value1: a.as_str().unwrap().to_string(),
            value2: "".to_string(),
        }]
    }
}

fn handle_one_element_null_arrays<'a>(key: &'a str, a: &Value, b: &Value) -> Vec<ArrayDiff> {
    one_element_null_guard_debug("handle_one_element_null_arrays", &a, &b);
    let mut array_diff = vec![];

    if a.is_null() {
        for b_item in b.as_array().unwrap() {
            array_diff.push(ArrayDiff {
                key: key.to_string(),
                descriptor: ArrayDiffDesc::BHas,
                value: b_item.as_str().unwrap().to_string(),
            });
        }
    } else {
        for a_item in a.as_array().unwrap() {
            array_diff.push(ArrayDiff {
                key: key.to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: a_item.as_str().unwrap().to_string(),
            });
        }
    }

    array_diff
}

fn handle_one_element_null_objects<'a>(
    parent_key: &'a str,
    a: Value,
    b: Value,
    working_context: &WorkingContext,
) -> ComparisionResult {
    one_element_null_guard_debug("handle_one_element_null_objects", &a, &b);

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

// Debug guards

fn elements_different_types_guard_debug(
    function_name: &str,
    key: &str,
    type_a: &ValueType,
    type_b: &ValueType,
) {
    debug_assert!(
        type_a != type_b,
        "{} was called with the same types: {}: {}",
        function_name,
        key,
        type_a
    );
}

fn one_element_null_guard_debug(function_name: &str, a: &Value, b: &Value) {
    debug_assert!(
        a.is_null() ^ b.is_null(),
        "{} called with wrong parameters: {} {}",
        function_name,
        a,
        b
    );
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Map, Value};

    use crate::{
        compare_arrays, compare_objects, compare_primitives,
        diff_types::{
            ArrayDiff, ArrayDiffDesc, KeyDiff, TypeDiff, ValueDiff, WorkingContext, WorkingFile,
        },
        handle_different_types, handle_one_element_null_arrays, handle_one_element_null_objects,
        handle_one_element_null_primitives,
    };

    #[test]
    #[should_panic]
    fn test_handle_one_element_null_objects_panics_if_both_null() {
        // arrange
        let a = json!(null);
        let b = json!(null);
        let working_context = create_test_working_context();

        // act & assert (#[should_panic macro])
        handle_one_element_null_objects("parent_key", a, b, &working_context);
    }

    #[test]
    #[should_panic]
    fn test_handle_one_element_null_objects_panics_neither_is_null() {
        // arrange
        let a = json!({ "key": "something" });
        let b = json!({ "key": "anything" });
        let working_context = create_test_working_context();

        // act & assert (#[should_panic macro])
        handle_one_element_null_objects("parent_key", a, b, &working_context);
    }

    #[test]
    #[should_panic]
    fn test_handle_one_element_null_arrays_panics_if_both_null() {
        // arrange
        let a = json!(null);
        let b = json!(null);

        // act & assert (#[should_panic macro])
        handle_one_element_null_arrays("parent_key", &a, &b);
    }

    #[test]
    #[should_panic]
    fn test_handle_one_element_null_arrays_panics_neither_is_null() {
        // arrange
        let a = json!({ "key": vec!["something"] });
        let b = json!({ "key": vec!["anything"] });

        // act & assert (#[should_panic macro])
        handle_one_element_null_arrays("parent_key", &a, &b);
    }

    #[test]
    #[should_panic]
    fn test_handle_one_element_null_primitives_panics_if_both_null() {
        // arrange
        let a = json!(null);
        let b = json!(null);

        // act & assert (#[should_panic macro])
        handle_one_element_null_primitives("parent_key", &a, &b);
    }

    #[test]
    #[should_panic]
    fn test_handle_one_element_null_primitives_panics_neither_is_null() {
        // arrange
        let a = json!({ "key": "something" });
        let b = json!({ "key": "anything" });

        // act & assert (#[should_panic macro])
        handle_one_element_null_primitives("parent_key", &a, &b);
    }

    #[test]
    fn test_compare_arrays_returns_correct_when_not_same_order() {
        //arrange
        let arr_a = [1, 2, 3, 4, 5, 6, 7].map(|num| json!(num)).to_vec();
        let arr_b = [5, 7, 3, 11, 5, 2, 1].map(|num| json!(num)).to_vec();

        let expected = vec![
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::BHas,
                value: "11".to_string(),
            },
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::AMisses,
                value: "11".to_string(),
            },
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: "4".to_string(),
            },
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::BMisses,
                value: "4".to_string(),
            },
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: "6".to_string(),
            },
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::BMisses,
                value: "6".to_string(),
            },
        ];

        let working_context = WorkingContext {
            file_a: WorkingFile {
                name: "file_a".to_string(),
            },
            file_b: WorkingFile {
                name: "file_b".to_string(),
            },
        };

        // act
        let (_key_diff, _type_diff, _value_diff, array_diff) =
            compare_arrays("key", &arr_a, &arr_b, &working_context, false);

        // assert
        assert_eq!(array_diff.len(), expected.len());
        assert!(array_diff.iter().all(|num| expected.contains(num)));
    }

    #[test]
    fn test_compare_arrays_returns_correct_when_not_same_length() {
        //arrange
        let arr_a = [1, 2, 3, 4, 5, 6, 7, 8].map(|num| json!(num)).to_vec();
        let arr_b = [5, 7, 3, 11, 5, 2, 1].map(|num| json!(num)).to_vec();

        let expected = vec![
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::BHas,
                value: "11".to_string(),
            },
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::AMisses,
                value: "11".to_string(),
            },
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: "4".to_string(),
            },
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::BMisses,
                value: "4".to_string(),
            },
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: "6".to_string(),
            },
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::BMisses,
                value: "6".to_string(),
            },
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: "8".to_string(),
            },
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::BMisses,
                value: "8".to_string(),
            },
        ];

        let working_context = WorkingContext {
            file_a: WorkingFile {
                name: "file_a".to_string(),
            },
            file_b: WorkingFile {
                name: "file_b".to_string(),
            },
        };

        // act
        let (_key_diff, _type_diff, _value_diff, array_diff) =
            compare_arrays("key", &arr_a, &arr_b, &working_context, true);

        // assert
        assert_eq!(array_diff.len(), expected.len());
        assert!(array_diff.iter().all(|num| expected.contains(num)));
    }

    #[test]
    fn test_compare_arrays_returns_correct_when_same_order_same_length() {
        //arrange
        let arr_a = [1, 2, 3, 4, 5, 6, 7].map(|num| json!(num)).to_vec();
        let arr_b = [5, 2, 3, 5, 5, 8, 1].map(|num| json!(num)).to_vec();

        let expected = vec![
            ValueDiff {
                key: "key[0]".to_string(),
                value1: 1.to_string(),
                value2: 5.to_string(),
            },
            ValueDiff {
                key: "key[3]".to_string(),
                value1: 4.to_string(),
                value2: 5.to_string(),
            },
            ValueDiff {
                key: "key[5]".to_string(),
                value1: 6.to_string(),
                value2: 8.to_string(),
            },
            ValueDiff {
                key: "key[6]".to_string(),
                value1: 7.to_string(),
                value2: 1.to_string(),
            },
        ];

        let working_context = WorkingContext {
            file_a: WorkingFile {
                name: "file_a".to_string(),
            },
            file_b: WorkingFile {
                name: "file_b".to_string(),
            },
        };

        // act
        let (_key_diff, _type_diff, value_diff, _array_diff) =
            compare_arrays("key", &arr_a, &arr_b, &working_context, true);

        // assert
        assert_eq!(value_diff.len(), expected.len());
        assert!(value_diff.iter().all(|num| expected.contains(num)));
    }

    #[test]
    #[should_panic]
    fn test_handle_different_types_panics_if_same_type() {
        // arrange
        let a = json!(5);
        let b = json!(2);

        // act & assert (#[should_panic macro])
        handle_different_types("key", &a, &b);
    }

    #[test]
    fn test_compare_objects_array_same_order_same_length() {
        // arrange
        let object_a: Map<String, Value> = json!({
            "key_diff_a_has": "key_diff_a_has",
            "type_diff_a_string_b_int": "type_diff_a_string_b_int",
            "value_diff": "a",
            "array_diff_same_order": [
                1, 2, 3, 4, 5
            ],
            "nested_object": {
                "key_diff_a_has": "key_diff_a_has",
                "type_diff_a_string_b_int": "type_diff_a_string_b_int",
                "value_diff": "a",
                "array_diff_same_order": [
                    1, 2, 3, 4, 5
                ],
            }
        })
        .as_object()
        .unwrap()
        .to_owned();
        let object_b: Map<String, Value> = json!({
            "type_diff_a_string_b_int": 2,
            "value_diff": "b",
            "array_diff_same_order": [
                1, 2, 6, 4, 5
            ],
            "nested_object": {
                "type_diff_a_string_b_int": 2,
                "value_diff": "b",
                "array_diff_same_order": [
                    1, 2, 6, 4, 5
                ],
            }
        })
        .as_object()
        .unwrap()
        .to_owned();

        let expected_key_diffs = vec![
            KeyDiff {
                key: "key_diff_a_has".to_string(),
                has: "a.json".to_string(),
                misses: "b.json".to_string(),
            },
            KeyDiff {
                key: "nested_object.key_diff_a_has".to_string(),
                has: "a.json".to_string(),
                misses: "b.json".to_string(),
            },
        ];

        let expected_type_diffs = vec![
            TypeDiff {
                key: "type_diff_a_string_b_int".to_string(),
                type1: "string".to_string(),
                type2: "number".to_string(),
            },
            TypeDiff {
                key: "nested_object.type_diff_a_string_b_int".to_string(),
                type1: "string".to_string(),
                type2: "number".to_string(),
            },
        ];

        let expected_value_diffs = vec![
            ValueDiff {
                key: "value_diff".to_string(),
                value1: "a".to_string(),
                value2: "b".to_string(),
            },
            ValueDiff {
                key: "nested_object.value_diff".to_string(),
                value1: "a".to_string(),
                value2: "b".to_string(),
            },
            ValueDiff {
                key: "array_diff_same_order[2]".to_string(),
                value1: 3.to_string(),
                value2: 6.to_string(),
            },
            ValueDiff {
                key: "nested_object.array_diff_same_order[2]".to_string(),
                value1: 3.to_string(),
                value2: 6.to_string(),
            },
        ];

        let expected_array_diffs: Vec<ArrayDiff> = vec![];

        let working_context = &WorkingContext {
            file_a: WorkingFile {
                name: "a.json".to_string(),
            },
            file_b: WorkingFile {
                name: "b.json".to_string(),
            },
        };

        // act
        let (key_diffs, type_diffs, value_diffs, array_diffs) =
            compare_objects("", &object_a, &object_b, working_context, true);

        // assert
        assert_eq!(key_diffs.len(), expected_key_diffs.len());
        assert!(key_diffs.iter().eq(expected_key_diffs.iter()));

        assert_eq!(type_diffs.len(), expected_type_diffs.len());
        assert!(type_diffs
            .iter()
            .all(|diff| expected_type_diffs.contains(diff)));

        assert_eq!(value_diffs.len(), expected_value_diffs.len());
        assert!(value_diffs
            .iter()
            .all(|diff| expected_value_diffs.contains(diff)));

        assert_eq!(array_diffs.len(), expected_array_diffs.len());
        assert!(array_diffs
            .iter()
            .all(|diff| expected_array_diffs.contains(diff)));
    }

    #[test]
    fn test_compare_objects_array_same_order_different_length() {
        // arrange
        let object_a: Map<String, Value> = json!({
            "key_diff_a_has": "key_diff_a_has",
            "type_diff_a_string_b_int": "type_diff_a_string_b_int",
            "value_diff": "a",
            "array_diff_same_order": [
                1, 2, 3, 4, 5, 8
            ],
            "nested_object": {
                "key_diff_a_has": "key_diff_a_has",
                "type_diff_a_string_b_int": "type_diff_a_string_b_int",
                "value_diff": "a",
                "array_diff_same_order": [
                    1, 2, 3, 4, 5, 8
                ],
            }
        })
        .as_object()
        .unwrap()
        .to_owned();
        let object_b: Map<String, Value> = json!({
            "type_diff_a_string_b_int": 2,
            "value_diff": "b",
            "array_diff_same_order": [
                1, 2, 6, 4, 5
            ],
            "nested_object": {
                "type_diff_a_string_b_int": 2,
                "value_diff": "b",
                "array_diff_same_order": [
                    1, 2, 6, 4, 5
                ],
            }
        })
        .as_object()
        .unwrap()
        .to_owned();

        let expected_key_diffs = vec![
            KeyDiff {
                key: "key_diff_a_has".to_string(),
                has: "a.json".to_string(),
                misses: "b.json".to_string(),
            },
            KeyDiff {
                key: "nested_object.key_diff_a_has".to_string(),
                has: "a.json".to_string(),
                misses: "b.json".to_string(),
            },
        ];

        let expected_type_diffs = vec![
            TypeDiff {
                key: "type_diff_a_string_b_int".to_string(),
                type1: "string".to_string(),
                type2: "number".to_string(),
            },
            TypeDiff {
                key: "nested_object.type_diff_a_string_b_int".to_string(),
                type1: "string".to_string(),
                type2: "number".to_string(),
            },
        ];

        let expected_value_diffs = vec![
            ValueDiff {
                key: "value_diff".to_string(),
                value1: "a".to_string(),
                value2: "b".to_string(),
            },
            ValueDiff {
                key: "nested_object.value_diff".to_string(),
                value1: "a".to_string(),
                value2: "b".to_string(),
            },
        ];

        let expected_array_diffs: Vec<ArrayDiff> = vec![
            ArrayDiff {
                key: "array_diff_same_order".to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: "8".to_string(),
            },
            ArrayDiff {
                key: "array_diff_same_order".to_string(),
                descriptor: ArrayDiffDesc::BMisses,
                value: "8".to_string(),
            },
            ArrayDiff {
                key: "nested_object.array_diff_same_order".to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: "8".to_string(),
            },
            ArrayDiff {
                key: "nested_object.array_diff_same_order".to_string(),
                descriptor: ArrayDiffDesc::BMisses,
                value: "8".to_string(),
            },
            ArrayDiff {
                key: "array_diff_same_order".to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: "3".to_string(),
            },
            ArrayDiff {
                key: "array_diff_same_order".to_string(),
                descriptor: ArrayDiffDesc::BMisses,
                value: "3".to_string(),
            },
            ArrayDiff {
                key: "nested_object.array_diff_same_order".to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: "3".to_string(),
            },
            ArrayDiff {
                key: "nested_object.array_diff_same_order".to_string(),
                descriptor: ArrayDiffDesc::BMisses,
                value: "3".to_string(),
            },
            ArrayDiff {
                key: "array_diff_same_order".to_string(),
                descriptor: ArrayDiffDesc::BHas,
                value: "6".to_string(),
            },
            ArrayDiff {
                key: "array_diff_same_order".to_string(),
                descriptor: ArrayDiffDesc::AMisses,
                value: "6".to_string(),
            },
            ArrayDiff {
                key: "nested_object.array_diff_same_order".to_string(),
                descriptor: ArrayDiffDesc::BHas,
                value: "6".to_string(),
            },
            ArrayDiff {
                key: "nested_object.array_diff_same_order".to_string(),
                descriptor: ArrayDiffDesc::AMisses,
                value: "6".to_string(),
            },
        ];

        let working_context = &WorkingContext {
            file_a: WorkingFile {
                name: "a.json".to_string(),
            },
            file_b: WorkingFile {
                name: "b.json".to_string(),
            },
        };

        // act
        let (key_diffs, type_diffs, value_diffs, array_diffs) =
            compare_objects("", &object_a, &object_b, working_context, true);

        // assert
        assert_eq!(key_diffs.len(), expected_key_diffs.len());
        assert!(key_diffs.iter().eq(expected_key_diffs.iter()));

        assert_eq!(type_diffs.len(), expected_type_diffs.len());
        assert!(type_diffs
            .iter()
            .all(|diff| expected_type_diffs.contains(diff)));

        assert_eq!(value_diffs.len(), expected_value_diffs.len());
        assert!(value_diffs
            .iter()
            .all(|diff| expected_value_diffs.contains(diff)));

        assert_eq!(array_diffs.len(), expected_array_diffs.len());
        assert!(array_diffs
            .iter()
            .all(|diff| expected_array_diffs.contains(diff)));
    }

    #[test]
    fn test_compare_objects_array_different_order_same_length() {
        // arrange
        let object_a: Map<String, Value> = json!({
            "key_diff_a_has": "key_diff_a_has",
            "type_diff_a_string_b_int": "type_diff_a_string_b_int",
            "value_diff": "a",
            "array_diff_different_order": [
                1, 2, 3, 4, 5
            ],
            "nested_object": {
                "key_diff_a_has": "key_diff_a_has",
                "type_diff_a_string_b_int": "type_diff_a_string_b_int",
                "value_diff": "a",
                "array_diff_different_order": [
                    1, 2, 3, 4, 5
                ],
            }
        })
        .as_object()
        .unwrap()
        .to_owned();
        let object_b: Map<String, Value> = json!({
            "type_diff_a_string_b_int": 2,
            "value_diff": "b",
            "array_diff_different_order": [
                1, 2, 6, 4, 5
            ],
            "nested_object": {
                "type_diff_a_string_b_int": 2,
                "value_diff": "b",
                "array_diff_different_order": [
                    1, 2, 6, 4, 5
                ],
            }
        })
        .as_object()
        .unwrap()
        .to_owned();

        let expected_key_diffs = vec![
            KeyDiff {
                key: "key_diff_a_has".to_string(),
                has: "a.json".to_string(),
                misses: "b.json".to_string(),
            },
            KeyDiff {
                key: "nested_object.key_diff_a_has".to_string(),
                has: "a.json".to_string(),
                misses: "b.json".to_string(),
            },
        ];

        let expected_type_diffs = vec![
            TypeDiff {
                key: "type_diff_a_string_b_int".to_string(),
                type1: "string".to_string(),
                type2: "number".to_string(),
            },
            TypeDiff {
                key: "nested_object.type_diff_a_string_b_int".to_string(),
                type1: "string".to_string(),
                type2: "number".to_string(),
            },
        ];

        let expected_value_diffs = vec![
            ValueDiff {
                key: "value_diff".to_string(),
                value1: "a".to_string(),
                value2: "b".to_string(),
            },
            ValueDiff {
                key: "nested_object.value_diff".to_string(),
                value1: "a".to_string(),
                value2: "b".to_string(),
            },
        ];

        let expected_array_diffs: Vec<ArrayDiff> = vec![
            ArrayDiff {
                key: "array_diff_different_order".to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: "3".to_string(),
            },
            ArrayDiff {
                key: "array_diff_different_order".to_string(),
                descriptor: ArrayDiffDesc::BMisses,
                value: "3".to_string(),
            },
            ArrayDiff {
                key: "nested_object.array_diff_different_order".to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: "3".to_string(),
            },
            ArrayDiff {
                key: "nested_object.array_diff_different_order".to_string(),
                descriptor: ArrayDiffDesc::BMisses,
                value: "3".to_string(),
            },
            ArrayDiff {
                key: "array_diff_different_order".to_string(),
                descriptor: ArrayDiffDesc::BHas,
                value: "6".to_string(),
            },
            ArrayDiff {
                key: "array_diff_different_order".to_string(),
                descriptor: ArrayDiffDesc::AMisses,
                value: "6".to_string(),
            },
            ArrayDiff {
                key: "nested_object.array_diff_different_order".to_string(),
                descriptor: ArrayDiffDesc::BHas,
                value: "6".to_string(),
            },
            ArrayDiff {
                key: "nested_object.array_diff_different_order".to_string(),
                descriptor: ArrayDiffDesc::AMisses,
                value: "6".to_string(),
            },
        ];

        let working_context = &WorkingContext {
            file_a: WorkingFile {
                name: "a.json".to_string(),
            },
            file_b: WorkingFile {
                name: "b.json".to_string(),
            },
        };

        // act
        let (key_diffs, type_diffs, value_diffs, array_diffs) =
            compare_objects("", &object_a, &object_b, working_context, false);

        // assert
        assert_eq!(key_diffs.len(), expected_key_diffs.len());
        assert!(key_diffs.iter().eq(expected_key_diffs.iter()));

        assert_eq!(type_diffs.len(), expected_type_diffs.len());
        assert!(type_diffs
            .iter()
            .all(|diff| expected_type_diffs.contains(diff)));

        assert_eq!(value_diffs.len(), expected_value_diffs.len());
        assert!(value_diffs
            .iter()
            .all(|diff| expected_value_diffs.contains(diff)));

        assert_eq!(array_diffs.len(), expected_array_diffs.len());
        assert!(array_diffs
            .iter()
            .all(|diff| expected_array_diffs.contains(diff)));
    }

    #[test]
    fn test_handle_different_types_returns_type_diff_vec() {
        // arrange
        let a = json!(5);
        let b = json!("2");

        let expected = vec![TypeDiff {
            key: "key".to_string(),
            type1: "number".to_string(),
            type2: "string".to_string(),
        }];

        // act
        let result = handle_different_types("key", &a, &b);

        // assert
        assert_eq!(result.len(), expected.len());
        assert!(result.iter().eq(expected.iter()));
    }

    #[test]
    fn test_handle_one_element_null_primitives_returns_a_if_b_is_null() {
        // arrange
        let a = json!("something");
        let b = json!(null);

        let expected = vec![ValueDiff {
            key: "key".to_string(),
            value1: "something".to_string(),
            value2: "".to_string(),
        }];

        // act
        let result = handle_one_element_null_primitives("key", &a, &b);

        assert_eq!(result.len(), expected.len());
        assert!(result.iter().eq(expected.iter()));
    }

    #[test]
    fn test_handle_one_element_null_primitives_returns_b_if_a_is_null() {
        // arrange
        let a = json!(null);
        let b = json!("something");

        let expected = vec![ValueDiff {
            key: "key".to_string(),
            value1: "".to_string(),
            value2: "something".to_string(),
        }];

        // act
        let result = handle_one_element_null_primitives("key", &a, &b);

        assert_eq!(result.len(), expected.len());
        assert!(result.iter().eq(expected.iter()));
    }

    #[test]
    fn test_handle_one_element_null_arrays_returns_a_if_b_is_null() {
        // arrange
        let a = json!(vec!["something", "anything"]);
        let b = json!(null);

        let expected = vec![
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: "something".to_string(),
            },
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: "anything".to_string(),
            },
        ];

        // act
        let result = handle_one_element_null_arrays("key", &a, &b);

        assert_eq!(result.len(), expected.len());
        assert!(result.iter().eq(expected.iter()));
    }

    #[test]
    fn test_handle_one_element_null_arrays_returns_b_if_a_is_null() {
        // arrange
        let a = json!(null);
        let b = json!(vec!["something", "anything"]);

        let expected = vec![
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::BHas,
                value: "something".to_string(),
            },
            ArrayDiff {
                key: "key".to_string(),
                descriptor: ArrayDiffDesc::BHas,
                value: "anything".to_string(),
            },
        ];

        // act
        let result = handle_one_element_null_arrays("key", &a, &b);

        // assert
        assert_eq!(result.len(), expected.len());
        assert!(result.iter().eq(expected.iter()));
    }

    #[test]
    fn test_handle_one_element_null_objects_return_a_if_b_is_null() {
        // arrange
        let a = json!({ "key": "something" });
        let b = json!(null);
        let working_context = create_test_working_context();

        // act
        let (key_diff, type_diff, value_diff, array_diff) =
            handle_one_element_null_objects("parent_key", a, b, &working_context);

        // assert
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
        // arrange
        let a = json!(null);
        let b = json!({ "key": "something" });
        let working_context = create_test_working_context();

        // act
        let (key_diff, type_diff, value_diff, array_diff) =
            handle_one_element_null_objects("parent_key", a, b, &working_context);

        // assert
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

    #[test]
    fn test_compare_primitives_returns_empty_vec_if_equal() {
        // arrange
        let a = json!(2);
        let b = json!(2);

        let expected = vec![];

        // act
        let result = compare_primitives("key", &a, &b);

        // assert
        assert_eq!(result.len(), expected.len());
        assert!(result.iter().eq(expected.iter()));
    }

    #[test]
    fn test_compare_primitives_returns_correct_diff_vec() {
        // arrange
        let a = json!(4);
        let b = json!(2);

        let expected = vec![ValueDiff {
            key: "key".to_string(),
            value1: "4".to_string(),
            value2: "2".to_string(),
        }];

        // act
        let result = compare_primitives("key", &a, &b);

        // assert
        assert_eq!(result.len(), expected.len());
        assert!(result.iter().eq(expected.iter()));
    }

    // Test utils

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
