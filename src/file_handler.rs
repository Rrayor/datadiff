use std::{error::Error, fs::File, io::BufReader};

use libdtf::{core::diff_types::WorkingFile, json::read_json_file, yaml::read_yaml_file};

use crate::dtfterminal_types::{
    Config, ConfigBuilder, DiffCollection, DtfError, LibConfig, LibWorkingContext, SavedConfig,
    SavedContext, WorkingContext,
};

pub struct FileHandler {
    user_config: Config,
    saved_config: Option<SavedConfig>,
}

impl FileHandler {
    pub fn new(user_config: Config, saved_config: Option<SavedConfig>) -> FileHandler {
        FileHandler {
            user_config,
            saved_config,
        }
    }

    pub fn read_json_file(
        file_path: &str,
    ) -> Result<serde_json::Map<String, serde_json::Value>, serde_json::Error> {
        read_json_file(file_path)
    }

    pub fn read_yaml_file(
        file_path: &str,
    ) -> Result<serde_yaml::Mapping, serde_yaml::Error> {
        read_yaml_file(file_path)
    }

    pub fn write_to_file(&self, diffs: DiffCollection) -> Result<(), DtfError> {
        let (key_diff_option, type_diff_option, value_diff_option, array_diff_option) = diffs;
        let key_diff = key_diff_option.unwrap_or_default();
        let type_diff = type_diff_option.unwrap_or_default();
        let value_diff = value_diff_option.unwrap_or_default();
        let array_diff = array_diff_option.unwrap_or_default();

        let config = &self.user_config;
        if config.write_to_file.is_none() {
            panic!("File write path is missing!")
        }
        let file = File::create(config.write_to_file.as_ref().unwrap());

        match serde_json::to_writer(
            &mut file.unwrap(),
            &SavedContext::new(
                key_diff,
                type_diff,
                value_diff,
                array_diff,
                SavedConfig::new(
                    config.check_for_key_diffs,
                    config.check_for_type_diffs,
                    config.check_for_value_diffs,
                    config.check_for_array_diffs,
                    config.file_a.clone().unwrap(),
                    config.file_b.clone().unwrap(),
                    config.array_same_order,
                ),
            ),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(DtfError::IoError(e.into())),
        }
    }

    pub fn load_saved_results(
        &mut self,
    ) -> Result<(DiffCollection, WorkingContext), Box<dyn Error>> {
        let saved_data = match self.read_from_file(&self.user_config.read_from_file) {
            Ok(data) => data,
            Err(e) => return Err(Box::new(DtfError::IoError(e.into()))),
        };
        self.saved_config = Some(saved_data.config);

        let diff_collection = (
            Some(saved_data.key_diff),
            Some(saved_data.type_diff),
            Some(saved_data.value_diff),
            Some(saved_data.array_diff),
        );

        let working_context = self.build_working_context_from_loaded_data();

        Ok((diff_collection, working_context))
    }

    fn build_working_context_from_loaded_data(&self) -> WorkingContext {
        if self.saved_config.is_none() {
            panic!("Saved data is corrupted! Config options not present!")
        }

        let saved_config = self.saved_config.as_ref().unwrap();
        let user_config = &self.user_config;

        let file_a = WorkingFile::new(saved_config.file_a.clone());
        let file_b = WorkingFile::new(saved_config.file_b.clone());
        let lib_working_context = LibWorkingContext::new(
            file_a,
            file_b,
            LibConfig::new(saved_config.array_same_order),
        );
        WorkingContext::new(
            lib_working_context,
            ConfigBuilder::new()
                .check_for_key_diffs(saved_config.check_for_key_diffs)
                .check_for_type_diffs(saved_config.check_for_type_diffs)
                .check_for_value_diffs(saved_config.check_for_value_diffs)
                .check_for_array_diffs(saved_config.check_for_array_diffs)
                .render_key_diffs(user_config.render_key_diffs)
                .render_type_diffs(user_config.render_type_diffs)
                .render_value_diffs(user_config.render_value_diffs)
                .render_array_diffs(user_config.render_array_diffs)
                .read_from_file(user_config.read_from_file.clone())
                .write_to_file(user_config.write_to_file.clone())
                .file_a(Some(saved_config.file_a.clone()))
                .file_b(Some(saved_config.file_b.clone()))
                .array_same_order(saved_config.array_same_order)
                .build(),
        )
    }

    fn read_from_file(&self, file_path: &str) -> serde_json::Result<SavedContext> {
        let file =
            File::open(file_path).unwrap_or_else(|_| panic!("Could not open file {}", file_path));
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
    }
}
