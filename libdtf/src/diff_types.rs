use std::fmt;

use serde_json::Value;

#[derive(Debug, PartialEq)]
pub enum ValueType {
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value_type_str = match self {
            ValueType::Null => "null",
            ValueType::Boolean => "bool",
            ValueType::Number => "number",
            ValueType::String => "string",
            ValueType::Array => "array",
            ValueType::Object => "object",
        };
        write!(f, "{}", value_type_str)
    }
}

#[derive(PartialEq, Debug)]
pub enum ArrayDiffDesc {
    AHas,
    AMisses,
    BHas,
    BMisses,
}

#[derive(PartialEq, Debug)]
pub struct DataValue {
    pub key: String,
    pub value: Value,
}

impl DataValue {
    pub fn new(key: String, value: Value) -> DataValue {
        DataValue { key, value }
    }
}

pub struct Config {
    pub array_same_order: bool,
}

impl Config {
    pub fn new(array_same_order: bool) -> Config {
        Config { array_same_order }
    }
}

pub struct WorkingFile {
    pub name: String,
}

impl WorkingFile {
    pub fn new(name: String) -> WorkingFile {
        WorkingFile { name }
    }
}

pub struct WorkingContext {
    pub file_a: WorkingFile,
    pub file_b: WorkingFile,
    pub config: Config,
}

impl WorkingContext {
    pub fn new(file_a: WorkingFile, file_b: WorkingFile, config: Config) -> WorkingContext {
        WorkingContext {
            file_a,
            file_b,
            config,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct KeyDiff {
    pub key: String,
    pub has: String,
    pub misses: String,
}

impl KeyDiff {
    pub fn new(key: String, has: String, misses: String) -> KeyDiff {
        KeyDiff { key, has, misses }
    }
}
#[derive(PartialEq, Debug)]
pub struct TypeDiff {
    pub key: String,
    pub type1: String,
    pub type2: String,
}

impl TypeDiff {
    pub fn new(key: String, type1: String, type2: String) -> TypeDiff {
        TypeDiff { key, type1, type2 }
    }
}

#[derive(PartialEq, Debug)]
pub struct ValueDiff {
    pub key: String,
    pub value1: String, // TODO: would be better as Option
    pub value2: String,
}

impl ValueDiff {
    pub fn new(key: String, value1: String, value2: String) -> ValueDiff {
        ValueDiff {
            key,
            value1,
            value2,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct ArrayDiff {
    pub key: String,
    pub descriptor: ArrayDiffDesc,
    pub value: String,
}

impl ArrayDiff {
    pub fn new(key: String, descriptor: ArrayDiffDesc, value: String) -> ArrayDiff {
        ArrayDiff {
            key,
            descriptor,
            value,
        }
    }
}

pub type ComparisionResult = (Vec<KeyDiff>, Vec<TypeDiff>, Vec<ValueDiff>, Vec<ArrayDiff>);
