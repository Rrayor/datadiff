use serde_json::{json, Map, Result, Value};
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;

#[derive(Copy, Clone)]
enum ArrayDiffDesc {
    AHas,
    AMisses,
    BHas,
    BMisses,
}

pub struct WorkingFile {
    pub name: String,
}

pub struct WorkingContext {
    pub file_a: WorkingFile,
    pub file_b: WorkingFile,
}

pub struct KeyDiff {
    key: String,
    has: String,
    misses: String,
}

impl KeyDiff {
    pub fn get_formatted_string(&self) -> String {
        format!(
            "\nKey diff: key: {}, has: {}, misses: {}\n",
            self.key, self.has, self.misses
        )
    }
}

pub struct ValueDiff {
    key: String,
    value1: String,
    value2: String,
}

impl ValueDiff {
    pub fn get_formatted_string(&self) -> String {
        format!(
            "\nValue diff: key: {}, value1: {}, value2: {}\n",
            self.key, self.value1, self.value2
        )
    }
}

pub struct ArrayDiff {
    key: String,
    descriptor: ArrayDiffDesc,
    value: String,
}

impl ArrayDiff {
    pub fn get_formatted_string(&self, working_context: &WorkingContext) -> String {
        format!(
            "\nArray diff: key: {}, Description: {}, value: {}\n",
            self.key,
            get_array_diff_descriptor_str(self.descriptor, working_context),
            self.value
        )
    }
}

pub struct TypeDiff {
    key: String,
    type1: String,
    type2: String,
}

impl TypeDiff {
    pub fn get_formatted_string(&self) -> String {
        format!(
            "\nType diff: key: {}, type1: {}, type2: {}\n",
            self.key, self.type1, self.type2
        )
    }
}

pub type ComparisionResult = (Vec<KeyDiff>, Vec<TypeDiff>, Vec<ValueDiff>, Vec<ArrayDiff>);

pub fn read_json_file(file_path: &str) -> Result<Map<String, Value>> {
    let file = File::open(file_path).expect(&format!("Could not open file {}", file_path));
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)?;
    Ok(result)
}

pub fn compare_objects<'a>(
    key_in: String,
    a: &'a Map<String, Value>,
    b: &'a Map<String, Value>,
    working_context: &WorkingContext,
) -> ComparisionResult {
    let mut key_diff = vec![];
    let mut type_diff = vec![];
    let mut value_diff = vec![];
    let mut array_diff = vec![];

    for (a_key, a_value) in a.iter() {
        let key = if key_in.is_empty() {
            a_key.to_string()
        } else {
            format!("{}.{}", key_in, a_key)
        };
        //Comparing keys
        if let Some(b_value) = b.get(a_key) {
            let (
                mut field_key_diff,
                mut field_type_diff,
                mut field_value_diff,
                mut field_array_diff,
            ) = compare_field(key, a_value, b_value, working_context);

            key_diff.append(&mut field_key_diff);
            type_diff.append(&mut field_type_diff);
            value_diff.append(&mut field_value_diff);
            array_diff.append(&mut field_array_diff);
        } else {
            key_diff.push(KeyDiff {
                key: key,
                has: working_context.file_a.name.clone(),
                misses: working_context.file_b.name.clone(),
            });
        }
    }

    (key_diff, type_diff, value_diff, array_diff)
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

fn handle_one_element_null_arrays<'a>(key: &'a str, a: Value, b: Value) -> Vec<ArrayDiff> {
    let mut array_diff = vec![];

    if a.is_null() {
        // b should always be an array, because the function is called from the appropriate match arm
        for b_item in b.as_array().unwrap() {
            array_diff.push(ArrayDiff {
                key: key.to_string(),
                descriptor: ArrayDiffDesc::BHas,
                value: b_item.to_string(),
            });
        }
    } else {
        // a should always be an array, because the function is called from the appropriate match arm
        for a_item in a.as_array().unwrap() {
            array_diff.push(ArrayDiff {
                key: key.to_string(),
                descriptor: ArrayDiffDesc::AHas,
                value: a_item.to_string(),
            });
        }
    }

    array_diff
}

fn handle_one_element_null_objects<'a>(
    parent_key: String,
    a: Value,
    b: Value,
    working_context: &WorkingContext,
) -> ComparisionResult {
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
            key: full_key,
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

    (key_diff, type_diff, value_diff, vec![]) // TODO: handle arrays here?
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
                    format!("{}[{}]", key.to_string(), i.to_string()),
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
            array_diff = handle_different_order_arrays(a, b, key);
        }
    } else {
        array_diff = handle_different_order_arrays(a, b, key);
    }

    (key_diff, type_diff, value_diff, array_diff)
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
    let mut b_misses = vec![];

    for item in a {
        if !b.contains(item) {
            a_has.push(item);
            b_misses.push(item);
        }
    }

    for item in b {
        if !a.contains(item) {
            b_has.push(item);
            a_misses.push(item);
        }
    }

    (a_has, a_misses, b_has, b_misses)
}

fn compare_field<'a>(
    key: String,
    a_value: &'a Value,
    b_value: &'a Value,
    working_context: &WorkingContext,
) -> ComparisionResult {
    match (a_value, b_value) {
        // Primitives of same type
        (Value::Null, Value::Null) => (vec![], vec![], vec![], vec![]),
        (Value::String(a_value), Value::String(b_value)) => (
            vec![],
            vec![],
            compare_primitives(key.as_str(), a_value, b_value),
            vec![],
        ),
        (Value::Number(a_value), Value::Number(b_value)) => (
            vec![],
            vec![],
            compare_primitives(key.as_str(), a_value, b_value),
            vec![],
        ),
        (Value::Bool(a_value), Value::Bool(b_value)) => (
            vec![],
            vec![],
            compare_primitives(key.as_str(), a_value, b_value),
            vec![],
        ),

        // Composites of same type
        (Value::Array(a_value), Value::Array(b_value)) => {
            compare_arrays(key.as_str(), a_value, b_value, working_context)
        }
        (Value::Object(a_value), Value::Object(b_value)) => {
            compare_objects(key, a_value, b_value, working_context)
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
            vec![],
        ),
        (Value::Null, Value::Number(b_value)) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(
                key.as_str(),
                a_value.clone(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),
        (Value::Null, Value::Bool(b_value)) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(
                key.as_str(),
                a_value.clone(),
                json!(b_value).to_owned(),
            ),
            vec![],
        ),

        (Value::String(a_value), Value::Null) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(
                key.as_str(),
                json!(a_value).to_owned(),
                b_value.clone(),
            ),
            vec![],
        ),
        (Value::Number(a_value), Value::Null) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(
                key.as_str(),
                json!(a_value).to_owned(),
                b_value.clone(),
            ),
            vec![],
        ),
        (Value::Bool(a_value), Value::Null) => (
            vec![],
            vec![],
            handle_one_element_null_primitives(
                key.as_str(),
                json!(a_value).to_owned(),
                b_value.clone(),
            ),
            vec![],
        ),

        // One value is null, composites
        (Value::Null, Value::Array(b_value)) => (
            vec![],
            vec![],
            vec![],
            handle_one_element_null_arrays(
                key.as_str(),
                a_value.clone(),
                json!(b_value).to_owned(),
            ),
        ),
        (Value::Null, Value::Object(b_value)) => handle_one_element_null_objects(
            key,
            a_value.clone(),
            json!(b_value).to_owned(),
            working_context,
        ),

        (Value::Array(a_value), Value::Null) => (
            vec![],
            vec![],
            vec![],
            handle_one_element_null_arrays(
                key.as_str(),
                json!(a_value).to_owned(),
                b_value.clone(),
            ),
        ),
        (Value::Object(a_value), Value::Null) => handle_one_element_null_objects(
            key,
            json!(a_value).to_owned(),
            b_value.clone(),
            working_context,
        ),

        // Type difference a: string
        (Value::String(a_value), Value::Number(b_value)) => (
            vec![],
            handle_different_types(
                key.as_str(),
                json!(a_value).to_owned(),
                json!(b_value).to_owned(),
            ),
            vec![],
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

fn get_array_diff_descriptor_str(
    diff_desc: ArrayDiffDesc,
    working_context: &WorkingContext,
) -> String {
    match diff_desc {
        ArrayDiffDesc::AHas => format!("{} has", working_context.file_a.name),
        ArrayDiffDesc::AMisses => format!("{} has", working_context.file_a.name),
        ArrayDiffDesc::BHas => format!("{} has", working_context.file_b.name),
        ArrayDiffDesc::BMisses => format!("{} has", working_context.file_b.name),
    }
}
