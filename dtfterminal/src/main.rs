use libdtf::{compare_objects, read_json_file, WorkingContext, WorkingFile};

fn main() -> Result<(), ()> {
    let file_name1 = "test_data/person3.json";
    let file_name2 = "test_data/person4.json";
    let data1 = read_json_file(file_name1).expect(&format!("Couldn't read file: {}", file_name1));
    let data2 = read_json_file(file_name2).expect(&format!("Couldn't read file: {}", file_name2));
    let working_context = WorkingContext {
        file_a: WorkingFile {
            name: file_name1.to_string(),
        },
        file_b: WorkingFile {
            name: file_name2.to_string(),
        },
    };
    let (key_diff, type_diff, value_diff, array_diff) =
        compare_objects("".to_string(), &data1, &data2, &working_context);

    for ele in key_diff {
        print!("{}", ele.get_formatted_string());
    }

    for ele in type_diff {
        print!("{}", ele.get_formatted_string());
    }

    for ele in value_diff {
        print!("{}", ele.get_formatted_string());
    }

    for ele in array_diff {
        print!("{}", ele.get_formatted_string(&working_context));
    }

    Ok(())
}
