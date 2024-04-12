use std::fmt::Write;

use html_builder::{Buffer, Html5};
use libdtf::core::diff_types::{ArrayDiff, ArrayDiffDesc};

use crate::{
    dtfterminal_types::{DtfError, WorkingContext},
    utils::{get_display_values_by_column, group_by_key, is_yaml_file},
};

struct Classes {
    code: &'static str,
    header: &'static str,
    lead: &'static str,
    table_of_contents: &'static str,
    diff_table: &'static str,
    original: &'static str,
    checkmark: &'static str,
    multiply: &'static str,
}

struct Ids {
    key_diff: &'static str,
    type_diff: &'static str,
    value_diff: &'static str,
    array_diff: &'static str,
}

struct DisplayText {
    comparing: &'static str,
    title: &'static str,
    table_of_contents: &'static str,
    lead: &'static str,
    against: &'static str,
    key: &'static str,
    key_diff_title: &'static str,
    type_diff_title: &'static str,
    value_diff_title: &'static str,
    array_diff_title: &'static str,
    only: &'static str,
    has: &'static str,
}

/// Collection of CSS classes used in the HTML output.
const CLASSES: Classes = Classes {
    code: "code",
    header: "header",
    lead: "lead",
    table_of_contents: "table-of-contents",
    diff_table: "diff-table",
    original: "original",
    checkmark: "checkmark",
    multiply: "multiply",
};

/// Collection of HTML IDs used in the HTML output.
const IDS: Ids = Ids {
    key_diff: "key_diff",
    type_diff: "type_diff",
    value_diff: "value_diff",
    array_diff: "array_diff",
};

/// Collection of text displayed in the HTML output.
const DISPLAY_TEXT: DisplayText = DisplayText {
    comparing: "Comparing",
    title: "Data Differences",
    table_of_contents: "Table of Contents",
    lead: "The following differences were found comparing",
    against: "against",
    key: "Key",
    key_diff_title: "Key Differences",
    type_diff_title: "Type Differences",
    value_diff_title: "Value Differences",
    array_diff_title: "Array Differences",
    only: "Only",
    has: "has",
};

/// The `HtmlRenderer` struct is responsible for rendering the HTML output.
pub struct HtmlRenderer<'a> {
    context: &'a WorkingContext,
    css: String,
}

impl<'a> HtmlRenderer<'a> {
    pub fn new(context: &'a WorkingContext) -> HtmlRenderer<'a> {
        HtmlRenderer {
            context,
            css: HtmlRenderer::create_css(context.config.printer_friendly),
        }
    }

    /// Initializes the HTML document.
    /// This function writes the doctype, html, head, and body tags to the buffer.
    /// # Arguments
    /// * `buf``: The buffer to write the HTML document to.
    /// * `render_options`: A tuple of booleans that determine which sections of the HTML document to render.
    ///  The tuple is in the following order: key_diffs, type_diffs, value_diffs, array_diffs.
    pub fn init_document(
        &mut self,
        buf: &mut Buffer,
        render_options: (bool, bool, bool, bool),
    ) -> Result<(), DtfError> {
        buf.doctype();
        let mut html = buf.html().attr("lang='en'");
        let mut head = html.head();
        self.write_title(&mut head)?;
        self.write_meta(&mut head)?;
        let mut body = html.body();
        let mut header = body.div().attr(&format!("class='{}'", CLASSES.header));
        let mut lead = header.div().attr(&format!("class='{}'", CLASSES.lead));
        self.write_header(&mut lead)?;
        self.write_table_of_contents(&mut header, render_options)?;
        Ok(())
    }

    /// Writes the title of the HTML document.
    fn write_title(&mut self, head: &mut html_builder::Node) -> Result<(), DtfError> {
        let (file_a, file_b) = self.context.get_file_names();
        self.write_line(
            &mut head.title(),
            &format!(
                "{} {} {} {}",
                DISPLAY_TEXT.comparing, file_a, DISPLAY_TEXT.against, file_b
            ),
        )
    }

    /// Writes the meta tags of the HTML document.
    fn write_meta(&mut self, head: &mut html_builder::Node) -> Result<(), DtfError> {
        head.meta().attr("charset='utf-8'");
        head.meta()
            .attr("name='viewport'")
            .attr("content='width=device-width, initial-scale=1.0'");
        let css = self.css.clone();
        self.write_line(&mut head.style(), css.as_str())
    }

    /// Writes the header of the HTML document including a title a small lead paragraph.
    fn write_header(&mut self, lead: &mut html_builder::Node) -> Result<(), DtfError> {
        let (file_name1, file_name2) = self.context.get_file_names();
        self.write_line(&mut lead.h1(), DISPLAY_TEXT.title)?;
        let mut lead_p = lead.p();
        self.write_line(&mut lead_p, DISPLAY_TEXT.lead)?;
        self.write_line(
            &mut lead_p.span().attr(&format!("class='{}'", CLASSES.code)),
            file_name1,
        )?;
        self.write_line(&mut lead_p, DISPLAY_TEXT.against)?;
        self.write_line(
            &mut lead_p.span().attr(&format!("class='{}'", CLASSES.code)),
            file_name2,
        )
    }

    /// Writes the table of contents of the HTML document.
    /// /// # Arguments
    /// * `buf``: The buffer to write the HTML document to.
    /// * `render_options`: A tuple of booleans that determine which sections of the HTML document to render.
    ///  The tuple is in the following order: key_diffs, type_diffs, value_diffs, array_diffs.
    fn write_table_of_contents(
        &mut self,
        header: &mut html_builder::Node,
        render_options: (bool, bool, bool, bool),
    ) -> Result<(), DtfError> {
        let (render_key_diffs, render_type_diffs, render_value_diffs, render_array_diffs) =
            render_options;
        let mut ul = header
            .ul()
            .attr(&format!("class='{}'", CLASSES.table_of_contents));
        self.write_line(&mut ul.h2(), DISPLAY_TEXT.table_of_contents)?;
        if render_key_diffs {
            self.write_line(
                &mut ul.li().a().attr(&format!("href='#{}'", IDS.key_diff)),
                DISPLAY_TEXT.key_diff_title,
            )?;
        }
        if render_type_diffs {
            self.write_line(
                &mut ul.li().a().attr(&format!("href='#{}'", IDS.type_diff)),
                DISPLAY_TEXT.type_diff_title,
            )?;
        }
        if render_value_diffs {
            self.write_line(
                &mut ul.li().a().attr(&format!("href='#{}'", IDS.value_diff)),
                DISPLAY_TEXT.value_diff_title,
            )?;
        }
        if render_array_diffs {
            self.write_line(
                &mut ul.li().a().attr(&format!("href='#{}'", IDS.array_diff)),
                DISPLAY_TEXT.array_diff_title,
            )?;
        }
        Ok(())
    }

    /// Renders the key differences table.
    pub fn render_key_diff_table(
        &mut self,
        buf: &mut Buffer,
        diffs: &Vec<libdtf::core::diff_types::KeyDiff>,
    ) -> Result<(), DtfError> {
        let mut html = buf.html();
        let mut body = html.body();
        let (file_a, file_b) = self.context.get_file_names();
        self.write_line(
            &mut body.h2().attr(&format!("id='{}'", IDS.key_diff)),
            DISPLAY_TEXT.key_diff_title,
        )?;
        let mut table = body
            .table()
            .attr(&format!("class='{}'", CLASSES.diff_table));
        let mut thead = table.thead();
        let mut tr1 = thead.tr();
        self.write_line(&mut tr1.th().attr("scope='col'"), DISPLAY_TEXT.key)?;
        self.write_line(&mut tr1.th().attr("scope='col'"), file_a)?;
        self.write_line(&mut tr1.th().attr("scope='col'"), file_b)?;

        let mut tbody = table.tbody();
        for diff in diffs {
            let key = &diff.key;
            let get_class = |file| {
                if diff.has.eq(file) {
                    CLASSES.checkmark
                } else {
                    CLASSES.multiply
                }
            };

            let class1 = get_class(file_a);
            let class2 = get_class(file_b);

            let mut tr = tbody.tr();
            self.write_line(
                &mut tr
                    .th()
                    .attr(&format!("class='{}'", CLASSES.code))
                    .attr("scope='row'"),
                &key.to_string(),
            )?;

            tr.td().span().attr(&format!("class='{}'", class1));
            tr.td().span().attr(&format!("class='{}'", class2));
        }
        Ok(())
    }

    /// Renders the type differences table.
    pub fn render_type_diff_table(
        &mut self,
        buf: &mut Buffer,
        diffs: &Vec<libdtf::core::diff_types::TypeDiff>,
    ) -> Result<(), DtfError> {
        let mut html = buf.html();
        let mut body = html.body();
        let (file_a, file_b) = self.context.get_file_names();
        self.write_line(
            &mut body.h2().attr(&format!("id='{}'", IDS.type_diff)),
            DISPLAY_TEXT.type_diff_title,
        )?;
        let mut table = body
            .table()
            .attr(&format!("class='{}'", CLASSES.diff_table));
        let mut thead = table.thead();
        let mut tr1 = thead.tr();
        self.write_line(&mut tr1.th().attr("scope='col'"), DISPLAY_TEXT.key)?;
        self.write_line(&mut tr1.th().attr("scope='col'"), file_a)?;
        self.write_line(&mut tr1.th().attr("scope='col'"), file_b)?;

        let mut tbody = table.tbody();
        for diff in diffs {
            let key = &diff.key;
            let val1 = &diff.type1;
            let val2 = &diff.type2;

            let mut tr = tbody.tr();
            self.write_line(
                &mut tr
                    .th()
                    .attr(&format!("class='{}'", CLASSES.code))
                    .attr("scope='row'"),
                key,
            )?;
            self.write_line(&mut tr.td(), val1)?;
            self.write_line(&mut tr.td(), val2)?;
        }
        Ok(())
    }

    /// Renders the value differences table.
    pub fn render_value_diff_table(
        &mut self,
        buf: &mut Buffer,
        diffs: &Vec<libdtf::core::diff_types::ValueDiff>,
    ) -> Result<(), DtfError> {
        let mut html = buf.html();
        let mut body = html.body();
        let (file_a, file_b) = self.context.get_file_names();
        self.write_line(
            &mut body.h2().attr(&format!("id='{}'", IDS.value_diff)),
            DISPLAY_TEXT.value_diff_title,
        )?;
        let mut table = body
            .table()
            .attr(&format!("class='{}'", CLASSES.diff_table));
        let mut thead = table.thead();
        let mut tr1 = thead.tr();
        self.write_line(&mut tr1.th().attr("scope='col'"), DISPLAY_TEXT.key)?;
        self.write_line(&mut tr1.th().attr("scope='col'"), file_a)?;
        self.write_line(&mut tr1.th().attr("scope='col'"), file_b)?;

        let mut tbody = table.tbody();
        for diff in diffs {
            let key = &diff.key;
            let val1 = &diff.value1;
            let val2 = &diff.value2;

            let mut tr = tbody.tr();
            self.write_line(
                &mut tr
                    .th()
                    .attr(&format!("class='{}'", CLASSES.code))
                    .attr("scope='row'"),
                key,
            )?;
            self.write_line(&mut tr.td(), val1)?;
            self.write_line(&mut tr.td(), val2)?;
        }
        Ok(())
    }

    /// Renders the array differences table.
    pub fn render_array_diff_table(
        &mut self,
        buf: &mut Buffer,
        diffs: &[ArrayDiff],
    ) -> Result<(), DtfError> {
        let mut html = buf.html();
        let mut body = html.body();
        self.write_line(
            &mut body.h2().attr(&format!("id='{}'", IDS.array_diff)),
            DISPLAY_TEXT.array_diff_title,
        )?;
        let mut table = body
            .table()
            .attr(&format!("class='{}'", CLASSES.diff_table));
        let mut thead = table.thead();
        let mut tr1 = thead.tr();
        self.write_line(&mut tr1.th().attr("scope='col'"), "Key")?;
        self.write_line(
            &mut tr1.th().attr("scope='col'"),
            &self.format_array_diff_table_header(true),
        )?;
        self.write_line(
            &mut tr1.th().attr("scope='col'"),
            &self.format_array_diff_table_header(false),
        )?;
        let map = group_by_key(diffs);
        let join_str = if is_yaml_file(self.context.get_file_names().0) {
            ""
        } else {
            ",\n"
        };

        let mut tbody = table.tbody();
        for (key, values) in map {
            let val1 = get_display_values_by_column(self.context, &values, ArrayDiffDesc::AHas);
            let val2 = get_display_values_by_column(self.context, &values, ArrayDiffDesc::BHas);

            let mut tr = tbody.tr();
            self.write_line(
                &mut tr
                    .th()
                    .attr(&format!("class='{}'", CLASSES.code))
                    .attr("scope='row'"),
                key,
            )?;
            self.write_line(
                &mut tr.td().pre().attr(&format!("class='{}'", CLASSES.original)),
                &val1.join(join_str),
            )?;
            self.write_line(
                &mut tr.td().pre().attr(&format!("class='{}'", CLASSES.original)),
                &val2.join(join_str),
            )?;
        }
        Ok(())
    }

    /// Creates a column header for the array differences table.
    /// # Arguments
    /// * `is_file_a`: A boolean that determines if the column header is for file A. If false, the column header is for file B.
    fn format_array_diff_table_header(&self, is_file_a: bool) -> String {
        let (file_a, file_b) = self.context.get_file_names();
        let file_name = if is_file_a { file_a } else { file_b };

        format!("{} {} {}", DISPLAY_TEXT.only, file_name, DISPLAY_TEXT.has)
    }

    /// Writes a line of text to the buffer.
    /// If an error occurs, it's mapped to a `DtfError`.
    fn write_line(&mut self, node: &mut html_builder::Node, text: &str) -> Result<(), DtfError> {
        writeln!(node, "{}", text).map_err(|e| DtfError::DiffError(format!("{}", e)))
    }

    /// Creates the CSS for the HTML output.
    /// # Arguments
    /// * `printer_friendly`: A boolean that determines if the CSS is for a printer-friendly output.
    /// Printer friendly output is basically a light theme with black text. And uses more widely compatible CSS formatting.
    fn create_css(printer_friendly: bool) -> String {
        if printer_friendly {
            // 0: code
            // 1: header
            // 2: lead
            // 3: code
            // 4. table-of-contents
            // 5. table-of-contents
            // 6. table-of-contents
            // 7. table-of-contents
            // 8. table-of-contents
            // 9. diff-table
            // 10. diff-table
            // 11. diff-table
            // 12. diff-table
            // 13. diff-table
            // 14. diff-table
            // 15. checkmark
            // 16. multiply
            format!(
                "* {{
            font-family: Arial, Helvetica, sans-serif;
            box-sizing: border-box;
        }}
        
        body {{
            padding: 1em;
            font-size: 14px;
            background-color: #fff;
            color: #000;
        }}
        
        h1, h2 {{
            width: 100%;
            text-align: left;
        }}
        
        h2 {{
            margin-top: 2em;
        }}
        
        .{} {{
            font-family: \"Lucida Console\", \"Courier New\", monospace;
        }}
        
        .{} .{} p .{} {{
            font-weight: bold;
            background-color: rgba(100, 100, 100, 0.4);
            padding: 0.2em;
            border-radius: 2px;
        }}
        
        ul.{} {{
            max-width: 30%;
            margin-top: 2em;
            margin-bottom: 2em;
            padding: 1em;
            list-style-type: none;
            border: 1px solid #000;
        }}
        
        .{} h2 {{
            margin-top: 0;
        }}
        
        .{} li {{
            width: 100%;
            padding: 0.5em 0;
            font-size: 1.2em;
        }}
        
        .{} li a {{
            color: #000;
            text-decoration: underline;
        }}
        
        .{} li a:hover {{
            color: #0000ff;
        }}
        
        .{} {{
            margin: auto;
            margin-top: 2em;
            text-align: center;
            width: 100%;
            color: #000;
        }}
        
        .{} th, .{} td{{
            padding: 1.2em;
            text-align: left;
        }}
        
        .{} th {{
            background-color: rgba(100, 100, 100, 0.3);
        }}
        
        .{} tr:nth-child(odd) {{
            background-color: rgba(100, 100, 100, 0.1);
        }}
        
        .{} tr:nth-child(even) {{
            background-color: rgba(100, 100, 100, 0.2);
        }}
        
        .{}::before {{
            visibility: visible;
            content: \"\\2713\";
            font-weight: bold;
            font-size: 1.5em;
            color: #5aa25a;
        }}

        .{}::before {{
            visibility: visible;
            content: \"\\00D7\";
            font-weight: bold;
            font-size: 1.5em;
            color: #ff0000;
        }}",
                CLASSES.code,              // 0
                CLASSES.header,            // 1
                CLASSES.lead,              // 2
                CLASSES.code,              // 3
                CLASSES.table_of_contents, // 4
                CLASSES.table_of_contents, // 5
                CLASSES.table_of_contents, // 6
                CLASSES.table_of_contents, // 7
                CLASSES.table_of_contents, // 8
                CLASSES.diff_table,        // 9
                CLASSES.diff_table,        // 10
                CLASSES.diff_table,        // 11
                CLASSES.diff_table,        // 12
                CLASSES.diff_table,        // 13
                CLASSES.diff_table,        // 14
                CLASSES.checkmark,         // 15
                CLASSES.multiply,          // 16
            )
        } else {
            // 0: code
            // 1: header
            // 2: header
            // 3: lead
            // 4: header
            // 5: lead
            // 6: code
            // 7. table-of-contents
            // 8. table-of-contents
            // 9. table-of-contents
            // 10. table-of-contents
            // 11. table-of-contents
            // 12. diff-table
            // 13. diff-table
            // 14. diff-table
            // 15. diff-table
            // 16. diff-table
            // 17. diff-table
            // 18. checkmark
            // 19. multiply
            format!(
                "* {{
            font-family: Arial, Helvetica, sans-serif;
            box-sizing: border-box;
        }}
        
        body {{
            padding: 1em;
            font-size: 14px;
            background-color: #0a0b0b;
            color: #fff;
        }}
        
        h1, h2 {{
            width: fit-content;
            width: -moz-fit-content;
            text-align: left;
            background: linear-gradient(to right, #8e2de2, #4a00e0);
            background-clip: text;
            -webkit-text-fill-color: transparent;
        }}
        
        h2 {{
            margin-top: 2em;
        }}
        
        .{} {{
            font-family: \"Lucida Console\", \"Courier New\", monospace;
        }}
        
        .{} {{
            display: flex;
            flex-direction: row;
            justify-content: space-between;
        }}
        
        .{} .{} {{
            display: flex;
            flex-direction: column;
        }}
        
        .{} .{} p .{} {{
            font-weight: bold;
            background-color: rgba(100, 100, 100, 0.4);
            padding: 0.2em;
            border-radius: 2px;
        }}
        
        ul.{} {{
            width: fit-content;
            width: -moz-fit-content;
            margin-top: 2em;
            margin-bottom: 2em;
            padding: 1em;
            background-color: rgba(100, 100, 100, 0.2);
            border-radius: 10px;
            list-style-type: none;
        }}
        
        .{} h2 {{
            margin-top: 0;
        }}
        
        .{} li {{
            width: 100%;
            padding: 0.5em 0;
            font-size: 1.2em;
        }}
        
        .{} li a {{
            color: #ffffff;
            text-decoration: underline;
        }}
        
        .{} li a:hover {{
            color: #787878;
        }}
        
        .{} {{
            margin: auto;
            margin-top: 2em;
            text-align: center;
            width: 100%;
            color: #ffffff;
            border-radius: 10px;
        }}
        
        .{} th, .{} td{{
            padding: 1.2em;
            text-align: left;
        }}
        
        .{} th {{
            background-color: rgba(100, 100, 100, 0.3);
        }}
        
        .{} tr:nth-child(odd) {{
            background-color: rgba(100, 100, 100, 0.1);
        }}
        
        .{} tr:nth-child(even) {{
            background-color: rgba(100, 100, 100, 0.2);
        }}
        
        .{}::before {{
            visibility: visible;
            content: \"\\2713\";
            font-weight: bold;
            font-size: 1.5em;
            color:  #00ff00;
        }}

        .{}::before {{
            visibility: visible;
            content: \"\\00D7\";
            font-weight: bold;
            font-size: 1.5em;
            color: #ff0000;
        }}",
                CLASSES.code,              // 0
                CLASSES.header,            // 1
                CLASSES.header,            // 2
                CLASSES.lead,              // 3
                CLASSES.header,            // 4
                CLASSES.lead,              // 5
                CLASSES.code,              // 6
                CLASSES.table_of_contents, // 7
                CLASSES.table_of_contents, // 8
                CLASSES.table_of_contents, // 9
                CLASSES.table_of_contents, // 10
                CLASSES.table_of_contents, // 11
                CLASSES.diff_table,        // 12
                CLASSES.diff_table,        // 13
                CLASSES.diff_table,        // 14
                CLASSES.diff_table,        // 15
                CLASSES.diff_table,        // 16
                CLASSES.diff_table,        // 17
                CLASSES.checkmark,         // 18
                CLASSES.multiply,          // 19
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dtfterminal_types::ConfigBuilder;

    use super::*;

    #[test]
    fn test_format_array_diff_table_header() {
        let working_context = get_working_context();
        let renderer = HtmlRenderer::new(&working_context);
        assert_eq!(
            renderer.format_array_diff_table_header(true),
            "Only FileA.yaml has"
        );
        assert_eq!(
            renderer.format_array_diff_table_header(false),
            "Only FileB.yaml has"
        );
    }

    #[test]
    fn test_write_line() {
        let working_context = get_working_context();
        let mut renderer = HtmlRenderer::new(&working_context);
        let expected = "<html>\n <body>\nHello, World!\n </body>\n</html>\n";
        let mut buf = html_builder::Buffer::new();
        let mut html = buf.html();
        let mut node = html.body();
        renderer.write_line(&mut node, "Hello, World!").unwrap();
        assert_eq!(buf.finish(), expected);
    }

    #[test]
    fn test_create_css() {
        assert_eq!(
            HtmlRenderer::create_css(true),
            r#"* {
            font-family: Arial, Helvetica, sans-serif;
            box-sizing: border-box;
        }
        
        body {
            padding: 1em;
            font-size: 14px;
            background-color: #fff;
            color: #000;
        }
        
        h1, h2 {
            width: 100%;
            text-align: left;
        }
        
        h2 {
            margin-top: 2em;
        }
        
        .code {
            font-family: "Lucida Console", "Courier New", monospace;
        }
        
        .header .lead p .code {
            font-weight: bold;
            background-color: rgba(100, 100, 100, 0.4);
            padding: 0.2em;
            border-radius: 2px;
        }
        
        ul.table-of-contents {
            max-width: 30%;
            margin-top: 2em;
            margin-bottom: 2em;
            padding: 1em;
            list-style-type: none;
            border: 1px solid #000;
        }
        
        .table-of-contents h2 {
            margin-top: 0;
        }
        
        .table-of-contents li {
            width: 100%;
            padding: 0.5em 0;
            font-size: 1.2em;
        }
        
        .table-of-contents li a {
            color: #000;
            text-decoration: underline;
        }
        
        .table-of-contents li a:hover {
            color: #0000ff;
        }
        
        .diff-table {
            margin: auto;
            margin-top: 2em;
            text-align: center;
            width: 100%;
            color: #000;
        }
        
        .diff-table th, .diff-table td{
            padding: 1.2em;
            text-align: left;
        }
        
        .diff-table th {
            background-color: rgba(100, 100, 100, 0.3);
        }
        
        .diff-table tr:nth-child(odd) {
            background-color: rgba(100, 100, 100, 0.1);
        }
        
        .diff-table tr:nth-child(even) {
            background-color: rgba(100, 100, 100, 0.2);
        }
        
        .checkmark::before {
            visibility: visible;
            content: "\2713";
            font-weight: bold;
            font-size: 1.5em;
            color: #5aa25a;
        }

        .multiply::before {
            visibility: visible;
            content: "\00D7";
            font-weight: bold;
            font-size: 1.5em;
            color: #ff0000;
        }"#
        );
        assert_eq!(
            HtmlRenderer::create_css(false),
            r#"* {
            font-family: Arial, Helvetica, sans-serif;
            box-sizing: border-box;
        }
        
        body {
            padding: 1em;
            font-size: 14px;
            background-color: #0a0b0b;
            color: #fff;
        }
        
        h1, h2 {
            width: fit-content;
            width: -moz-fit-content;
            text-align: left;
            background: linear-gradient(to right, #8e2de2, #4a00e0);
            background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        
        h2 {
            margin-top: 2em;
        }
        
        .code {
            font-family: "Lucida Console", "Courier New", monospace;
        }
        
        .header {
            display: flex;
            flex-direction: row;
            justify-content: space-between;
        }
        
        .header .lead {
            display: flex;
            flex-direction: column;
        }
        
        .header .lead p .code {
            font-weight: bold;
            background-color: rgba(100, 100, 100, 0.4);
            padding: 0.2em;
            border-radius: 2px;
        }
        
        ul.table-of-contents {
            width: fit-content;
            width: -moz-fit-content;
            margin-top: 2em;
            margin-bottom: 2em;
            padding: 1em;
            background-color: rgba(100, 100, 100, 0.2);
            border-radius: 10px;
            list-style-type: none;
        }
        
        .table-of-contents h2 {
            margin-top: 0;
        }
        
        .table-of-contents li {
            width: 100%;
            padding: 0.5em 0;
            font-size: 1.2em;
        }
        
        .table-of-contents li a {
            color: #ffffff;
            text-decoration: underline;
        }
        
        .table-of-contents li a:hover {
            color: #787878;
        }
        
        .diff-table {
            margin: auto;
            margin-top: 2em;
            text-align: center;
            width: 100%;
            color: #ffffff;
            border-radius: 10px;
        }
        
        .diff-table th, .diff-table td{
            padding: 1.2em;
            text-align: left;
        }
        
        .diff-table th {
            background-color: rgba(100, 100, 100, 0.3);
        }
        
        .diff-table tr:nth-child(odd) {
            background-color: rgba(100, 100, 100, 0.1);
        }
        
        .diff-table tr:nth-child(even) {
            background-color: rgba(100, 100, 100, 0.2);
        }
        
        .checkmark::before {
            visibility: visible;
            content: "\2713";
            font-weight: bold;
            font-size: 1.5em;
            color:  #00ff00;
        }

        .multiply::before {
            visibility: visible;
            content: "\00D7";
            font-weight: bold;
            font-size: 1.5em;
            color: #ff0000;
        }"#
        );
    }

    fn get_working_context() -> WorkingContext {
        let working_file_a = libdtf::core::diff_types::WorkingFile::new("FileA.yaml".to_string());
        let working_file_b = libdtf::core::diff_types::WorkingFile::new("FileB.yaml".to_string());
        let lib_working_context = libdtf::core::diff_types::WorkingContext::new(
            working_file_a,
            working_file_b,
            libdtf::core::diff_types::Config {
                array_same_order: false,
            },
        );
        let working_context =
            WorkingContext::new(lib_working_context, ConfigBuilder::new().build());
        working_context
    }
}
