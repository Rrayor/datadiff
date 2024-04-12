use crate::{
    dtfterminal_types::{DiffCollection, WorkingContext},
    file_handler::FileHandler,
};

use libdtf::{
    core::diff_types::{ArrayDiff, Checker, KeyDiff, TypeDiff, ValueDiff},
    json::diff_types::CheckingData,
};
use serde_json::{Map, Value};

/// Responsible for the main functionality of the app. Makes sure everything runs in the correct order.
pub struct JsonApp {
    data1: Map<String, Value>,
    data2: Map<String, Value>,
    context: WorkingContext,
}

impl JsonApp {
    /// Creates a new App instance
    /// 1. Parses the command line arguments
    /// 2. Checks for differences and stores them
    pub fn new(path1: String, path2: String, context: WorkingContext) -> JsonApp {
        let data1 = FileHandler::read_json_file(&path1).expect("Could not read JSON file");
        let data2 = FileHandler::read_json_file(&path2).expect("Could not read JSON file");
        JsonApp {
            data1,
            data2,
            context,
        }
    }

    /// Checks for differences between the two files
    pub fn perform_new_check(&self) -> DiffCollection {
        self.check_for_diffs(&self.data1, &self.data2)
    }

    /// Checks for differences between the two files
    fn check_for_diffs(
        &self,
        data1: &Map<String, Value>,
        data2: &Map<String, Value>,
    ) -> DiffCollection {
        let key_diff = if self.context.config.check_for_key_diffs {
            let mut checking_data: CheckingData<KeyDiff> =
                CheckingData::new("", data1, data2, &self.context.lib_working_context);
            checking_data.check();
            Some(checking_data.diffs()).cloned()
        } else {
            None
        };
        let type_diff = if self.context.config.check_for_type_diffs {
            let mut checking_data: CheckingData<TypeDiff> =
                CheckingData::new("", data1, data2, &self.context.lib_working_context);
            checking_data.check();
            Some(checking_data.diffs()).cloned()
        } else {
            None
        };
        let value_diff = if self.context.config.check_for_value_diffs {
            let mut checking_data: CheckingData<ValueDiff> =
                CheckingData::new("", data1, data2, &self.context.lib_working_context);
            checking_data.check();
            Some(checking_data.diffs()).cloned()
        } else {
            None
        };
        let array_diff = if self.context.config.check_for_array_diffs {
            let mut checking_data: CheckingData<ArrayDiff> =
                CheckingData::new("", data1, data2, &self.context.lib_working_context);
            checking_data.check();
            Some(checking_data.diffs()).cloned()
        } else {
            None
        };

        (key_diff, type_diff, value_diff, array_diff)
    }
}

#[cfg(test)]
mod tests {
    use crate::dtfterminal_types::ConfigBuilder;

    use super::*;

    #[test]
    fn test_only_key_diffs_turned_on() {
        let working_context = get_working_context(true, false, false, false);
        let json_app = JsonApp::new(
            "test_data/json/person3.json".to_string(),
            "test_data/json/person4.json".to_string(),
            working_context,
        );
        let diffs = json_app.perform_new_check();
        assert_eq!(diffs.0.is_some(), true);
        assert_eq!(diffs.1.is_none(), true);
        assert_eq!(diffs.2.is_none(), true);
        assert_eq!(diffs.3.is_none(), true);
    }

    #[test]
    fn test_only_type_diffs_turned_on() {
        let working_context = get_working_context(false, true, false, false);
        let json_app = JsonApp::new(
            "test_data/json/person3.json".to_string(),
            "test_data/json/person4.json".to_string(),
            working_context,
        );
        let diffs = json_app.perform_new_check();
        assert_eq!(diffs.0.is_none(), true);
        assert_eq!(diffs.1.is_some(), true);
        assert_eq!(diffs.2.is_none(), true);
        assert_eq!(diffs.3.is_none(), true);
    }

    #[test]
    fn test_only_value_diffs_turned_on() {
        let working_context = get_working_context(false, false, true, false);
        let json_app = JsonApp::new(
            "test_data/json/person3.json".to_string(),
            "test_data/json/person4.json".to_string(),
            working_context,
        );
        let diffs = json_app.perform_new_check();
        assert_eq!(diffs.0.is_none(), true);
        assert_eq!(diffs.1.is_none(), true);
        assert_eq!(diffs.2.is_some(), true);
        assert_eq!(diffs.3.is_none(), true);
    }

    #[test]
    fn test_only_array_diffs_turned_on() {
        let working_context = get_working_context(false, false, false, true);
        let json_app = JsonApp::new(
            "test_data/json/person3.json".to_string(),
            "test_data/json/person4.json".to_string(),
            working_context,
        );
        let diffs = json_app.perform_new_check();
        assert_eq!(diffs.0.is_none(), true);
        assert_eq!(diffs.1.is_none(), true);
        assert_eq!(diffs.2.is_none(), true);
        assert_eq!(diffs.3.is_some(), true);
    }

    #[test]
    fn test_every_diff_turned_on() {
        let working_context = get_working_context(true, true, true, true);
        let json_app = JsonApp::new(
            "test_data/json/person3.json".to_string(),
            "test_data/json/person4.json".to_string(),
            working_context,
        );
        let diffs = json_app.perform_new_check();
        assert_eq!(diffs.0.is_some(), true);
        assert_eq!(diffs.1.is_some(), true);
        assert_eq!(diffs.2.is_some(), true);
        assert_eq!(diffs.3.is_some(), true);
    }

    // Note: We shouldn't get to this point as the arguments do not allow this setup, but it's good to test that the code works as expected
    #[test]
    fn test_no_diffs_are_turned_on() {
        let working_context = get_working_context(false, false, false, false);
        let json_app = JsonApp::new(
            "test_data/json/person3.json".to_string(),
            "test_data/json/person4.json".to_string(),
            working_context,
        );
        let diffs = json_app.perform_new_check();
        assert_eq!(diffs.0.is_none(), true);
        assert_eq!(diffs.1.is_none(), true);
        assert_eq!(diffs.2.is_none(), true);
        assert_eq!(diffs.3.is_none(), true);
    }

    fn get_working_context(
        key_diffs: bool,
        type_diffs: bool,
        value_diffs: bool,
        array_diffs: bool,
    ) -> WorkingContext {
        let working_file_a = libdtf::core::diff_types::WorkingFile::new("FileA.yaml".to_string());
        let working_file_b = libdtf::core::diff_types::WorkingFile::new("FileB.yaml".to_string());
        let lib_working_context = libdtf::core::diff_types::WorkingContext::new(
            working_file_a,
            working_file_b,
            libdtf::core::diff_types::Config {
                array_same_order: false,
            },
        );
        let working_context = WorkingContext::new(
            lib_working_context,
            ConfigBuilder::new()
                .check_for_key_diffs(key_diffs)
                .check_for_type_diffs(type_diffs)
                .check_for_value_diffs(value_diffs)
                .check_for_array_diffs(array_diffs)
                .build(),
        );
        working_context
    }
}
