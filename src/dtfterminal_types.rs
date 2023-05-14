use std::{error::Error, fmt};

use libdtf::diff_types::{ArrayDiff, KeyDiff, TypeDiff, ValueDiff};
use serde::{Deserialize, Serialize};

pub type LibConfig = libdtf::diff_types::Config;
pub type LibWorkingContext = libdtf::diff_types::WorkingContext;

pub type DiffCollection = (
    Option<Vec<KeyDiff>>,
    Option<Vec<TypeDiff>>,
    Option<Vec<ValueDiff>>,
    Option<Vec<ArrayDiff>>,
);

#[derive(Serialize, Deserialize)]
pub struct SavedConfig {
    pub check_for_key_diffs: bool,
    pub check_for_type_diffs: bool,
    pub check_for_value_diffs: bool,
    pub check_for_array_diffs: bool,
    pub file_a: String,
    pub file_b: String,
    pub array_same_order: bool,
}

impl SavedConfig {
    pub fn new(
        check_for_key_diffs: bool,
        check_for_type_diffs: bool,
        check_for_value_diffs: bool,
        check_for_array_diffs: bool,
        file_a: String,
        file_b: String,
        array_same_order: bool,
    ) -> SavedConfig {
        SavedConfig {
            check_for_key_diffs,
            check_for_type_diffs,
            check_for_value_diffs,
            check_for_array_diffs,
            file_a,
            file_b,
            array_same_order,
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub check_for_key_diffs: bool,
    pub check_for_type_diffs: bool,
    pub check_for_value_diffs: bool,
    pub check_for_array_diffs: bool,
    pub render_key_diffs: bool,
    pub render_type_diffs: bool,
    pub render_value_diffs: bool,
    pub render_array_diffs: bool,
    pub read_from_file: String,
    pub write_to_file: Option<String>,
    pub file_a: Option<String>,
    pub file_b: Option<String>,
    pub array_same_order: bool,
}

impl Config {
    pub fn new(
        check_for_key_diffs: bool,
        check_for_type_diffs: bool,
        check_for_value_diffs: bool,
        check_for_array_diffs: bool,
        render_key_diffs: bool,
        render_type_diffs: bool,
        render_value_diffs: bool,
        render_array_diffs: bool,
        read_from_file: String,
        write_to_file: Option<String>,
        file_a: Option<String>,
        file_b: Option<String>,
        array_same_order: bool,
    ) -> Config {
        Config {
            check_for_key_diffs,
            check_for_type_diffs,
            check_for_value_diffs,
            check_for_array_diffs,
            render_key_diffs,
            render_type_diffs,
            render_value_diffs,
            render_array_diffs,
            read_from_file,
            write_to_file,
            file_a,
            file_b,
            array_same_order,
        }
    }
}

#[derive(Clone)]
pub struct WorkingContext {
    pub lib_working_context: LibWorkingContext,
    pub config: Config,
}

impl WorkingContext {
    pub fn new(lib_working_context: LibWorkingContext, config: Config) -> WorkingContext {
        WorkingContext {
            lib_working_context,
            config,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SavedContext {
    pub key_diff: Vec<KeyDiff>,
    pub type_diff: Vec<TypeDiff>,
    pub value_diff: Vec<ValueDiff>,
    pub array_diff: Vec<ArrayDiff>,
    pub config: SavedConfig,
}

impl SavedContext {
    pub fn new(
        key_diff: Vec<KeyDiff>,
        type_diff: Vec<TypeDiff>,
        value_diff: Vec<ValueDiff>,
        array_diff: Vec<ArrayDiff>,
        config: SavedConfig,
    ) -> SavedContext {
        SavedContext {
            key_diff,
            type_diff,
            value_diff,
            array_diff,
            config,
        }
    }
}

#[derive(Debug)]
pub struct IOError;

impl Error for IOError {}

impl fmt::Display for IOError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IO action failed!")
    }
}
