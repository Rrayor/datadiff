use std::{fs::File, error::Error, io::Read};

#[derive(Debug)]
struct DataObject<'a> {
    name: &'a str,
    value: Option<&'a str>,
    children: Vec<DataObject<'a>>
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let test_do = DataObject {
        name: "donnow",
        value: Some("something"),
        children: vec![]
    };
    println!("{}", format!("Test Dataobject: {test_do:?}"));
    let test_file_content = read_file("test_file.json")?;
    println!("Test file contents:\n{}", &test_file_content);
    Ok(())
}

fn read_file(file_name: &str) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(file_name)
        .map_err(
            |e| format!("Unable to open file {}: {}", file_name, e)
        )?;
    let mut file_contents = String::new();
    file
        .read_to_string(&mut file_contents)
        .map_err(
            |e| format!("Unable to read contents of file {}: {}", file_name, e)
        )?;
    Ok(file_contents)

}
