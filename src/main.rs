use serde_json::{json, Map, Number, Result, Value};
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;

enum ArrayDiffDesc {
    AHas,
    AMisses,
    BHas,
    BMisses,
}

struct KeyDiff {
    key: String,
    has: String,
    misses: String,
}

impl KeyDiff {
    fn get_formatted_string(&self) -> String {
        format!(
            "\nKey diff: key: {}, has: {}, misses: {}\n",
            self.key, self.has, self.misses
        )
    }
}

struct ValueDiff {
    key: String,
    value1: String,
    value2: String,
}

impl ValueDiff {
    fn get_formatted_string(&self) -> String {
        format!(
            "\nValue diff: key: {}, value1: {}, value2: {}\n",
            self.key, self.value1, self.value2
        )
    }
}

struct ArrayDiff {
    key: String,
    descriptor: ArrayDiffDesc,
    value: String,
}

impl ArrayDiff {
    fn get_formatted_string(&self) -> String {
        format!(
            "\nArray diff: key: {}, Description: {}, value: {}\n",
            self.key, self.descriptor, self.value
        )
    }
}

struct TypeDiff {
    key: String,
    type1: String,
    type2: String,
}

impl TypeDiff {
    fn get_formatted_string(&self) -> String {
        format!(
            "\nType diff: key: {}, type1: {}, type2: {}\n",
            self.key, self.type1, self.type2
        )
    }
}

fn main() -> Result<()> {
    let file_name1 = "test_data/person3.json";
    let file_name2 = "test_data/person4.json";
    let data1 = read_json_file(file_name1)?;
    let data2 = read_json_file(file_name2)?;
    let (key_diff, type_diff, value_diff) = compare_objects(
        file_name1.to_string(),
        file_name2.to_string(),
        &data1,
        &data2,
    );

    for ele in key_diff {
        print!("{}", ele.get_formatted_string());
    }

    for ele in type_diff {
        print!("{}", ele.get_formatted_string());
    }

    for ele in value_diff {
        print!("{}", ele.get_formatted_string());
    }
    Ok(())
}

fn read_json_file(file_path: &str) -> Result<Map<String, Value>> {
    let file = File::open(file_path).expect(&format!("Could not open file {}", file_path));
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)?;
    Ok(result)
}

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

fn handle_one_element_null_arrays<'a>(key: &'a str, a: Value, b: Value) -> Vec<ValueDiff> {
    let mut value_diff = vec![];

    if a.is_null() {
        // b should always be an array, because the function is called from the appropriate match arm
        for b_item in b.as_array().unwrap() {
            value_diff.push(ValueDiff {
                key: key.to_string(),
                value1: "".to_string(),
                value2: b_item.to_string(),
            });
        }
    } else {
        // a should always be an array, because the function is called from the appropriate match arm
        for a_item in a.as_array().unwrap() {
            value_diff.push(ValueDiff {
                key: key.to_string(),
                value1: a_item.to_string(),
                value2: "".to_string(),
            });
        }
    }

    value_diff
}

fn handle_one_element_null_objects<'a>(
    a: Value,
    b: Value,
) -> (Vec<KeyDiff>, Vec<TypeDiff>, Vec<ValueDiff>) {
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
            has: if a.is_null() {
                "b".to_string()
            } else {
                "a".to_string()
            },
            misses: if a.is_null() {
                "a".to_string()
            } else {
                "b".to_string()
            },
        });

        type_diff.push(TypeDiff {
            key: key.to_string(),
            type1: "".to_string(),
            type2: get_type(value),
        });

        value_diff.push(ValueDiff {
            key: key.to_string(),
            value1: if a.is_null() {
                "".to_string()
            } else {
                value.to_string()
            },
            value2: if a.is_null() {
                value.to_string()
            } else {
                "".to_string()
            },
        });
    }

    (key_diff, type_diff, value_diff)
}

fn handle_different_types<'a>(key: &'a str, a: Value, b: Value) -> Vec<TypeDiff> {
    vec![TypeDiff {
        key: key.to_string(),
        type1: get_type(&a),
        type2: get_type(&b),
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

fn compare_arrays<'a>(
    key: &'a str,
    a: &'a Vec<Value>,
    b: &'a Vec<Value>,
) -> (Vec<ArrayDiff>, Vec<ValueDiff>) {
    let mut value_diff = vec![];
    let mut array_diff: Vec<ArrayDiff> = vec![];
    let same_order = true; // TODO: this should be configurable

    if a.len() == b.len() {
        if same_order {
            for (i, a_item) in a.iter().enumerate() {
                let mut item_value_diff = compare_primitives(key, a_item, &b[i]);
                value_diff.append(&mut item_value_diff);
            }
        } else {
            array_diff = handle_different_order_arrays(a, b, key);
        }
    } else {
        array_diff = handle_different_order_arrays(a, b, key);
    }

    (array_diff, value_diff)
}

fn handle_different_order_arrays(a: &Vec<Value>, b: &Vec<Value>, key: &str) -> Vec<ArrayDiff> {
    let mut array_diff = vec![];

    let (a_has, a_misses, b_has, b_misses) = fill_diff_vectors(a, b);
    for ah in a_has.iter() {
        array_diff.push(ArrayDiff {
            key: key.to_string(),
            descriptor: ArrayDiffDesc::AHas,
            value: ah.to_string(),
        });
    }

    for am in a_misses.iter() {
        array_diff.push(ArrayDiff {
            key: key.to_string(),
            descriptor: ArrayDiffDesc::AMisses,
            value: am.to_string(),
        });
    }

    for bh in b_has.iter() {
        array_diff.push(ArrayDiff {
            key: key.to_string(),
            descriptor: ArrayDiffDesc::BHas,
            value: bh.to_string(),
        });
    }

    for bm in b_misses.iter() {
        array_diff.push(ArrayDiff {
            key: key.to_string(),
            descriptor: ArrayDiffDesc::BMisses,
            value: bm.to_string(),
        });
    }

    array_diff
}

fn fill_diff_vectors<'a, T: PartialEq + Display>(
    a: &'a [T],
    b: &'a [T],
) -> (Vec<&'a T>, Vec<&'a T>, Vec<&'a T>, Vec<&'a T>) {
    let mut a_has = vec![];
    let mut a_misses = vec![];
    let mut b_has = vec![];

    for item in a {
        if b.contains(item) {
            a_has.push(item);
        } else {
            a_misses.push(item);
        }
    }

    for item in b {
        if !a.contains(item) {
            b_has.push(item);
        }
    }

    let mut a_misses_out = vec![];
    for item in &a_misses {
        if !b.contains(item) {
            a_misses_out.push(item.to_owned());
        }
    }

    let mut b_misses_out = vec![];
    for item in &b_has {
        if !a.contains(item) {
            b_misses_out.push(item.to_owned());
        }
    }

    (a_has, a_misses_out, b_has, b_misses_out)
}

fn compare_objects<'a>(
    a_name: String,
    b_name: String,
    a: &'a Map<String, Value>,
    b: &'a Map<String, Value>,
) -> (Vec<KeyDiff>, Vec<TypeDiff>, Vec<ValueDiff>) {
    let mut key_diff = vec![];
    let mut type_diff = vec![];
    let mut value_diff = vec![];

    for (a_key, a_value) in a.iter() {
        //Comparing keys
        if let Some(b_value) = b.get(a_key) {
            let key_start = a_name.split("json").last().unwrap_or("");
            let key = if key_start.is_empty() {
                a_key.to_string()
            } else {
                format!("{}.{}", key_start, a_key)
            };
            let (mut field_key_diff, mut field_type_diff, mut field_value_diff) =
                compare_field(key, a_value, b_value);

            key_diff.append(&mut field_key_diff);
            type_diff.append(&mut field_type_diff);
            value_diff.append(&mut field_value_diff);
        } else {
            key_diff.push(KeyDiff {
                key: a_key.to_string(),
                has: a_name.to_string(),
                misses: b_name.to_string(),
            });
        }
    }

    (key_diff, type_diff, value_diff)
}

fn compare_field<'a>(
    key: String,
    a_value: &'a Value,
    b_value: &'a Value,
) -> (Vec<KeyDiff>, Vec<TypeDiff>, Vec<ValueDiff>) {
    match (a_value, b_value) {
        // Primitives of same type
        (Value::Null, Value::Null) => (vec![], vec![], vec![]),
        (Value::String(a_value), Value::String(b_value)) => (
            vec![],
            vec![],
            compare_primitives(key.as_str(), a_value, b_value),
        ),
        (Value::Number(a_value), Value::Number(b_value)) => (
            vec![],
            vec![],
            compare_primitives(key.as_str(), a_value, b_value),
        ),
        (Value::Bool(a_value), Value::Bool(b_value)) => (
            vec![],
            vec![],
            compare_primitives(key.as_str(), a_value, b_value),
        ),

        // Composites of same type
        (Value::Array(a_value), Value::Array(b_value)) => (
            vec![],
            vec![],
            compare_arrays(key.as_str(), a_value, b_value),
        ),
        (Value::Object(a_value), Value::Object(b_value)) => {
            compare_objects(key.clone(), key, a_value, b_value)
        }

        // One value is null primitives
        (Value::Null, Value::String(b_value)) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(
                key.as_str(),
                a_value.clone(),
                json!(b_value).to_owned(),
            ),
        ),
        (Value::Null, Value::Number(b_value)) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(
                key.as_str(),
                a_value.clone(),
                json!(b_value).to_owned(),
            ),
        ),
        (Value::Null, Value::Bool(b_value)) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(
                key.as_str(),
                a_value.clone(),
                json!(b_value).to_owned(),
            ),
        ),

        (Value::String(a_value), Value::Null) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(
                key.as_str(),
                json!(a_value).to_owned(),
                b_value.clone(),
            ),
        ),
        (Value::Number(a_value), Value::Null) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(
                key.as_str(),
                json!(a_value).to_owned(),
                b_value.clone(),
            ),
        ),
        (Value::Bool(a_value), Value::Null) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(
                key.as_str(),
                json!(a_value).to_owned(),
                b_value.clone(),
            ),
        ),

        // One value is null, composites
        (Value::Null, Value::Array(b_value)) => (
            vec![],
            vec![],
            handle_one_element_null_arrays(
                key.as_str(),
                a_value.clone(),
                json!(b_value).to_owned(),
            ),
        ),
        (Value::Null, Value::Object(b_value)) => {
            handle_one_element_null_objects(a_value.clone(), json!(b_value).to_owned())
        }

        (Value::Array(a_value), Value::Null) => (
            vec![],
            vec![],
            handle_one_element_null_arrays(
                key.as_str(),
                json!(a_value).to_owned(),
                b_value.clone(),
            ),
        ),
        (Value::Object(a_value), Value::Null) => {
            handle_one_element_null_objects(json!(a_value).to_owned(), b_value.clone())
        }

        // Type difference a: string
        (Value::String(a_value), Value::Number(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::String(a_value), Value::Bool(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::String(a_value), Value::Array(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::String(a_value), Value::Object(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),

        // Type difference a: number
        (Value::Number(a_value), Value::String(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::Number(a_value), Value::Bool(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::Number(a_value), Value::Array(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::Number(a_value), Value::Object(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),

        // Type difference a: bool
        (Value::Bool(a_value), Value::String(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::Bool(a_value), Value::Number(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::Bool(a_value), Value::Array(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::Bool(a_value), Value::Object(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),

        // Type difference a: array
        (Value::Array(a_value), Value::String(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::Array(a_value), Value::Bool(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::Array(a_value), Value::Number(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::Array(a_value), Value::Object(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),

        // Type difference a: object
        (Value::Object(a_value), Value::String(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::Object(a_value), Value::Bool(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::Object(a_value), Value::Array(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::Object(a_value), Value::Number(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
    }
}

fn compare_primitives<'a, T: PartialEq + Display>(
    key: &'a str,
    a: &'a T,
    b: &'a T,
) -> Vec<ValueDiff> {
    let mut value_diff = vec![];

    if !a.eq(b) {
        value_diff.push(ValueDiff {
            key: key.to_string(),
            value1: a.to_string(),
            value2: b.to_string(),
        });
    }

    value_diff
}

fn get_array_diff_descriptor_str(diff_desc: ArrayDiffDesc) -> String {
    match diff_desc {
        ArrayDiffDesc::AHas => "a has".to_string(),
        ArrayDiffDesc::AMisses => "a misses".to_string(),
        ArrayDiffDesc::BHas => "b has".to_string(),
        ArrayDiffDesc::BMisses => "b_misses".to_string(),
    }
}
