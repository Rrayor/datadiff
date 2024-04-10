use crate::{
    dtfterminal_types::{DiffCollection, WorkingContext},
    file_handler::FileHandler,
};

use libdtf::{
    core::diff_types::{ArrayDiff, Checker, KeyDiff, TypeDiff, ValueDiff},
    yaml::diff_types::CheckingData,
};
use serde_yaml::Mapping;

/// Responsible for the main functionality of the app. Makes sure everything runs in the correct order.
pub struct YamlApp {
    data1: Mapping,
    data2: Mapping,
    context: WorkingContext,
}

impl YamlApp {
    /// Creates a new App instance
    /// 1. Parses the command line arguments
    /// 2. Checks for differences and stores them
    pub fn new(path1: String, path2: String, context: WorkingContext) -> YamlApp {
        let data1 = FileHandler::read_yaml_file(&path1).expect("Could not read YAML file");
        let data2 = FileHandler::read_yaml_file(&path2).expect("Could not read YAML file");
        YamlApp {
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
    fn check_for_diffs(&self, data1: &Mapping, data2: &Mapping) -> DiffCollection {
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
