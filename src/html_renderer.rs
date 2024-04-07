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

const IDS: Ids = Ids {
    key_diff: "key_diff",
    type_diff: "type_diff",
    value_diff: "value_diff",
    array_diff: "array_diff",
};

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

pub struct HtmlRenderer<'a> {
    context: &'a WorkingContext,
    css: String,
}

// TODO: improve code quality
impl<'a> HtmlRenderer<'a> {
    pub fn new(context: &'a WorkingContext) -> HtmlRenderer<'a> {
        HtmlRenderer {
            context,
            css: HtmlRenderer::create_css(),
        }
    }

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

    fn write_meta(&mut self, head: &mut html_builder::Node) -> Result<(), DtfError> {
        head.meta().attr("charset='utf-8'");
        head.meta()
            .attr("name='viewport'")
            .attr("content='width=device-width, initial-scale=1.0'");
        let css = self.css.clone();
        self.write_line(&mut head.style(), css.as_str())
    }

    fn write_header(&mut self, lead: &mut html_builder::Node) -> Result<(), DtfError> {
        let (file_name1, file_name2) = self.context.get_file_names();
        self.write_line(&mut lead.h1(), &format!("{}", DISPLAY_TEXT.title))?;
        let mut lead_p = lead.p();
        self.write_line(&mut lead_p, DISPLAY_TEXT.lead)?;
        self.write_line(
            &mut lead_p.span().attr(&format!("class='{}'", CLASSES.code)),
            &file_name1,
        )?;
        self.write_line(&mut lead_p, DISPLAY_TEXT.against)?;
        self.write_line(
            &mut lead_p.span().attr(&format!("class='{}'", CLASSES.code)),
            &file_name2,
        )
    }

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

    pub fn html_render_key_diff_table(
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
        self.write_line(&mut tr1.th(), DISPLAY_TEXT.key)?;
        self.write_line(&mut tr1.th(), file_a)?;
        self.write_line(&mut tr1.th(), file_b)?;

        let mut tbody = table.tbody();
        for diff in diffs {
            let key = &diff.key;
            // TODO: extract
            let val1 = diff.has.eq(file_a);
            let val2 = diff.has.eq(file_b);

            let class1 = if val1 {
                CLASSES.checkmark
            } else {
                CLASSES.multiply
            };
            let class2 = if val2 {
                CLASSES.checkmark
            } else {
                CLASSES.multiply
            };

            let mut tr = tbody.tr();
            self.write_line(
                &mut tr.td().attr(&format!("class='{}'", CLASSES.code)),
                &key.to_string(),
            )?;
            // TODO: checkmark and x mark
            self.write_line(
                &mut tr.td().span().attr(&format!("class='{}'", class1)),
                " ",
            )?;
            self.write_line(
                &mut tr.td().span().attr(&format!("class='{}'", class2)),
                " ",
            )?;
        }
        Ok(())
    }

    pub fn html_render_type_diff_table(
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
        self.write_line(&mut tr1.th(), DISPLAY_TEXT.key)?;
        self.write_line(&mut tr1.th(), file_a)?;
        self.write_line(&mut tr1.th(), file_b)?;

        let mut tbody = table.tbody();
        for diff in diffs {
            let key = &diff.key;
            let val1 = &diff.type1;
            let val2 = &diff.type2;

            let mut tr = tbody.tr();
            self.write_line(&mut tr.td().attr(&format!("class='{}'", CLASSES.code)), key)?;
            self.write_line(&mut tr.td(), val1)?;
            self.write_line(&mut tr.td(), val2)?;
        }
        Ok(())
    }

    pub fn html_render_value_diff_table(
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
        self.write_line(&mut tr1.th(), DISPLAY_TEXT.key)?;
        self.write_line(&mut tr1.th(), file_a)?;
        self.write_line(&mut tr1.th(), file_b)?;

        let mut tbody = table.tbody();
        for diff in diffs {
            let key = &diff.key;
            let val1 = &diff.value1;
            let val2 = &diff.value2;

            let mut tr = tbody.tr();
            self.write_line(&mut tr.td().attr(&format!("class='{}'", CLASSES.code)), key)?;
            self.write_line(&mut tr.td(), val1)?;
            self.write_line(&mut tr.td(), val2)?;
        }
        Ok(())
    }

    pub fn html_render_array_diff_table(
        &mut self,
        buf: &mut Buffer,
        diffs: &Vec<ArrayDiff>,
    ) -> Result<(), DtfError> {
        let mut html = buf.html();
        let mut body = html.body();
        let mut table = body
            .table()
            .attr(&format!("class='{}'", CLASSES.diff_table));
        let mut thead = table.thead();
        let mut tr1 = thead.tr();
        self.write_line(&mut tr1.th(), "Key")?;
        self.write_line(&mut tr1.th(), &self.format_header(true))?;
        self.write_line(&mut tr1.th(), &self.format_header(false))?;
        let map = group_by_key(diffs);
        let join_str = if is_yaml_file(self.context.get_file_names().0) {
            ""
        } else {
            ",\n"
        };

        let mut tbody = table.tbody();
        for (key, values) in map {
            let val1 = get_display_values_by_column(&self.context, &values, ArrayDiffDesc::AHas);
            let val2 = get_display_values_by_column(&self.context, &values, ArrayDiffDesc::BHas);

            let mut tr = tbody.tr();
            self.write_line(
                &mut tr.td().attr(&format!("class='{}'", CLASSES.code)),
                &key.to_string(),
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

    fn format_header(&self, is_file_a: bool) -> String {
        let (file_a, file_b) = self.context.get_file_names();
        let file_name = if is_file_a { file_a } else { file_b };

        format!("{} {} {}", DISPLAY_TEXT.only, file_name, DISPLAY_TEXT.has)
    }

    fn write_line(&mut self, node: &mut html_builder::Node, text: &str) -> Result<(), DtfError> {
        writeln!(node, "{}", text).map_err(|e| DtfError::DiffError(format!("{}", e)))
    }

    fn create_css() -> String {
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
            margin: 1em 0;
            padding: 0.5em 0;
            border-top: 1px solid #ffffff;
        }}
        
        .{} li a {{
            color: #ffffff;
            text-decoration: none;
        }}
        
        .{} li a:hover {{
            color: #ffffff;
            text-decoration: underline;
        
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
        
        .{}::after {{
            content: \"\\2713\";
            color:  #00ff00;
        }}

        .{}::after {{
            content: \"\\00D7\";
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
            CLASSES.multiply           // 19
        )
    }
}
