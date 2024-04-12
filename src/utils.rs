use std::collections::HashMap;

use libdtf::core::diff_types::{ArrayDiff, ArrayDiffDesc, WorkingFile};
use serde_yaml::Value;

use crate::dtfterminal_types::{Config, LibConfig, LibWorkingContext, WorkingContext};

/// Unicode representation of a checkmark to render in the terminal
pub const CHECKMARK: &str = "\u{2713}";

/// Unicode representation of a cross to render in the terminal
pub const MULTIPLY: &str = "\u{00D7}";

/// Group array diffs by key
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

/// Get values to display in each column.
/// Columns represent the files compared.
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

/// Creates a working context object based on user configuration
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

#[cfg(test)]
mod tests {
    use crate::dtfterminal_types::ConfigBuilder;

    use super::*;

    #[test]
    fn test_get_display_values_by_column() {
        let context = WorkingContext::new(
            LibWorkingContext::new(
                WorkingFile::new("file_a.txt".to_owned()),
                WorkingFile::new("file_b.txt".to_owned()),
                LibConfig::new(true),
            ),
            ConfigBuilder::new().build(),
        );

        let diff1 = ArrayDiff {
            descriptor: ArrayDiffDesc::AHas,
            key: "key1".to_owned(),
            value: "value1".to_owned(),
        };
        let diff2 = ArrayDiff {
            descriptor: ArrayDiffDesc::AHas,
            key: "key2".to_owned(),
            value: "value2".to_owned(),
        };
        let diff3 = ArrayDiff {
            descriptor: ArrayDiffDesc::AHas,
            key: "key3".to_owned(),
            value: "value3".to_owned(),
        };
        let values = vec![&diff1, &diff2, &diff3];

        let diff_desc = ArrayDiffDesc::AHas;

        let display_values = get_display_values_by_column(&context, &values, diff_desc);

        assert_eq!(display_values, vec!["value1", "value2", "value3"]);
    }

    #[test]
    fn test_create_working_context() {
        let config = ConfigBuilder::new()
            .file_a(Some("file_a.txt".to_owned()))
            .file_b(Some("file_b.txt".to_owned()))
            .array_same_order(true)
            .build();

        let working_context = create_working_context(&config);

        let (file_a_in_context, file_b_in_context) = working_context.get_file_names();
        assert_eq!(file_a_in_context, "file_a.txt");
        assert_eq!(file_b_in_context, "file_b.txt");
        assert_eq!(
            working_context.lib_working_context.config.array_same_order,
            true
        );
    }

    #[test]
    fn test_is_yaml_file() {
        let yaml_file = "file.yaml";
        let yml_file = "file.yml";
        let txt_file = "file.txt";
        let json_file = "file.json";

        assert_eq!(is_yaml_file(yaml_file), true);
        assert_eq!(is_yaml_file(yml_file), true);
        assert_eq!(is_yaml_file(txt_file), false);
        assert_eq!(is_yaml_file(json_file), false);
    }

    #[test]
    fn test_group_by_key() {
        let data = vec![
            ArrayDiff {
                descriptor: ArrayDiffDesc::AHas,
                key: "key1".to_owned(),
                value: "value1".to_owned(),
            },
            ArrayDiff {
                descriptor: ArrayDiffDesc::AHas,
                key: "key2".to_owned(),
                value: "value2".to_owned(),
            },
            ArrayDiff {
                descriptor: ArrayDiffDesc::AHas,
                key: "key2".to_owned(),
                value: "value3".to_owned(),
            },
            ArrayDiff {
                descriptor: ArrayDiffDesc::AHas,
                key: "key3".to_owned(),
                value: "value4".to_owned(),
            },
        ];

        let grouped_data = group_by_key(&data);

        assert_eq!(grouped_data.len(), 3);
        assert_eq!(grouped_data.get("key1"), Some(&vec![&data[0]]));
        assert_eq!(grouped_data.get("key2"), Some(&vec![&data[1], &data[2]]));
        assert_eq!(grouped_data.get("key3"), Some(&vec![&data[3]]));
    }
}
