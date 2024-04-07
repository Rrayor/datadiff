use std::collections::HashMap;

use libdtf::core::diff_types::{ArrayDiff, ArrayDiffDesc, WorkingFile};
use serde_yaml::Value;

use crate::dtfterminal_types::{Config, LibConfig, LibWorkingContext, WorkingContext};

pub const CHECKMARK: &str = "\u{2713}";
pub const MULTIPLY: &str = "\u{00D7}";

pub fn group_by_key(data: &[ArrayDiff]) -> HashMap<&str, Vec<&ArrayDiff>> {
    let mut map = HashMap::new();

    for ad in data {
        let key = ad.key.as_str();

        if !map.contains_key(key) {
            map.insert(key, vec![]);
        }

        map.get_mut(key).unwrap().push(ad);
    }

    map
}

pub fn get_display_values_by_column(
    context: &WorkingContext,
    values: &[&ArrayDiff],
    diff_desc: ArrayDiffDesc,
) -> Vec<String> {
    let file_names = context.get_file_names();
    values
        .iter()
        .filter(|ad| ad.descriptor == diff_desc)
        .map(|ad| prettify_data(file_names, ad.value.as_str()))
        .collect()
}

pub fn create_working_context(config: &Config) -> WorkingContext {
    let file_a = WorkingFile::new(config.file_a.as_ref().unwrap().clone());
    let file_b = WorkingFile::new(config.file_b.as_ref().unwrap().clone());

    let lib_working_context =
        LibWorkingContext::new(file_a, file_b, LibConfig::new(config.array_same_order));

    WorkingContext::new(lib_working_context, config.clone())
}

/// Formats data based on file type
pub fn prettify_data(file_names: (&str, &str), data: &str) -> String {
    // at this point we can be sure, both file names have the same file type, so we can just check the first one
    let (file1, _) = file_names;
    if is_yaml_file(file1) {
        return prettify_yaml_str(data);
    }

    prettify_json_str(data)
}

/// Formats JSON strings
pub fn prettify_json_str(json_str: &str) -> String {
    match serde_json::from_str::<Value>(json_str) {
        Ok(json_value) => serde_json::to_string_pretty(&json_value).unwrap_or(json_str.to_owned()),
        Err(_) => json_str.to_owned(),
    }
}

/// Formats YAML strings
pub fn prettify_yaml_str(yaml_str: &str) -> String {
    match serde_yaml::from_str::<Value>(yaml_str) {
        Ok(yaml_value) => serde_yaml::to_string(&yaml_value).unwrap_or(yaml_str.to_owned()),
        Err(_) => yaml_str.to_owned(),
    }
}

/// Checks if a file is a YAML file
pub fn is_yaml_file(path: &str) -> bool {
    path.ends_with(".yaml") || path.ends_with(".yml")
}
