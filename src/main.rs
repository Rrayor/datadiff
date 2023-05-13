use dtfterminal::{collect_data, parse_args, render_tables};

fn main() -> Result<(), ()> {
    let (data1, data2, working_context) = parse_args();
    let (key_diff, type_diff, value_diff, array_diff) =
        collect_data(&data1, &data2, &working_context);
    render_tables(
        key_diff,
        type_diff,
        value_diff,
        array_diff,
        &working_context,
    );
    Ok(())
}
