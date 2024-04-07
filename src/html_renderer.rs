use std::fmt::Write;

use html_builder::{Buffer, Html5};
use libdtf::core::diff_types::{ArrayDiff, ArrayDiffDesc};

use crate::{
    dtfterminal_types::{DtfError, WorkingContext},
    utils::{get_display_values_by_column, group_by_key, is_yaml_file},
};

const CSS: &str = "* {
    font-family: Arial, Helvetica, sans-serif;
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
    font-family: \"Lucida Console\", \"Courier New\", monospace;
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
    margin: 1em 0;
    padding: 0.5em 0;
    border-top: 1px solid #ffffff;
}

.table-of-contents li a {
    color: #ffffff;
    text-decoration: none;
}

.table-of-contents li a:hover {
    color: #ffffff;
    text-decoration: underline;

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
}";

pub struct HtmlRenderer<'a> {
    context: &'a WorkingContext,
}

// TODO: improve code quality
impl<'a> HtmlRenderer<'a> {
    pub fn new(context: &'a WorkingContext) -> HtmlRenderer<'a> {
        HtmlRenderer { context }
    }

    pub fn init_document(
        &mut self,
        buf: &mut Buffer,
        render_options: (bool, bool, bool, bool),
    ) -> Result<(), DtfError> {
        let (render_key_diffs, render_type_diffs, render_value_diffs, render_array_diffs) =
            render_options;
        buf.doctype();
        let mut html = buf.html().attr("lang='en'");
        let mut head = html.head();
        writeln!(
            head.title(),
            "Datadiff Comparing {} and {}",
            self.context
                .config
                .file_a
                .clone()
                .unwrap_or_else(|| { "Unknown".to_owned() }),
            self.context
                .config
                .file_b
                .clone()
                .unwrap_or_else(|| { "Unknown".to_owned() })
        )
        .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        head.meta().attr("charset='utf-8'");
        head.meta()
            .attr("name='viewport'")
            .attr("content='width=device-width, initial-scale=1.0'");
        writeln!(head.style(), "{}", CSS).map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        let mut body = html.body();
        let mut header = body.div().attr("class='header'");
        let mut lead = header.div().attr("class='lead'");
        writeln!(lead.h1(), "Data Differences")
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        let (file_name1, file_name2) = self.context.get_file_names();
        let mut lead_p = lead.p();
        writeln!(lead_p, "The following differences were found comparing")
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        writeln!(lead_p.span().attr("class='code'"), "{}", file_name1)
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        writeln!(lead_p, "against").map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        writeln!(lead_p.span().attr("class='code'"), "{}", file_name2)
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        let mut ul = header.ul().attr("class='table-of-contents'");
        writeln!(ul.h2(), "Table of Contents")
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        if render_key_diffs {
            writeln!(ul.li().a().attr("href='#key_diff'"), "Key Differences")
                .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        }
        if render_type_diffs {
            writeln!(ul.li().a().attr("href='#type_diff'"), "Type Differences")
                .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        }
        if render_value_diffs {
            writeln!(ul.li().a().attr("href='#value_diff'"), "Value Differences")
                .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        }
        if render_array_diffs {
            writeln!(ul.li().a().attr("href='#array_diff'"), "Array Differences")
                .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
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
        writeln!(body.h2().attr("id='key_diff'"), "Key Differences")
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        let mut table = body.table().attr("class='diff-table'");
        let mut tr1 = table.tr();
        writeln!(tr1.th(), "Key").map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        writeln!(
            tr1.th(),
            "{}",
            self.context
                .config
                .file_a
                .clone()
                .unwrap_or_else(|| "Unknown".to_owned())
        )
        .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        writeln!(
            tr1.th(),
            "{}",
            self.context
                .config
                .file_b
                .clone()
                .unwrap_or_else(|| "Unknown".to_owned())
        )
        .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        Ok(for diff in diffs {
            let key = &diff.key;
            let val1 = diff
                .has
                .eq(self.context.lib_working_context.file_a.name.as_str());
            let val2 = diff
                .has
                .eq(self.context.lib_working_context.file_b.name.as_str());

            let mut tr = table.tr();
            writeln!(tr.td().attr("class='code'"), "{}", key)
                .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
            // TODO: checkmark and x mark
            writeln!(tr.td(), "{}", val1).map_err(|e| DtfError::DiffError(format!("{}", e)))?;
            writeln!(tr.td(), "{}", val2).map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        })
    }

    pub fn html_render_type_diff_table(
        &mut self,
        buf: &mut Buffer,
        diffs: &Vec<libdtf::core::diff_types::TypeDiff>,
    ) -> Result<(), DtfError> {
        let mut html = buf.html();
        let mut body = html.body();
        writeln!(body.h2().attr("id='type_diff'"), "Type Differences")
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        let mut table = body.table().attr("class='diff-table'");
        let mut tr1 = table.tr();
        writeln!(tr1.th(), "Key").map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        writeln!(
            tr1.th(),
            "{}",
            self.context
                .config
                .file_a
                .clone()
                .unwrap_or_else(|| "Unknown".to_owned())
        )
        .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        writeln!(
            tr1.th(),
            "{}",
            self.context
                .config
                .file_b
                .clone()
                .unwrap_or_else(|| "Unknown".to_owned())
        )
        .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        Ok(for diff in diffs {
            let key = &diff.key;
            let val1 = &diff.type1;
            let val2 = &diff.type2;

            let mut tr = table.tr();
            writeln!(tr.td().attr("class='code'"), "{}", key)
                .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
            writeln!(tr.td(), "{}", val1).map_err(|e| DtfError::DiffError(format!("{}", e)))?;
            writeln!(tr.td(), "{}", val2).map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        })
    }

    pub fn html_render_value_diff_table(
        &mut self,
        buf: &mut Buffer,
        diffs: &Vec<libdtf::core::diff_types::ValueDiff>,
    ) -> Result<(), DtfError> {
        let mut html = buf.html();
        let mut body = html.body();
        writeln!(body.h2().attr("id='value_diff'"), "Value Differences")
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        let mut table = body.table().attr("class='diff-table'");
        let mut tr1 = table.tr();
        writeln!(tr1.th(), "Key").map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        writeln!(
            tr1.th(),
            "{}",
            self.context
                .config
                .file_a
                .clone()
                .unwrap_or_else(|| "Unknown".to_owned())
        )
        .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        writeln!(
            tr1.th(),
            "{}",
            self.context
                .config
                .file_b
                .clone()
                .unwrap_or_else(|| "Unknown".to_owned())
        )
        .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        Ok(for diff in diffs {
            let key = &diff.key;
            let val1 = &diff.value1;
            let val2 = &diff.value2;

            let mut tr = table.tr();
            writeln!(tr.td().attr("class='code'"), "{}", key)
                .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
            writeln!(tr.td(), "{}", val1).map_err(|e| DtfError::DiffError(format!("{}", e)))?;
            writeln!(tr.td(), "{}", val2).map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        })
    }

    pub fn html_render_array_diff_table(
        &mut self,
        buf: &mut Buffer,
        diffs: &Vec<ArrayDiff>,
    ) -> Result<(), DtfError> {
        let mut html = buf.html();
        let mut body = html.body();
        writeln!(body.h2().attr("id='array_diff'"), "Array Differences")
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        let mut table = body.table().attr("class='diff-table'");
        let mut tr1 = table.tr();
        writeln!(tr1.th(), "Key").map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        writeln!(
            tr1.th(),
            "Only \"{}\" has",
            self.context
                .config
                .file_a
                .clone()
                .unwrap_or_else(|| "Unknown".to_owned())
        )
        .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        writeln!(
            tr1.th(),
            "Only \"{}\" has",
            self.context
                .config
                .file_b
                .clone()
                .unwrap_or_else(|| "Unknown".to_owned())
        )
        .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        let map = group_by_key(diffs);
        let join_str = if is_yaml_file(self.context.get_file_names().0) {
            ""
        } else {
            ",\n"
        };
        Ok(for (key, values) in map {
            let val1 = get_display_values_by_column(&self.context, &values, ArrayDiffDesc::AHas);
            let val2 = get_display_values_by_column(&self.context, &values, ArrayDiffDesc::BHas);

            let mut tr = table.tr();
            writeln!(tr.td().attr("class='code'"), "{}", key)
                .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
            writeln!(
                tr.td().pre().attr("class='original'"),
                "{}",
                val1.join(join_str)
            )
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
            writeln!(
                tr.td().pre().attr("class='original'"),
                "{}",
                val2.join(join_str)
            )
            .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        })
    }
}
