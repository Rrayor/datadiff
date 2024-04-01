use std::collections::HashMap;
use std::{error::Error, fs::File};
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;

use crate::prettify_data;
use crate::{
    array_table::ArrayTable, dtfterminal_types::{
        Config, ConfigBuilder, DiffCollection, DtfError, LibConfig, LibWorkingContext, ParsedArgs,
        TermTable, WorkingContext,
    }, file_handler::FileHandler, is_yaml_file, json_app::JsonApp, key_table::KeyTable, type_table::TypeTable, value_table::ValueTable, yaml_app::YamlApp, Arguments
};

use ::clap::Parser;
use html_builder::{Buffer, Html5};
use libdtf::core::diff_types::{ArrayDiff, ArrayDiffDesc, WorkingFile};
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
        } else if self.context.config.write_to_html {
            self.render_html()
                .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
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
            .write_to_html(args.write_to_html)
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

    fn render_html(&self) -> Result<(), DtfError> {
        let mut buf = Buffer::new();
        buf.doctype();
        let mut html = buf.html().attr("lang='en'");
        let mut head = html.head();
        writeln!(head.title(),
             "Datadiff Comparing {} and {}",
              self.context.config.file_a.clone().unwrap_or_else(|| { "Unknown".to_owned() }), 
              self.context.config.file_b.clone().unwrap_or_else(|| { "Unknown".to_owned() }))
              .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        head.meta().attr("charset='utf-8'");
        writeln!(head.style(), "* {{
            font-family: Arial, Helvetica, sans-serif;
        }}
        
        h1, h2 {{
            width: 100%;
            margin: auto;
            text-align: center;
        }}
        
        h2 {{
            margin-top: 2em;
        }}
        
        .code {{
            font-family: \"Lucida Console\", \"Courier New\", monospace;
        }}
        
        .diff-table {{
            width: 100%;
        }}
        
        .diff-table th {{
            background-color: #f0f0f0;
        }}
        
        .diff-table th, .diff-table td{{
            padding: 8px;
            text-align: left;
        }}
        
        .diff-table td {{
            padding: 8px;
        }}
        
        .diff-table tr:nth-child(odd) {{
            background-color: #f6f6f6;
        }}")
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        let mut body = html.body();
        writeln!(body.h1(), "Data Differences")
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        writeln!(body.p(), "The following differences were found:")
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;

        if self.context.config.render_key_diffs {
            if let Some(diffs) = self.diffs.0.as_ref().filter(|kd| !kd.is_empty()) {
                writeln!(body.h2(), "Key Differences")
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                let mut table = body.table().attr("class='diff-table'").attr("cellpadding='8'");
                let mut tr1 = table.tr();
                writeln!(tr1.th(), "Key")
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                writeln!(tr1.th(), "{}", self.context.config.file_a.clone().unwrap_or_else(|| "Unknown".to_owned()))
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                writeln!(tr1.th(), "{}", self.context.config.file_b.clone().unwrap_or_else(|| "Unknown".to_owned()))
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;

                for diff in diffs {
                    let key = &diff.key;
                    let val1 = diff.has.eq(self.context.lib_working_context.file_a.name.as_str());
                    let val2 = diff.has.eq(self.context.lib_working_context.file_b.name.as_str());

                    let mut tr = table.tr();
                    writeln!(tr.td().attr("class='code'"), "{}", key)
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                    writeln!(tr.td(), "{}", val1)
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                    writeln!(tr.td(), "{}", val2)
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                }
            }
        }

        if self.context.config.render_type_diffs {
            if let Some(diffs) = self.diffs.1.as_ref().filter(|td| !td.is_empty()) {
                writeln!(body.h2(), "Type Differences")
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                let mut table = body.table().attr("class='diff-table'").attr("cellpadding='8'");
                let mut tr1 = table.tr();
                writeln!(tr1.th(), "Key")
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                writeln!(tr1.th(), "{}", self.context.config.file_a.clone().unwrap_or_else(|| "Unknown".to_owned()))
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                writeln!(tr1.th(), "{}", self.context.config.file_b.clone().unwrap_or_else(|| "Unknown".to_owned()))
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;

                for diff in diffs {
                    let key = &diff.key;
                    let val1 = &diff.type1;
                    let val2 = &diff.type2;

                    let mut tr = table.tr();
                    writeln!(tr.td().attr("class='code'"), "{}", key)
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                    writeln!(tr.td(), "{}", val1)
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                    writeln!(tr.td(), "{}", val2)
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                }
            }
        }

        if self.context.config.render_value_diffs {
            if let Some(diffs) = self.diffs.2.as_ref().filter(|vd| !vd.is_empty()) {
                writeln!(body.h2(), "Value Differences")
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                let mut table = body.table().attr("class='diff-table'").attr("cellpadding='8'");
                let mut tr1 = table.tr();
                writeln!(tr1.th(), "Key")
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                writeln!(tr1.th(), "{}", self.context.config.file_a.clone().unwrap_or_else(|| "Unknown".to_owned()))
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                writeln!(tr1.th(), "{}", self.context.config.file_b.clone().unwrap_or_else(|| "Unknown".to_owned()))
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;

                for diff in diffs {
                    let key = &diff.key;
                    let val1 = &diff.value1;
                    let val2 = &diff.value2;

                    let mut tr = table.tr();
                    writeln!(tr.td().attr("class='code'"), "{}", key)
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                    writeln!(tr.td(), "{}", val1)
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                    writeln!(tr.td(), "{}", val2)
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                }
            }
        }

        if self.context.config.render_array_diffs {
            if let Some(diffs) = self.diffs.3.as_ref().filter(|ad| !ad.is_empty()) {
                writeln!(body.h2(), "Array Differences")
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                let mut table = body.table().attr("class='diff-table'").attr("cellpadding='8'");
                let mut tr1 = table.tr();
                writeln!(tr1.th(), "Key")
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                writeln!(tr1.th(), "Only \"{}\" has", self.context.config.file_a.clone().unwrap_or_else(|| "Unknown".to_owned()))
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                writeln!(tr1.th(), "Only \"{}\" has", self.context.config.file_b.clone().unwrap_or_else(|| "Unknown".to_owned()))
                    .map_err(|e| DtfError::DiffError(format!("{}", e)))?;


                    let map = App::group_by_key(diffs);
                    let join_str = if is_yaml_file(self.get_file_names().0) {
                        ""
                    } else {
                        ",\n"
                    };

                    for (key, values) in map {
                        let val1 = self.get_display_values_by_column(&values, ArrayDiffDesc::AHas);
                        let val2 = self.get_display_values_by_column(&values, ArrayDiffDesc::BHas);

                        let mut tr = table.tr();
                        writeln!(tr.td().attr("class='code'"), "{}", key)
                            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                        writeln!(tr.td().pre().attr("class='original'"), "{}", val1.join(join_str))
                            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                        writeln!(tr.td().pre().attr("class='original'"), "{}", val2.join(join_str))
                            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
                    }
                }
            }

        // TODO: Proper file name
        // TODO: Refactor
        // TODO: Proper error handling
        // TODO: Styling
        let mut file = File::create("diff.html")
            .map_err(|e| DtfError::DiffError(format!("Could not create file: {}", e)))?;

        write!(file, "{}", buf.finish()).map_err(|e| DtfError::DiffError(format!("{}", e)))?;

        Ok(())
    }
    
    fn get_display_values_by_column(
        &self,
        values: &[&ArrayDiff],
        diff_desc: ArrayDiffDesc,
    ) -> Vec<String> {
        let file_names = self.get_file_names();
        values
            .iter()
            .filter(|ad| ad.descriptor == diff_desc)
            .map(|ad| prettify_data(file_names, ad.value.as_str()))
            .collect()
    }

    fn group_by_key(data: &[ArrayDiff]) -> HashMap<&str, Vec<&ArrayDiff>> {
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

    fn get_file_names(&self) -> (&str, &str) {
        let file_name_a = self.context.lib_working_context.file_a.name.as_str();
        let file_name_b = self.context.lib_working_context.file_b.name.as_str();
        (file_name_a, file_name_b)
    }

    fn create_working_context(config: &Config) -> WorkingContext {
        let file_a = WorkingFile::new(config.file_a.as_ref().unwrap().clone());
        let file_b = WorkingFile::new(config.file_b.as_ref().unwrap().clone());

        let lib_working_context =
            LibWorkingContext::new(file_a, file_b, LibConfig::new(config.array_same_order));

        WorkingContext::new(lib_working_context, config.clone())
    }
}
