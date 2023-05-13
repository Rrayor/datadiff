use serde_json::{Map, Result, Value};
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;

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
        b_keys.insert(format_key(key_in, &b_key));
    }

    for (a_key, a_value) in a.into_iter() {
        let key = format_key(key_in, &a_key);

        if let Some(b_value) = b.get(a_key) {
            b_keys.remove(&key);

            key_diff.append(&mut find_key_diffs_in_values(
                &key,
                a_value,
                b_value,
                working_context,
            ));
        } else {
            key_diff.push(KeyDiff::new(
                key,
                working_context.file_a.name.clone(),
                working_context.file_b.name.clone(),
            ));
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
    find_diff_in_values(
        a,
        b,
        working_context,
        || {
            find_key_diffs(
                &key_in,
                a.as_object().unwrap(),
                b.as_object().unwrap(),
                working_context,
            )
        },
        || {
            a.as_array()
                .unwrap()
                .into_iter()
                .enumerate()
                .flat_map(|(i, a_item)| {
                    find_key_diffs_in_values(
                        &format!("{}[{}]", key_in, i),
                        a_item,
                        &b.as_array().unwrap()[i],
                        working_context,
                    )
                })
                .collect()
        },
    )
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
            type_diff.append(&mut find_type_diffs_in_values(
                &format_key(key_in, &a_key),
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
    let mut type_diff = find_diff_in_values(
        a,
        b,
        working_context,
        || {
            find_type_diffs(
                &key_in,
                a.as_object().unwrap(),
                b.as_object().unwrap(),
                working_context,
            )
        },
        || {
            a.as_array()
                .unwrap()
                .into_iter()
                .enumerate()
                .flat_map(|(i, a_item)| {
                    find_type_diffs_in_values(
                        &format!("{}[{}]", key_in, i),
                        a_item,
                        &b.as_array().unwrap()[i],
                        working_context,
                    )
                })
                .collect()
        },
    );

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
            value_diff.append(&mut find_value_diffs_in_values(
                &format_key(key_in, &a_key),
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
    } else if working_context.config.array_same_order
        && a.is_array()
        && b.is_array()
        && a.as_array().unwrap().len() == b.as_array().unwrap().len()
    {
        for (index, a_item) in a.as_array().unwrap().into_iter().enumerate() {
            let array_key = format!("{}[{}]", key_in, index);
            value_diff.append(&mut find_value_diffs_in_values(
                &array_key,
                &a_item,
                &b.as_array().unwrap()[index],
                working_context,
            ));
        }
    } else if a != b {
        value_diff.push(ValueDiff::new(
            key_in.to_owned(),
            // String values are escaped by default if to_string() is called on them, so if it is a string, we call as_str() first.
            a.as_str().map_or_else(|| a.to_string(), |v| v.to_owned()),
            b.as_str().map_or_else(|| b.to_string(), |v| v.to_owned()),
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
            array_diff.append(&mut find_array_diffs_in_values(
                &format_key(key_in, &a_key),
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
    let mut array_diff = find_diff_in_values(
        a,
        b,
        working_context,
        || {
            find_array_diffs(
                &key_in,
                a.as_object().unwrap(),
                b.as_object().unwrap(),
                working_context,
            )
        },
        || vec![],
    );

    if a.is_array() && b.is_array() {
        let (a_has, a_misses, b_has, b_misses) =
            fill_diff_vectors(&a.as_array().unwrap(), b.as_array().unwrap());

        let array_diff_iter = a_has
            .iter()
            .map(|v| (v, ArrayDiffDesc::AHas))
            .chain(a_misses.iter().map(|v| (v, ArrayDiffDesc::AMisses)))
            .chain(b_has.iter().map(|v| (v, ArrayDiffDesc::BHas)))
            .chain(b_misses.iter().map(|v| (v, ArrayDiffDesc::BMisses)))
            .map(|(value, desc)| {
                ArrayDiff::new(
                    key_in.to_owned(),
                    desc,
                    value
                        .as_str()
                        .map_or_else(|| value.to_string(), |v| v.to_owned()),
                )
            });

        array_diff.extend(array_diff_iter);
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

fn find_diff_in_values<T, F, G>(
    a: &Value,
    b: &Value,
    working_context: &WorkingContext,
    run_if_objects: F,
    run_if_arrays: G,
) -> Vec<T>
where
    F: Fn() -> Vec<T>,
    G: Fn() -> Vec<T>,
{
    let mut diffs: Vec<T> = vec![];

    if a.is_object() && b.is_object() {
        diffs.append(&mut run_if_objects());
    }

    if working_context.config.array_same_order
        && a.is_array()
        && b.is_array()
        && a.as_array().unwrap().len() == b.as_array().unwrap().len()
    {
        diffs.append(&mut run_if_arrays());
    }

    diffs
}

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

fn format_key(key_in: &str, current_key: &str) -> String {
    if key_in.is_empty() {
        current_key.to_owned()
    } else {
        format!("{}.{}", key_in, current_key)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        diff_types::{
            ArrayDiff, ArrayDiffDesc, Config, KeyDiff, TypeDiff, ValueDiff, WorkingContext,
            WorkingFile,
        },
        find_array_diffs, find_key_diffs, find_type_diffs, find_value_diffs,
    };

    const FILE_NAME_A: &str = "a.json";
    const FILE_NAME_B: &str = "b.json";

    #[test]
    fn test_find_key_diffs() {
        // arrange
        let a = json!({
            "a_has": "a_has",
            "both_have": "both_have",
            "nested": {
                "a_has": "a_has",
                "both_have": "both_have"
            }
        });
        let b = json!({
            "b_has": "b_has",
            "both_have": "both_have",
            "nested": {
                "b_has": "b_has",
                "both_have": "both_have"
            }
        });

        let expected = vec![
            KeyDiff::new(
                "a_has".to_owned(),
                FILE_NAME_A.to_owned(),
                FILE_NAME_B.to_owned(),
            ),
            KeyDiff::new(
                "nested.a_has".to_owned(),
                FILE_NAME_A.to_owned(),
                FILE_NAME_B.to_owned(),
            ),
            KeyDiff::new(
                "b_has".to_owned(),
                FILE_NAME_B.to_owned(),
                FILE_NAME_A.to_owned(),
            ),
            KeyDiff::new(
                "nested.b_has".to_owned(),
                FILE_NAME_B.to_owned(),
                FILE_NAME_A.to_owned(),
            ),
        ];

        let working_context = create_test_working_context(false);

        // act
        let result = find_key_diffs(
            "",
            &a.as_object().unwrap(),
            &b.as_object().unwrap(),
            &working_context,
        );

        // assert
        assert_array(&expected, &result);
    }

    #[test]
    fn test_find_type_diffs_no_array_same_order() {
        // arrange
        let a = json!({
            "a_string_b_int": "a_string_b_int",
            "both_string": "both_string",
            "array_3_a_string_b_int": [
                "string",
                "string2",
                "string3",
                "string4",
                8,
                true
            ],
            "nested": {
                "a_bool_b_string": true,
                "both_number": 4,
                "array_3_a_int_b_bool": [
                    "string",
                    "string2",
                    "string3",
                    6,
                    8,
                    true
                ],
            }
        });
        let b = json!({
            "a_string_b_int": 2,
            "both_string": "both_string",
            "array_3_a_string_b_int": [
                "other_string",
                "other_string2",
                "other_string3",
                5,
                1,
                false
            ],
            "nested": {
                "a_bool_b_string": "a_bool_b_string",
                "both_number": 1,
                "array_3_a_int_b_bool": [
                "other_string",
                "other_string2",
                "other_string3",
                false,
                2,
                false
            ],
            }
        });

        let expected = vec![
            TypeDiff::new(
                "a_string_b_int".to_owned(),
                "string".to_owned(),
                "number".to_owned(),
            ),
            TypeDiff::new(
                "nested.a_bool_b_string".to_owned(),
                "bool".to_owned(),
                "string".to_owned(),
            ),
        ];

        let working_context = create_test_working_context(false);

        // act
        let result = find_type_diffs(
            "",
            &a.as_object().unwrap(),
            &b.as_object().unwrap(),
            &working_context,
        );

        // assert
        assert_array(&expected, &result);
    }

    #[test]
    fn test_find_type_diffs_array_same_order() {
        // arrange
        let a = json!({
            "a_string_b_int": "a_string_b_int",
            "both_string": "both_string",
            "array_3_a_string_b_int": [
                "string",
                "string2",
                "string3",
                "string4",
                8,
                true
            ],
            "nested": {
                "a_bool_b_string": true,
                "both_number": 4,
                "array_3_a_int_b_bool": [
                    "string",
                    "string2",
                    "string3",
                    6,
                    8,
                    true
                ],
            }
        });
        let b = json!({
            "a_string_b_int": 2,
            "both_string": "both_string",
            "array_3_a_string_b_int": [
                "other_string",
                "other_string2",
                "other_string3",
                5,
                1,
                false
            ],
            "nested": {
                "a_bool_b_string": "a_bool_b_string",
                "both_number": 1,
                "array_3_a_int_b_bool": [
                "other_string",
                "other_string2",
                "other_string3",
                false,
                2,
                false
            ],
            }
        });

        let expected = vec![
            TypeDiff::new(
                "a_string_b_int".to_owned(),
                "string".to_owned(),
                "number".to_owned(),
            ),
            TypeDiff::new(
                "nested.a_bool_b_string".to_owned(),
                "bool".to_owned(),
                "string".to_owned(),
            ),
            TypeDiff::new(
                "array_3_a_string_b_int[3]".to_owned(),
                "string".to_owned(),
                "number".to_owned(),
            ),
            TypeDiff::new(
                "nested.array_3_a_int_b_bool[3]".to_owned(),
                "number".to_owned(),
                "bool".to_owned(),
            ),
        ];

        let working_context = create_test_working_context(true);

        // act
        let result = find_type_diffs(
            "",
            &a.as_object().unwrap(),
            &b.as_object().unwrap(),
            &working_context,
        );

        // assert
        assert_array(&expected, &result);
    }

    #[test]
    fn test_find_value_diffs_no_array_same_order() {
        // arrange
        let a = json!({
            "no_diff_string": "no_diff_string",
            "diff_string": "a",
            "no_diff_number": "no_diff_number",
            "diff_number": 1,
            "no_diff_boolean": true,
            "diff_boolean": true,
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                1, 2, 3, 4
            ],
            "nested": {
                "no_diff_string": "no_diff_string",
                "diff_string": "a",
                "no_diff_number": "no_diff_number",
                "diff_number": 1,
                "no_diff_boolean": true,
                "diff_boolean": true,
                "no_diff_array": [
                    1, 2, 3, 4
                ],
                "diff_array": [
                    1, 2, 3, 4
                ],
            },
        });

        let b = json!({
            "no_diff_string": "no_diff_string",
            "diff_string": "b",
            "no_diff_number": "no_diff_number",
            "diff_number": 2,
            "no_diff_boolean": true,
            "diff_boolean": false,
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                5, 6, 7, 8
            ],
            "nested": {
                "no_diff_string": "no_diff_string",
                "diff_string": "b",
                "no_diff_number": "no_diff_number",
                "diff_number": 2,
                "no_diff_boolean": true,
                "diff_boolean": false,
                "no_diff_array": [
                    1, 2, 3, 4
                ],
                "diff_array": [
                    5, 6, 7, 8
                ],
            },
        });

        let expected = vec![
            ValueDiff::new("diff_string".to_owned(), "a".to_owned(), "b".to_owned()),
            ValueDiff::new("diff_number".to_owned(), "1".to_owned(), "2".to_owned()),
            ValueDiff::new(
                "diff_boolean".to_owned(),
                "true".to_owned(),
                "false".to_owned(),
            ),
            ValueDiff::new(
                "diff_array".to_owned(),
                "[1,2,3,4]".to_owned(),
                "[5,6,7,8]".to_owned(),
            ),
            ValueDiff::new(
                "nested.diff_string".to_owned(),
                "a".to_owned(),
                "b".to_owned(),
            ),
            ValueDiff::new(
                "nested.diff_number".to_owned(),
                "1".to_owned(),
                "2".to_owned(),
            ),
            ValueDiff::new(
                "nested.diff_boolean".to_owned(),
                "true".to_owned(),
                "false".to_owned(),
            ),
            ValueDiff::new(
                "nested.diff_array".to_owned(),
                "[1,2,3,4]".to_owned(),
                "[5,6,7,8]".to_owned(),
            ),
        ];

        let working_context = create_test_working_context(false);

        // act
        let result = find_value_diffs(
            "",
            &a.as_object().unwrap(),
            &b.as_object().unwrap(),
            &working_context,
        );

        // assert
        assert_array(&expected, &result);
    }

    #[test]
    fn test_find_value_diffs_array_same_order() {
        // arrange
        let a = json!({
            "no_diff_string": "no_diff_string",
            "diff_string": "a",
            "no_diff_number": "no_diff_number",
            "diff_number": 1,
            "no_diff_boolean": true,
            "diff_boolean": true,
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                1, 2, 3, 4
            ],
            "nested": {
                "no_diff_string": "no_diff_string",
                "diff_string": "a",
                "no_diff_number": "no_diff_number",
                "diff_number": 1,
                "no_diff_boolean": true,
                "diff_boolean": true,
                "no_diff_array": [
                    1, 2, 3, 4
                ],
                "diff_array": [
                    1, 2, 3, 4
                ],
            },
        });

        let b = json!({
            "no_diff_string": "no_diff_string",
            "diff_string": "b",
            "no_diff_number": "no_diff_number",
            "diff_number": 2,
            "no_diff_boolean": true,
            "diff_boolean": false,
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                1, 2, 8, 4
            ],
            "nested": {
                "no_diff_string": "no_diff_string",
                "diff_string": "b",
                "no_diff_number": "no_diff_number",
                "diff_number": 2,
                "no_diff_boolean": true,
                "diff_boolean": false,
                "no_diff_array": [
                    1, 2, 3, 4
                ],
                "diff_array": [
                    1, 2, 8, 4
                ],
            },
        });

        let expected = vec![
            ValueDiff::new("diff_string".to_owned(), "a".to_owned(), "b".to_owned()),
            ValueDiff::new("diff_number".to_owned(), "1".to_owned(), "2".to_owned()),
            ValueDiff::new(
                "diff_boolean".to_owned(),
                "true".to_owned(),
                "false".to_owned(),
            ),
            ValueDiff::new("diff_array[2]".to_owned(), "3".to_owned(), "8".to_owned()),
            ValueDiff::new(
                "nested.diff_string".to_owned(),
                "a".to_owned(),
                "b".to_owned(),
            ),
            ValueDiff::new(
                "nested.diff_number".to_owned(),
                "1".to_owned(),
                "2".to_owned(),
            ),
            ValueDiff::new(
                "nested.diff_boolean".to_owned(),
                "true".to_owned(),
                "false".to_owned(),
            ),
            ValueDiff::new(
                "nested.diff_array[2]".to_owned(),
                "3".to_owned(),
                "8".to_owned(),
            ),
        ];

        let working_context = create_test_working_context(true);

        // act
        let result = find_value_diffs(
            "",
            &a.as_object().unwrap(),
            &b.as_object().unwrap(),
            &working_context,
        );

        // assert
        assert_array(&expected, &result);
    }

    #[test]
    fn test_find_array_diffs() {
        // arrange
        let a = json!({
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                1, 2, 3, 4
            ],
            "nested": {
                "no_diff_array": [
                    1, 2, 3, 4
                ],
                "diff_array": [
                    1, 2, 3, 4
                ],
            },
        });

        let b = json!({
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                1, 2, 8, 4
            ],
            "nested": {
                "no_diff_array": [
                    1, 2, 3, 4
                ],
                "diff_array": [
                    1, 2, 8, 4
                ],
            },
        });

        let expected = vec![
            ArrayDiff::new("diff_array".to_owned(), ArrayDiffDesc::AHas, "3".to_owned()),
            ArrayDiff::new(
                "diff_array".to_owned(),
                ArrayDiffDesc::BMisses,
                "3".to_owned(),
            ),
            ArrayDiff::new("diff_array".to_owned(), ArrayDiffDesc::BHas, "8".to_owned()),
            ArrayDiff::new(
                "diff_array".to_owned(),
                ArrayDiffDesc::AMisses,
                "8".to_owned(),
            ),
            ArrayDiff::new(
                "nested.diff_array".to_owned(),
                ArrayDiffDesc::AHas,
                "3".to_owned(),
            ),
            ArrayDiff::new(
                "nested.diff_array".to_owned(),
                ArrayDiffDesc::BMisses,
                "3".to_owned(),
            ),
            ArrayDiff::new(
                "nested.diff_array".to_owned(),
                ArrayDiffDesc::BHas,
                "8".to_owned(),
            ),
            ArrayDiff::new(
                "nested.diff_array".to_owned(),
                ArrayDiffDesc::AMisses,
                "8".to_owned(),
            ),
        ];

        let working_context = create_test_working_context(false);

        // act
        let result = find_array_diffs(
            "",
            &a.as_object().unwrap(),
            &b.as_object().unwrap(),
            &working_context,
        );

        // assert
        assert_array(&expected, &result);
    }

    // Test utils

    fn create_test_working_context(array_same_order: bool) -> WorkingContext {
        let config = Config::new(array_same_order);
        let working_file_a = WorkingFile::new(FILE_NAME_A.to_owned());
        let working_file_b = WorkingFile::new(FILE_NAME_B.to_owned());
        WorkingContext::new(working_file_a, working_file_b, config)
    }

    fn assert_array<T: PartialEq>(expected: &Vec<T>, result: &Vec<T>) {
        assert_eq!(expected.len(), result.len());
        assert!(expected.into_iter().all(|item| result.contains(&item)));
    }
}
