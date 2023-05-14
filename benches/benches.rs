use criterion::{criterion_group, criterion_main, Criterion};
use dtfterminal::{
    check_for_diffs,
    dtfterminal_types::{Config, ConfigBuilder, LibConfig, LibWorkingContext, WorkingContext},
};
use libdtf::diff_types::WorkingFile;
use serde_json::json;

const FILE_NAME_A: &str = "a.json";
const FILE_NAME_B: &str = "b.json";

fn benchmark_collect_data_no_array_same_order(c: &mut Criterion) {
    // arrange
    let a = json!({
        "no_diff_string": "no_diff_string",
        "diff_string": "a",
        "no_diff_number": "no_diff_number",
        "diff_number": 1,
        "no_diff_boolean": true,
        "diff_boolean": true,
        "no_diff_array": [
            1, 2, 3, 4
        ],
        "diff_array": [
            1, 2, 3, 4
        ],
        "nested": {
            "no_diff_string": "no_diff_string",
            "diff_string": "a",
            "no_diff_number": "no_diff_number",
            "diff_number": 1,
            "no_diff_boolean": true,
            "diff_boolean": true,
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                1, 2, 3, 4
            ],
        },
    });

    let b = json!({
        "no_diff_string": "no_diff_string",
        "diff_string": "b",
        "no_diff_number": "no_diff_number",
        "diff_number": 2,
        "no_diff_boolean": true,
        "diff_boolean": false,
        "no_diff_array": [
            1, 2, 3, 4
        ],
        "diff_array": [
            5, 6, 7, 8
        ],
        "nested": {
            "no_diff_string": "no_diff_string",
            "diff_string": "b",
            "no_diff_number": "no_diff_number",
            "diff_number": 2,
            "no_diff_boolean": true,
            "diff_boolean": false,
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                5, 6, 7, 8
            ],
        },
    });

    let working_context = create_test_working_context(false);

    // act
    c.bench_function("Collect Data No Array Same Order", |bencher| {
        bencher.iter(|| {
            let _ = check_for_diffs(
                a.as_object().unwrap(),
                &b.as_object().unwrap(),
                &working_context,
            );
        })
    });
}

fn benchmark_collect_data_array_same_order(c: &mut Criterion) {
    // arrange
    let a = json!({
        "no_diff_string": "no_diff_string",
        "diff_string": "a",
        "no_diff_number": "no_diff_number",
        "diff_number": 1,
        "no_diff_boolean": true,
        "diff_boolean": true,
        "no_diff_array": [
            1, 2, 3, 4
        ],
        "diff_array": [
            1, 2, 3, 4
        ],
        "nested": {
            "no_diff_string": "no_diff_string",
            "diff_string": "a",
            "no_diff_number": "no_diff_number",
            "diff_number": 1,
            "no_diff_boolean": true,
            "diff_boolean": true,
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                1, 2, 3, 4
            ],
        },
    });

    let b = json!({
        "no_diff_string": "no_diff_string",
        "diff_string": "b",
        "no_diff_number": "no_diff_number",
        "diff_number": 2,
        "no_diff_boolean": true,
        "diff_boolean": false,
        "no_diff_array": [
            1, 2, 3, 4
        ],
        "diff_array": [
            5, 6, 7, 8
        ],
        "nested": {
            "no_diff_string": "no_diff_string",
            "diff_string": "b",
            "no_diff_number": "no_diff_number",
            "diff_number": 2,
            "no_diff_boolean": true,
            "diff_boolean": false,
            "no_diff_array": [
                1, 2, 3, 4
            ],
            "diff_array": [
                5, 6, 7, 8
            ],
        },
    });

    let working_context = create_test_working_context(true);

    // act
    c.bench_function("Collect Data Array Same Order", |bencher| {
        bencher.iter(|| {
            let _ = check_for_diffs(
                a.as_object().unwrap(),
                &b.as_object().unwrap(),
                &working_context,
            );
        })
    });
}

// Benchmark utils

fn create_test_working_context(array_same_order: bool) -> WorkingContext {
    let working_file_a = WorkingFile::new(FILE_NAME_A.to_owned());
    let working_file_b = WorkingFile::new(FILE_NAME_B.to_owned());
    let config = ConfigBuilder::new()
        .check_for_key_diffs(true)
        .check_for_type_diffs(true)
        .check_for_value_diffs(true)
        .check_for_array_diffs(true)
        .render_key_diffs(true)
        .render_type_diffs(true)
        .render_value_diffs(true)
        .render_array_diffs(true)
        .read_from_file(String::new())
        .write_to_file(None)
        .file_a(Some(FILE_NAME_A.to_owned()))
        .file_b(Some(FILE_NAME_B.to_owned()))
        .array_same_order(array_same_order)
        .build();
    WorkingContext::new(
        LibWorkingContext::new(
            working_file_a,
            working_file_b,
            LibConfig::new(array_same_order),
        ),
        config,
    )
}

criterion_group!(
    benches,
    benchmark_collect_data_no_array_same_order,
    benchmark_collect_data_array_same_order
);
criterion_main!(benches);
