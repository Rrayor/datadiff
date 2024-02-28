use std::error::Error;

use crate::{
    array_table::ArrayTable, dtfterminal_types::{
        Config, ConfigBuilder, DiffCollection, DtfError, LibConfig, LibWorkingContext, ParsedArgs,
        TermTable, WorkingContext,
    }, file_handler::FileHandler, is_yaml_file, json_app::JsonApp, key_table::KeyTable, type_table::TypeTable, value_table::ValueTable, yaml_app::YamlApp, Arguments
};

use ::clap::Parser;
use libdtf::core::diff_types::WorkingFile;
use spinners::Spinner;

/// Responsible for the main functionality of the app. Makes sure everything runs in the correct order.
pub struct App {
    diffs: DiffCollection,
    context: WorkingContext,
    file_handler: FileHandler,
    json_app: Option<JsonApp>,
    yaml_app: Option<YamlApp>,
}

impl App {
    /// Creates a new App instance
    /// 1. Parses the command line arguments
    /// 2. Checks for differences and stores them
    pub fn new() -> App {
        let (path1, path2, config) = App::parse_args();
        let mut file_handler = FileHandler::new(config.clone(), None);
        let (diffs, context) = if config.read_from_file.is_empty() {
            (
                (None, None, None, None),
                App::create_working_context(&config),
            )
        } else {
            file_handler
                .load_saved_results()
                .expect("Could not load saved file!")
        };

        let json_app = match (&path1, &path2) {
            (Some(p1), Some(p2)) if p1.ends_with(".json") && p2.ends_with(".json") => {
                Some(JsonApp::new(p1.clone(), p2.clone(), context.clone()))
            }
            _ => None,
        };

        let yaml_app = match (&path1, &path2) {
            (Some(p1), Some(p2)) if is_yaml_file(p1) && is_yaml_file(p2) => {
                Some(YamlApp::new(p1.clone(), p2.clone(), context.clone()))
            }
            _ => None,
        };

        if json_app.is_none() && yaml_app.is_none() {
            panic!("No valid files to check!");
        }

        let mut app = App {
            diffs,
            context,
            file_handler,
            json_app,
            yaml_app,
        };

        app.collect_data(&config);

        app
    }

    /// Handles the output into file or to the terminal
    pub fn execute(&self) -> Result<(), DtfError> {
        let mut spinner = Spinner::new(spinners::Spinners::Monkey, "Checking for differences...\n".into());
        if self.context.config.write_to_file.is_some() {
            self.file_handler
                .write_to_file(self.diffs.clone())
                .map_err(|e| DtfError::GeneralError(Box::new(e)))?;
        } else {
            self.render_tables()
                .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        }

        spinner.stop();
        Ok(())
    }

    fn parse_args() -> ParsedArgs {
        let args = Arguments::parse();

        let (path1, path2) = if args.read_from_file.is_empty() {
            (
                Some(args.check_files[0].clone()),
                Some(args.check_files[1].clone()),
            )
        } else {
            (None, None)
        };

        let config = ConfigBuilder::new()
            .check_for_key_diffs(args.key_diffs)
            .check_for_type_diffs(args.type_diffs)
            .check_for_value_diffs(args.value_diffs)
            .check_for_array_diffs(args.array_diffs)
            .render_key_diffs(args.key_diffs)
            .render_type_diffs(args.type_diffs)
            .render_value_diffs(args.value_diffs)
            .render_array_diffs(args.array_diffs)
            .read_from_file(args.read_from_file)
            .write_to_file(args.write_to_file)
            .file_a(path1.clone())
            .file_b(path2.clone())
            .array_same_order(args.array_same_order)
            .build();

        (path1, path2, config)
    }

    fn collect_data(&mut self, user_config: &Config) {
        if user_config.read_from_file.is_empty() {
            self.diffs = self.perform_new_check().expect("Data check failed!")
        } else {
            self.diffs = self
                .file_handler
                .load_saved_results()
                .expect("Could not load saved file!")
                .0;
        }
    }

    fn perform_new_check(&self) -> Result<DiffCollection, Box<dyn Error>> {
        self.check_for_diffs()
    }

    fn check_for_diffs(&self) -> Result<DiffCollection, Box<dyn Error>> {
        if let Some(json_app) = &self.json_app {
            Ok(json_app.perform_new_check())
        } else if let Some(yaml_app) = &self.yaml_app {
            Ok(yaml_app.perform_new_check())
        } else {
            Err(Box::new(DtfError::DiffError(
                "No file to check".to_string(),
            )))
        }
    }

    fn render_tables(&self) -> Result<(), DtfError> {
        let (key_diff, type_diff, value_diff, array_diff) = &self.diffs;

        let mut rendered_tables = vec![];
        if self.context.config.render_key_diffs {
            if let Some(diffs) = key_diff.as_ref().filter(|kd| !kd.is_empty()) {
                let table = KeyTable::new(diffs, &self.context.lib_working_context);
                rendered_tables.push(table.render());
            }
        }

        if self.context.config.render_type_diffs {
            if let Some(diffs) = type_diff.as_ref().filter(|td| !td.is_empty()) {
                let table = TypeTable::new(diffs, &self.context.lib_working_context);
                rendered_tables.push(table.render());
            }
        }

        if self.context.config.render_value_diffs {
            if let Some(diffs) = value_diff.as_ref().filter(|vd| !vd.is_empty()) {
                let table = ValueTable::new(diffs, &self.context.lib_working_context);
                rendered_tables.push(table.render());
            }
        }

        if self.context.config.render_array_diffs {
            if let Some(diffs) = array_diff.as_ref().filter(|ad| !ad.is_empty()) {
                let table = ArrayTable::new(diffs, &self.context.lib_working_context);
                rendered_tables.push(table.render());
            }
        }

        if rendered_tables.is_empty() {
            println!("The data is identical!");
            return Ok(());
        }

        for table in rendered_tables {
            println!("{}", table);
        }

        Ok(())
    }

    fn create_working_context(config: &Config) -> WorkingContext {
        let file_a = WorkingFile::new(config.file_a.as_ref().unwrap().clone());
        let file_b = WorkingFile::new(config.file_b.as_ref().unwrap().clone());

        let lib_working_context =
            LibWorkingContext::new(file_a, file_b, LibConfig::new(config.array_same_order));

        WorkingContext::new(lib_working_context, config.clone())
    }
}
