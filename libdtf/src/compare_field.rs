// use serde_json::{json, Value};

// use crate::diff_types::{ComparisionResult, DataValue, WorkingContext};
// use crate::{
//     compare_arrays, compare_primitives, handle_different_types, handle_one_element_null_arrays,
//     handle_one_element_null_objects, handle_one_element_null_primitives,
// };

// // Compares one data field
// // Moved here for readability only, because it is a very long match statement
// pub fn compare_field<'a>(
//     a: &'a DataValue,
//     b: &'a DataValue,
//     working_context: &WorkingContext,
// ) -> ComparisionResult {
//     let key = &a.key;
//     let a_value = &a.value;
//     let b_value = &b.value;
//     match (a_value, b_value) {
//         // Primitives of same type
//         (Value::Null, Value::Null) => (vec![], vec![], vec![], vec![]),
//         (Value::String(a_value), Value::String(b_value)) => (
//             vec![],
//             vec![],
//             compare_primitives(key, &a_value, &b_value),
//             vec![],
//         ),
//         (Value::Number(a_value), Value::Number(b_value)) => (
//             vec![],
//             vec![],
//             compare_primitives(key, &a_value, &b_value),
//             vec![],
//         ),
//         (Value::Bool(a_value), Value::Bool(b_value)) => (
//             vec![],
//             vec![],
//             compare_primitives(key, &a_value, &b_value),
//             vec![],
//         ),

//         // Composites of same type
//         (Value::Array(a_value), Value::Array(b_value)) => {
//             compare_arrays(key, &a_value, &b_value, working_context)
//         }
//         (Value::Object(a_value), Value::Object(b_value)) => {
//             compare_objects(key, &a_value, &b_value, working_context)
//         }

//         // One value is null primitives
//         (Value::Null, Value::String(b_value)) => (
//             vec![],
//             vec![],
//             handle_one_element_null_primitives(key, &a_value, &json!(b_value)),
//             vec![],
//         ),
//         (Value::Null, Value::Number(b_value)) => (
//             vec![],
//             vec![],
//             handle_one_element_null_primitives(key, &a_value, &json!(b_value)),
//             vec![],
//         ),
//         (Value::Null, Value::Bool(b_value)) => (
//             vec![],
//             vec![],
//             handle_one_element_null_primitives(key, &a_value, &json!(b_value)),
//             vec![],
//         ),

//         (Value::String(a_value), Value::Null) => (
//             vec![],
//             vec![],
//             handle_one_element_null_primitives(key, &json!(a_value), &b_value),
//             vec![],
//         ),
//         (Value::Number(a_value), Value::Null) => (
//             vec![],
//             vec![],
//             handle_one_element_null_primitives(key, &json!(a_value), &b_value),
//             vec![],
//         ),
//         (Value::Bool(a_value), Value::Null) => (
//             vec![],
//             vec![],
//             handle_one_element_null_primitives(key, &json!(a_value), &b_value),
//             vec![],
//         ),

//         // One value is null, composites
//         (Value::Null, Value::Array(b_value)) => (
//             vec![],
//             vec![],
//             vec![],
//             handle_one_element_null_arrays(key, &a_value, &json!(b_value)),
//         ),
//         (Value::Null, Value::Object(b_value)) => handle_one_element_null_objects(
//             key,
//             a_value.clone(),
//             json!(b_value).to_owned(),
//             working_context,
//         ),

//         (Value::Array(a_value), Value::Null) => (
//             vec![],
//             vec![],
//             vec![],
//             handle_one_element_null_arrays(key, &json!(a_value), &b_value),
//         ),
//         (Value::Object(a_value), Value::Null) => handle_one_element_null_objects(
//             key,
//             json!(a_value).to_owned(),
//             b_value.clone(),
//             working_context,
//         ),

//         // Type difference a: string
//         (Value::String(a_value), Value::Number(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::String(a_value), Value::Bool(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::String(a_value), Value::Array(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::String(a_value), Value::Object(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),

//         // Type difference a: number
//         (Value::Number(a_value), Value::String(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::Number(a_value), Value::Bool(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::Number(a_value), Value::Array(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::Number(a_value), Value::Object(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),

//         // Type difference a: bool
//         (Value::Bool(a_value), Value::String(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::Bool(a_value), Value::Number(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::Bool(a_value), Value::Array(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::Bool(a_value), Value::Object(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),

//         // Type difference a: array
//         (Value::Array(a_value), Value::String(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::Array(a_value), Value::Bool(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::Array(a_value), Value::Number(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::Array(a_value), Value::Object(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),

//         // Type difference a: object
//         (Value::Object(a_value), Value::String(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::Object(a_value), Value::Bool(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::Object(a_value), Value::Array(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//         (Value::Object(a_value), Value::Number(b_value)) => (
//             vec![],
//             handle_different_types(key, &json!(a_value), &json!(b_value)),
//             vec![],
//             vec![],
//         ),
//     }
// }
