use serde_json::{Map, Result, Value};
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;

mod compare_field;
pub mod diff_types;

use diff_types::{
    ArrayDiff, ArrayDiffDesc, KeyDiff, TypeDiff, ValueDiff, ValueType, WorkingContext,
};

pub fn read_json_file(file_path: &str) -> Result<Map<String, Value>> {
    let file = File::open(file_path).expect(&format!("Could not open file {}", file_path));
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)?;
    Ok(result)
}

pub fn find_key_diffs<'a>(
    key_in: &'a str,
    a: &Map<String, Value>,
    b: &Map<String, Value>,
    working_context: &WorkingContext,
) -> Vec<KeyDiff> {
    let mut key_diff = vec![];

    let mut b_keys = HashSet::new();
    for b_key in b.keys() {
        b_keys.insert(b_key);
    }

    for (a_key, a_value) in a.into_iter() {
        let key = if key_in.is_empty() {
            a_key.to_string()
        } else {
            format!("{}.{}", key_in, a_key)
        };

        if let Some(b_value) = b.get(a_key) {
            b_keys.remove(a_key);

            key_diff.append(&mut find_key_diffs_in_values(
                &key,
                a_value,
                b_value,
                working_context,
            ));
        } else {
            key_diff.push(KeyDiff {
                key,
                has: working_context.file_a.name.clone(),
                misses: working_context.file_b.name.clone(),
            });
        }
    }

    let mut remainder = b_keys
        .into_iter()
        .map(|key| {
            KeyDiff::new(
                key.to_owned(),
                working_context.file_b.name.to_owned(),
                working_context.file_a.name.to_owned(),
            )
        })
        .collect();

    key_diff.append(&mut remainder);

    key_diff
}

fn find_key_diffs_in_values(
    key_in: &str,
    a: &Value,
    b: &Value,
    working_context: &WorkingContext,
) -> Vec<KeyDiff> {
    let mut key_diff = vec![];

    if a.is_object() && b.is_object() {
        key_diff.append(&mut find_key_diffs(
            &key_in,
            a.as_object().unwrap(),
            b.as_object().unwrap(),
            working_context,
        ));
    }

    if working_context.config.array_same_order
        && a.is_array()
        && b.is_array()
        && a.as_array().unwrap().len() == b.as_array().unwrap().len()
    {
        for (index, _) in a.as_array().unwrap().into_iter().enumerate() {
            let array_key = format!("{}[{}]", key_in, index);
            key_diff.append(&mut find_key_diffs_in_values(
                &array_key,
                a,
                b,
                working_context,
            ));
        }
    }

    key_diff
}

pub fn find_type_diffs<'a>(
    key_in: &'a str,
    a: &Map<String, Value>,
    b: &Map<String, Value>,
    working_context: &WorkingContext,
) -> Vec<TypeDiff> {
    let mut type_diff = vec![];

    for (a_key, a_value) in a.into_iter() {
        if let Some(b_value) = b.get(a_key) {
            let key = if key_in.is_empty() {
                a_key.to_string()
            } else {
                format!("{}.{}", key_in, a_key)
            };

            type_diff.append(&mut find_type_diffs_in_values(
                &key,
                a_value,
                b_value,
                working_context,
            ))
        }
    }

    type_diff
}

fn find_type_diffs_in_values(
    key_in: &str,
    a: &Value,
    b: &Value,
    working_context: &WorkingContext,
) -> Vec<TypeDiff> {
    let mut type_diff = vec![];

    if a.is_object() && b.is_object() {
        type_diff.append(&mut find_type_diffs(
            &key_in,
            a.as_object().unwrap(),
            b.as_object().unwrap(),
            working_context,
        ));
    }

    if working_context.config.array_same_order
        && a.is_array()
        && b.is_array()
        && a.as_array().unwrap().len() == b.as_array().unwrap().len()
    {
        for (index, _) in a.as_array().unwrap().into_iter().enumerate() {
            let array_key = format!("{}[{}]", key_in, index);
            type_diff.append(&mut find_type_diffs_in_values(
                &array_key,
                a,
                b,
                working_context,
            ));
        }
    }

    let a_type = get_type(a);
    let b_type = get_type(b);

    if a_type != b_type {
        type_diff.push(TypeDiff::new(
            key_in.to_owned(),
            a_type.to_string(),
            b_type.to_string(),
        ));
    }

    type_diff
}

pub fn find_value_diffs<'a>(
    key_in: &'a str,
    a: &Map<String, Value>,
    b: &Map<String, Value>,
    working_context: &WorkingContext,
) -> Vec<ValueDiff> {
    let mut value_diff = vec![];

    for (a_key, a_value) in a.into_iter() {
        if let Some(b_value) = b.get(a_key) {
            let key = if key_in.is_empty() {
                a_key.to_string()
            } else {
                format!("{}.{}", key_in, a_key)
            };

            value_diff.append(&mut find_value_diffs_in_values(
                &key,
                a_value,
                b_value,
                working_context,
            ));
        }
    }

    value_diff
}

fn find_value_diffs_in_values<'a>(
    key_in: &'a str,
    a: &'a Value,
    b: &'a Value,
    working_context: &WorkingContext,
) -> Vec<ValueDiff> {
    let mut value_diff = vec![];
    if a.is_object() && b.is_object() {
        value_diff.append(&mut find_value_diffs(
            &key_in,
            a.as_object().unwrap(),
            b.as_object().unwrap(),
            working_context,
        ));
    }

    if working_context.config.array_same_order
        && a.is_array()
        && b.is_array()
        && a.as_array().unwrap().len() == b.as_array().unwrap().len()
    {
        for (index, item) in a.as_array().unwrap().into_iter().enumerate() {
            let array_key = format!("{}[{}]", key_in, index);
            value_diff.append(&mut find_value_diffs_in_values(
                &array_key,
                &item,
                &b[&index],
                working_context,
            ));
        }
    }

    if a != b {
        value_diff.push(ValueDiff::new(
            key_in.to_owned(),
            a.to_string(),
            b.to_string(),
        ));
    }

    value_diff
}

pub fn find_array_diffs<'a>(
    key_in: &'a str,
    a: &Map<String, Value>,
    b: &Map<String, Value>,
    working_context: &WorkingContext,
) -> Vec<ArrayDiff> {
    if working_context.config.array_same_order {
        return vec![];
    }

    let mut array_diff = vec![];

    for (a_key, a_value) in a.into_iter() {
        if let Some(b_value) = b.get(a_key) {
            let key = if key_in.is_empty() {
                a_key.to_string()
            } else {
                format!("{}.{}", key_in, a_key)
            };

            array_diff.append(&mut find_array_diffs_in_values(
                &key,
                a_value,
                b_value,
                working_context,
            ));
        }
    }

    array_diff
}

fn find_array_diffs_in_values(
    key_in: &str,
    a: &Value,
    b: &Value,
    working_context: &WorkingContext,
) -> Vec<ArrayDiff> {
    let mut array_diff = vec![];

    if a.is_object() && b.is_object() {
        array_diff.append(&mut find_array_diffs(
            &key_in,
            a.as_object().unwrap(),
            b.as_object().unwrap(),
            working_context,
        ));
    }

    if a.is_array() && b.is_array() {
        let (a_has, a_misses, b_has, b_misses) =
            fill_diff_vectors(&a.as_array().unwrap(), b.as_array().unwrap());

        for (value, desc) in a_has
            .iter()
            .map(|v| (v, ArrayDiffDesc::AHas))
            .chain(a_misses.iter().map(|v| (v, ArrayDiffDesc::AMisses)))
            .chain(b_has.iter().map(|v| (v, ArrayDiffDesc::BHas)))
            .chain(b_misses.iter().map(|v| (v, ArrayDiffDesc::BMisses)))
        {
            array_diff.push(ArrayDiff {
                key: key_in.to_owned(),
                descriptor: desc,
                value: value.to_string(),
            });
        }
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
/*
#[cfg(test)]
mod tests {
    use serde_json::{json, Map, Value};

    use crate::{
        compare_arrays, compare_objects, compare_primitives,
        diff_types::{
            ArrayDiff, ArrayDiffDesc, Config, KeyDiff, TypeDiff, ValueDiff, WorkingContext,
            WorkingFile,
        },
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

        let working_context =
            create_test_working_context_with_file_names("file_a.json", "file_b.json", false);

        // act
        let (_key_diff, _type_diff, _value_diff, array_diff) =
            compare_arrays("key", &arr_a, &arr_b, &working_context);

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

        let working_context =
            create_test_working_context_with_file_names("file_a.json", "file_b.json", true);

        // act
        let (_key_diff, _type_diff, _value_diff, array_diff) =
            compare_arrays("key", &arr_a, &arr_b, &working_context);

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

        let working_context =
            create_test_working_context_with_file_names("file_a.json", "file_b.json", true);

        // act
        let (_key_diff, _type_diff, value_diff, _array_diff) =
            compare_arrays("key", &arr_a, &arr_b, &working_context);

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

        let working_context = create_test_working_context_with_file_names("a.json", "b.json", true);

        // act
        let (key_diffs, type_diffs, value_diffs, array_diffs) =
            compare_objects("", &object_a, &object_b, &working_context);

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

        let working_context = create_test_working_context_with_file_names("a.json", "b.json", true);

        // act
        let (key_diffs, type_diffs, value_diffs, array_diffs) =
            compare_objects("", &object_a, &object_b, &working_context);

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

        let working_context =
            create_test_working_context_with_file_names("a.json", "b.json", false);

        // act
        let (key_diffs, type_diffs, value_diffs, array_diffs) =
            compare_objects("", &object_a, &object_b, &working_context);

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
        create_test_working_context_with_file_names("test1.json", "test2.json", false)
    }

    fn create_test_working_context_with_file_names(
        file_name_a: &str,
        file_name_b: &str,
        array_same_order: bool,
    ) -> WorkingContext {
        WorkingContext {
            file_a: WorkingFile {
                name: file_name_a.to_string(),
            },
            file_b: WorkingFile {
                name: file_name_b.to_string(),
            },
            config: Config { array_same_order },
        }
    }
}
*/
