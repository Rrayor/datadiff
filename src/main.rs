#[derive(Debug)]
struct DataObject<'a> {
    name: &'a str,
    value: Option<&'a str>,
    children: Vec<DataObject<'a>>
}

fn main() {
    let test_do = DataObject { name: "donnow", value: Some("something"), children: vec![] };
    println!("{}", format!("Test Dataobject: {test_do:?}"));
    println!("Printing just to see, nothing is broken...");
}
