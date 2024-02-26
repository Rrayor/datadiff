use crate::{dtfterminal_types::{
        DiffCollection, WorkingContext,
    }, file_handler::FileHandler};

use libdtf::{core::diff_types::{
    ArrayDiff, Checker, KeyDiff, TypeDiff, ValueDiff,
}, json::diff_types::CheckingData};
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

    pub fn perform_new_check(&self) -> DiffCollection {
        self.check_for_diffs(
            &self.data1,
            &self.data2,
        )
    }

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
