#![allow(dead_code, unused_variables)]
use std::io::prelude::*;
use std::process::Command;
use std::{fs::File, path::PathBuf};
use syntax::latex::{self, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

use petgraph::{
    dot::{Config as DotConfig, Dot},
    graph::NodeIndex,
    Graph,
};

fn split_after_indices(s: String, indices: &Vec<isize>) -> Vec<String> {
    let len = s.len() as isize;
    let mut parts = Vec::new();
    let mut prev_end = 0;

    for i in indices {
        let idx = if *i < 0 {
            (len + i) as usize
        } else {
            *i as usize
        };
        let split_point = (idx + 1).min(s.len());

        if split_point > prev_end {
            parts.push(s[prev_end..split_point].to_string());
            prev_end = split_point;
        }
    }

    if prev_end < s.len() {
        parts.push(s[prev_end..].to_string());
    }

    parts
}

fn indent_str(indent_level: usize, tabstop: u8) -> String {
    " ".repeat(indent_level * tabstop as usize)
}

pub struct FormatContext {
    tabstop: u8,
    indent_level: usize,
    math_mode: bool,
    line_length: u8,
}

pub struct Formatter {
    context: FormatContext,
}

impl Formatter {
    pub fn new() -> Self {
        Self {
            context: FormatContext {
                tabstop: 2,
                indent_level: 0,
                math_mode: false,
                line_length: 80,
            },
        }
    }

    pub fn visit(&mut self, element: &SyntaxElement) -> String {
        match element {
            SyntaxElement::Node(node) => self.visit_node(node),
            SyntaxElement::Token(token) => self.visit_token(token),
        }
    }

    fn visit_token(&self, token: &SyntaxToken) -> String {
        match token.kind() {
            SyntaxKind::WHITESPACE => {
                if token.to_string() == ' '.to_string() {
                    token.to_string()
                } else {
                    String::new()
                }
            }
            SyntaxKind::COMMENT | SyntaxKind::BLOCK_COMMENT => {
                let indent = indent_str(self.context.indent_level, self.context.tabstop);
                format!("\n{indent}{}", token.text().trim_end())
            }
            SyntaxKind::EQUALITY_SIGN => " = ".to_string(),
            _ => token.text().to_string(),
        }
    }

    fn visit_node(&mut self, node: &SyntaxNode) -> String {
        match node.kind() {
            SyntaxKind::PREAMBLE => self.visit_preamble(node),

            SyntaxKind::BRACK_GROUP_KEY_VALUE => self.visit_brack_group_key_value(node),
            SyntaxKind::CURLY_GROUP_KEY_VALUE => self.visit_curly_group_key_value(node),
            SyntaxKind::CURLY_GROUP_WORD | SyntaxKind::CURLY_GROUP_WORD_LIST => {
                self.visit_curly_group_word_list(node)
            }

            SyntaxKind::CURLY_GROUP => self.visit_curly_group(node),

            SyntaxKind::CURLY_GROUP_COMMAND
            | SyntaxKind::BRACK_GROUP
            | SyntaxKind::BRACK_GROUP_WORD
            | SyntaxKind::PAREN_GROUP
            | SyntaxKind::MIXED_GROUP => self.visit_group(node),

            //SyntaxKind::THEOREM_DEFINITION_AMSTHM => todo!(),
            //SyntaxKind::THEOREM_DEFINITION_THMTOOLS => todo!(),

            //SyntaxKind::COLOR_REFERENCE => todo!(),
            //SyntaxKind::COLOR_DEFINITION => todo!(),
            //SyntaxKind::COLOR_SET_DEFINITION => todo!(),
            SyntaxKind::LATEX_INCLUDE
            | SyntaxKind::BIBLATEX_INCLUDE
            | SyntaxKind::BIBTEX_INCLUDE
            | SyntaxKind::GRAPHICS_INCLUDE
            | SyntaxKind::SVG_INCLUDE
            | SyntaxKind::INKSCAPE_INCLUDE
            | SyntaxKind::VERBATIM_INCLUDE
            | SyntaxKind::IMPORT
            | SyntaxKind::GRAPHICS_PATH
            | SyntaxKind::TIKZ_LIBRARY_IMPORT => self.visit_include(node),
            SyntaxKind::GENERIC_COMMAND => self.visit_command(node),

            SyntaxKind::KEY_VALUE_BODY => self.visit_key_value_body(node),
            SyntaxKind::KEY_VALUE_PAIR => self.visit_key_value_pair(node),

            SyntaxKind::NEW_COMMAND_DEFINITION => self.visit_new_command(node),
            //SyntaxKind::OLD_COMMAND_DEFINITION => todo!(),
            //SyntaxKind::ENVIRONMENT_DEFINITION => todo!(),
            SyntaxKind::EQUATION => self.visit_display_math(node),

            //SyntaxKind::ERROR => todo!(),

            //SyntaxKind::COMMENT => todo!(),
            //SyntaxKind::BLOCK_COMMENT => todo!(),

            //SyntaxKind::KEY => todo!(),
            //SyntaxKind::VALUE => todo!(),
            SyntaxKind::ENVIRONMENT => self.visit_environment(node),
            SyntaxKind::TEXT => self.visit_text(node),
            SyntaxKind::ENUM_ITEM => self.visit_enum_item(node),

            SyntaxKind::PART
            | SyntaxKind::CHAPTER
            | SyntaxKind::SECTION
            | SyntaxKind::SUBSECTION
            | SyntaxKind::SUBSUBSECTION
            | SyntaxKind::PARAGRAPH
            | SyntaxKind::SUBPARAGRAPH => self.visit_section(node),

            //SyntaxKind::FORMULA => todo!(),
            //SyntaxKind::CAPTION => todo!(),
            //SyntaxKind::CITATION => todo!(),

            //SyntaxKind::LABEL_DEFINITION => todo!(),
            //SyntaxKind::LABEL_REFERENCE => todo!(),
            //SyntaxKind::LABEL_REFERENCE_RANGE => todo!(),
            //SyntaxKind::LABEL_NUMBER => todo!(),

            //SyntaxKind::MATH_OPERATOR => todo!(),

            //SyntaxKind::GLOSSARY_ENTRY_DEFINITION => todo!(),
            //SyntaxKind::GLOSSARY_ENTRY_REFERENCE => todo!(),

            //SyntaxKind::ACRONYM_DEFINITION => todo!(),
            //SyntaxKind::ACRONYM_DECLARATION => todo!(),
            //SyntaxKind::ACRONYM_REFERENCE => todo!(),

            //SyntaxKind::BIBITEM => todo!(),

            //SyntaxKind::TOC_CONTENTS_LINE => todo!(),
            //SyntaxKind::TOC_NUMBER_LINE => todo!(),
            _ => self.visit_children(node),
        }
    }

    fn visit_children(&mut self, node: &SyntaxNode) -> String {
        let mut output = "".to_string();
        for child in node.children_with_tokens() {
            output.push_str(&self.visit(&child));
        }
        output
    }

    fn visit_preamble(&mut self, node: &SyntaxNode) -> String {
        let mut output = String::new();

        for child in node.children() {
            match child.kind() {
                SyntaxKind::PACKAGE_INCLUDE | SyntaxKind::CLASS_INCLUDE => {
                    let line = self.visit_include(&child);
                    output.push_str(&line);
                    output.push('\n');
                }
                _ => {
                    output.push_str(&self.visit(&child.into()));
                }
            }
        }

        output
    }

    fn visit_include(&mut self, node: &SyntaxNode) -> String {
        let indent = indent_str(self.context.indent_level, self.context.tabstop);
        let cmd = node.first_token().unwrap().to_string();
        let mut options = String::new();
        let mut package_name = String::new();
        let mut newline = String::new();

        for child in node.children() {
            match child.kind() {
                SyntaxKind::BRACK_GROUP_KEY_VALUE => {
                    let raw = self.visit_node(&child);
                    options = raw.trim().to_string();
                }
                SyntaxKind::CURLY_GROUP_WORD_LIST => {
                    let raw = self.visit_node(&child);
                    package_name = raw.trim().to_string();
                }
                _ => {}
            }
        }

        match cmd.replace("\\", "").as_str() {
            "usetikzlibrary" | "setlength" | "addbibresource" | "bibliography" | "graphicspath"
            //| "setlength" | "parindent" | "parskip" | "newtheorem" | "newtheorem*"
            //| "declaretheorem" | "declaretheorem*" | "definecolor" | "definecolorset"
            //| "usepgflibrary" | "includegraphics" | "includesvg" | "includeinkscape"
            //| "verbatiminput" | "include" | "subfileinclude" | "input" | "subfile"
            //| "addbibresource" | "bibliography" | "import" | "subimport" | "inputfrom"
            | "subinputfrom" | "subincludefrom" | "pagestyle" | "documentclass" => {
                newline.push('\n');
            }
            _ => {}
        }

        format!("{indent}{cmd}{options}{package_name}{newline}")
    }

    fn visit_group(&mut self, node: &SyntaxNode) -> String {
        let inner = node
            .children_with_tokens()
            .map(|child| self.visit(&child))
            .collect::<String>();

        inner.to_string()
    }

    fn visit_curly_group(&mut self, unwrap: &SyntaxNode) -> String {
        let mut output = String::new();
        for child in unwrap.children() {
            output.push_str(&self.visit(&child.into()));
        }
        format!("{{{output}}}")
    }

    fn visit_curly_group_word_list(&mut self, node: &SyntaxNode) -> String {
        let mut words = Vec::new();

        for child in node.children() {
            if child.kind() == SyntaxKind::KEY {
                words.push(self.visit_key(&child));
            }
        }
        let content = words.join(", ");
        format!("{{{content}}}")
    }

    fn visit_curly_group_key_value(&self, _node: &SyntaxNode) -> String {
        todo!()
    }

    fn visit_brack_group_word(&mut self, unwrap: &SyntaxNode) -> String {
        let mut words = Vec::new();

        for child in unwrap.children() {
            if child.kind() == SyntaxKind::KEY {
                words.push(self.visit_key(&child));
            }
        }
        let content = words.join(", ");
        content.to_string()
    }

    fn visit_brack_group_key_value(
        &mut self,
        node: &rowan::SyntaxNode<latex::LatexLanguage>,
    ) -> String {
        let mut pairs = Vec::new();

        for child in node.children_with_tokens() {
            if child.kind() == SyntaxKind::KEY_VALUE_BODY {
                pairs.push(self.visit_node(child.as_node().unwrap()));
            }
        }

        format!("[{}]", pairs.join(", "))
    }

    fn visit_key(&self, node: &SyntaxNode) -> String {
        node.text().to_string().trim().to_string()
    }

    fn visit_key_value_pair(&mut self, node: &SyntaxNode) -> String {
        let mut key = String::new();
        let mut val = String::new();
        let mut output = String::new();

        for child in node.children() {
            match child.kind() {
                SyntaxKind::KEY => {
                    key.push_str(&self.visit_key(&child));
                }
                SyntaxKind::VALUE => {
                    val.push_str(&self.visit_value(&child));
                }
                _ => {}
            }
        }

        output.push_str(&key);
        if !val.is_empty() {
            output.push_str(" = ");
            output.push_str(&val);
        }
        output
    }

    fn visit_key_value_body(&mut self, node: &SyntaxNode) -> String {
        let mut pairs = Vec::new();
        let indent_level = self.context.indent_level + 1;
        let tab_stop = self.context.tabstop;
        let indent = indent_str(indent_level, tab_stop);

        for child in node.children() {
            if child.kind() == SyntaxKind::KEY_VALUE_PAIR {
                pairs.push(self.visit_key_value_pair(&child));
            }
        }

        if pairs.len() > 3 {
            let mut seperator = ",\n".to_string();
            seperator.push_str(&indent);
            let mut output = "\n".to_string();
            output.push_str(&indent);
            output.push_str(&pairs.join(&seperator));
            output.push('\n');
            output
        } else {
            pairs.join(", ")
        }
    }

    fn visit_value(&mut self, node: &SyntaxNode) -> String {
        let mut output = String::new();

        for child in node.children() {
            if child.kind() == SyntaxKind::TEXT {
                output.push_str(child.to_string().trim());
            }
        }
        output
    }

    fn visit_new_command(&mut self, node: &SyntaxNode) -> String {
        let name = node.first_token().unwrap().to_string();
        let mut command = String::new();
        let mut args = String::new();
        let mut content = String::new();

        for child in node.children() {
            match child.kind() {
                SyntaxKind::CURLY_GROUP_COMMAND => {
                    command.push_str(self.visit_node(&child).trim());
                }
                SyntaxKind::BRACK_GROUP_WORD => {
                    args.push_str(self.visit_node(&child).trim());
                }
                SyntaxKind::CURLY_GROUP => {
                    content.push_str(self.visit_node(&child).trim());
                }
                _ => {}
            }
        }
        let head = format!("{name}{command}{args}").len();
        let tail = content.len();
        if head + tail > 80 {
            let indent = indent_str(self.context.indent_level + 1, self.context.tabstop);
            //if indent.len() + tail - 2 < 80 {
            content = content.trim_end_matches('}').to_string();
            content = content.trim_end_matches('{').to_string();
            content = format!("{{\n{indent}{content}\n{indent}}}");
            //}
        }
        format!("{name}{command}{args}{content}\n")
    }

    fn visit_display_math(&mut self, node: &SyntaxNode) -> String {
        let mut output = String::new();
        for child in node.children_with_tokens() {
            match child.kind() {
                SyntaxKind::TEXT => {
                    output.push_str(&self.format_math(child.as_node().unwrap()));
                }
                _ => {
                    output.push_str(&self.visit(&child));
                }
            }
        }
        output
    }

    fn visit_environment(&mut self, node: &SyntaxNode) -> String {
        let env_name = match self.get_environment_name(node) {
            Some(name) => name,
            None => "ENV".to_string(), // fallback if we don't find a name
        };
        match env_name.as_str() {
            "document" => self.visit_generic_environment(node, env_name),
            "equation" | "equation*" => self.visit_equation_environment(node, env_name),
            "align" | "align*" | "aligned" | "alignedat" | "alignat" | "alignat*" => {
                self.visit_align_environment(node, env_name)
            }
            _ => self.visit_generic_environment(node, env_name),
        }
    }

    fn visit_equation_environment(&mut self, node: &SyntaxNode, env_name: String) -> String {
        let mut content = String::new();
        let outer_indent = indent_str(self.context.indent_level, self.context.tabstop);
        let inner_indent = indent_str(self.context.indent_level + 1, self.context.tabstop);
        for child in node.children_with_tokens() {
            match child.kind() {
                SyntaxKind::BEGIN | SyntaxKind::END => {}
                SyntaxKind::TEXT => {
                    content.push_str(self.format_math(child.as_node().unwrap()).trim());
                }
                SyntaxKind::WHITESPACE => {}
                _ => {
                    content.push_str(&self.visit(&child));
                }
            }
        }
        content = content.trim_end().to_string();
        let lines: Vec<String> = content
            .split("\n")
            .map(|line| [inner_indent.clone(), line.to_string()].join(""))
            .collect();
        content = lines.join("\n");
        format!(
            "\n{outer_indent}\\begin{{{env_name}}}\n{content}\n{outer_indent}\\end{{{env_name}}}"
        )
    }

    fn visit_generic_environment(&mut self, node: &SyntaxNode, env_name: String) -> String {
        let mut content = String::new();
        let outer_indent = indent_str(self.context.indent_level, self.context.tabstop);
        if env_name != *"document" {
            self.context.indent_level += 1;
        }
        let indent_level = self.context.indent_level;
        for child in node.children_with_tokens() {
            match child.kind() {
                SyntaxKind::BEGIN | SyntaxKind::END => {}
                SyntaxKind::ENUM_ITEM => {
                    content.push_str(&indent_str(self.context.indent_level, self.context.tabstop));
                    self.context.indent_level = 0;
                    content.push_str(self.visit(&child).trim());
                    content.push('\n');
                    self.context.indent_level = indent_level;
                }
                SyntaxKind::WHITESPACE => {}
                _ => {
                    content.push_str(&self.visit(&child));
                }
            }
        }
        content = content.trim_end().to_string();
        if env_name != *"document" {
            self.context.indent_level -= 1;
        }

        format!(
            "\n{outer_indent}\\begin{{{env_name}}}\n{content}\n{outer_indent}\\end{{{env_name}}}"
        )
    }

    fn visit_align_environment(&mut self, node: &SyntaxNode, env_name: String) -> String {
        self.context.math_mode = true;
        let mut content = String::new();

        let indent = indent_str(self.context.indent_level, self.context.tabstop);
        self.context.indent_level += 1;

        let parent_indent_level = self.context.indent_level;
        for child in node.children_with_tokens() {
            match child.kind() {
                SyntaxKind::BEGIN | SyntaxKind::END => {}
                SyntaxKind::TEXT => {
                    self.context.indent_level = 0;
                    content.push_str(&self.format_math(child.as_node().unwrap()));
                    self.context.indent_level = parent_indent_level;
                }
                _ => {
                    self.context.indent_level = 0;
                    content.push_str(&self.visit(&child));
                    self.context.indent_level = parent_indent_level;
                }
            }
        }

        content = content
            .trim_end()
            .to_string()
            .replace("\n", "")
            .replace("& =", "&=");
        let lines: Vec<&str> = content.split("\\\\").collect();
        let lines = lines.iter().map(|line| line.trim()).collect::<Vec<&str>>();
        let aligned_content = self
            .align_at_amp(lines.clone())
            .join(" \\\\\n")
            .replace("& =", "&=");

        self.context.indent_level -= 1;
        self.context.math_mode = false;
        format!("\n{indent}\\begin{{{env_name}}}\n{aligned_content}\n{indent}\\end{{{env_name}}}")
    }

    fn align_at_amp(&mut self, lines: Vec<&str>) -> Vec<String> {
        let split_lines: Vec<Vec<&str>> = lines
            .iter()
            .map(|line| line.split('&').map(|s| s.trim()).collect())
            .collect();

        let num_columns = split_lines.iter().map(|cols| cols.len()).max().unwrap_or(0);

        let mut max_widths = vec![0; num_columns];

        for cols in &split_lines {
            for (i, col) in cols.iter().enumerate() {
                if i < max_widths.len() && col.len() > max_widths[i] {
                    max_widths[i] = col.len();
                }
            }
        }

        let inner_indent = indent_str(self.context.indent_level, self.context.tabstop);

        let aligned_lines: Vec<String> = split_lines
            .iter()
            .map(|cols| {
                let padded_cols: Vec<String> = cols
                    .iter()
                    .enumerate()
                    .map(|(i, col)| {
                        let padding = max_widths[i].saturating_sub(col.len());
                        format!("{}{}", col, " ".repeat(padding))
                    })
                    .collect();
                format!("{}{}", inner_indent, padded_cols.join(" & "))
            })
            .collect();

        aligned_lines
    }

    fn get_environment_name(&mut self, node: &SyntaxNode) -> Option<String> {
        let subnode = node
            .children()
            .find(|child| child.kind() == SyntaxKind::BEGIN)?;

        let curly_group_word = subnode
            .children()
            .find(|child| child.kind() == SyntaxKind::CURLY_GROUP_WORD)?;

        let key_token = curly_group_word
            .children()
            .find(|child| child.kind() == SyntaxKind::KEY)?;

        Some(key_token.to_string())
    }

    fn visit_command(&mut self, node: &SyntaxNode) -> String {
        let name = node.first_token().unwrap().to_string();
        if name == "\\\\" {
            return "\\\\".to_string();
        }
        let name = name.replace("\\", "").trim().to_string();

        let temp_indent = self.context.indent_level;
        self.context.indent_level = 0;
        let mut output = String::new();

        for child in node.children_with_tokens() {
            match child.kind() {
                SyntaxKind::CURLY_GROUP => {
                    output.push_str(self.visit(&child).trim());
                }
                SyntaxKind::EQUALITY_SIGN => {
                    output.push('=');
                }
                SyntaxKind::TEXT => {
                    output.push_str(self.visit(&child).trim());
                }
                _ => {}
            }
        }

        self.context.indent_level = temp_indent;
        match name.as_str() {
            "usetikzlibrary" | "setlength" | "addbibresource" | "bibliography" | "graphicspath"
            //| "setlength" | "newtheorem" | "newtheorem*" | "declaretheorem" | "declaretheorem*"
            //| "definecolor" | "definecolorset" | "usepgflibrary" | "includegraphics"
            //| "includesvg" | "includeinkscape" | "verbatiminput" | "include" | "subfileinclude"
            //| "input" | "subfile" | "addbibresource" | "bibliography" | "import" | "subimport"
            | "inputfrom" | "subinputfrom" | "subincludefrom" | "pagestyle" => {
                output.push('\n');
            }
            _ => {}
        }
        format!("\\{name}{output}")
    }

    fn visit_section(&mut self, node: &SyntaxNode) -> String {
        self.context.indent_level = 0;
        let mut output = String::new();

        for child in node.children_with_tokens() {
            match child.kind() {
                SyntaxKind::CURLY_GROUP => {
                    output.push_str(&self.visit(&child));
                    if let Some(next) = child.next_sibling_or_token() {
                        if next.kind() != SyntaxKind::ENVIRONMENT {
                            output.push('\n');
                        }
                    }
                }
                _ => {
                    output.push_str(&self.visit(&child));
                }
            }
        }
        output = output.trim_end().to_string();
        format!("\n\n{output}")
    }

    fn visit_text(&self, node: &SyntaxNode) -> String {
        let indent = indent_str(self.context.indent_level, self.context.tabstop);
        let mut content = String::new();
        if self.context.math_mode {
            content.push_str(&self.format_math(node));
        } else {
            content.push_str(node.text().to_string().trim_matches('\n'));
        }
        format!("{indent}{content}")
    }

    fn format_math(&self, node: &SyntaxNode) -> String {
        let mut content = node.text().to_string().trim().to_string();
        content = content.replace(char::is_whitespace, "");
        let operators = ['+', '-', '/', '*', '=', '<', '>'];
        let mut padded_line = String::new();
        for c in content.chars() {
            if operators.contains(&c) {
                padded_line.push(' ');
                padded_line.push(c);
                padded_line.push(' ');
            } else {
                padded_line.push(c);
            }
        }
        padded_line.trim().to_string()
    }

    fn visit_enum_item(&mut self, node: &SyntaxNode) -> String {
        let mut output = String::new();
        for child in node.children_with_tokens() {
            match child.kind() {
                SyntaxKind::WHITESPACE => {}
                _ => {
                    output.push_str(self.visit(&child).trim());
                    output.push(' ');
                }
            }
        }
        output
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct LSTGraph {
    graph: Graph<String, String>,
}

impl LSTGraph {
    pub fn new() -> Self {
        let graph = Graph::<String, String>::new();
        LSTGraph { graph }
    }

    pub fn visit(&mut self, element: &SyntaxElement) -> NodeIndex {
        let mut output = "".to_string();
        match element {
            SyntaxElement::Node(node) => {
                output.push_str(format!("Kind: {:?}", node.kind()).as_str());
                let graph_node = self.graph.add_node(output);
                for child in node.children_with_tokens() {
                    let child_node = self.visit(&child);
                    self.graph.add_edge(graph_node, child_node, "".to_string());
                }
                graph_node
            }
            SyntaxElement::Token(token) => {
                output.push_str(token.text());
                self.graph.add_node(output)
            }
        }
    }

    pub fn print_graph(&self, path: &Option<PathBuf>) {
        let dot_filename = match path.clone() {
            Some(mut path) => {
                path.set_file_name("graph.dot");
                path
            }
            None => PathBuf::from("graph.dot"),
        };

        let mut file = File::create(dot_filename.clone()).unwrap();
        _ = file.write_fmt(format_args!(
            "{:?}",
            Dot::with_config(&self.graph, &[DotConfig::EdgeNoLabel])
        ));

        let png_filename = match path.clone() {
            Some(mut path) => {
                path.set_file_name("graph.svg");
                path
            }
            None => PathBuf::from("graph.svg"),
        };

        let output = Command::new("dot")
            .args([
                "-Tsvg",
                dot_filename.to_str().unwrap(),
                "-Nshape=box",
                "-o",
                png_filename.to_str().unwrap(),
            ])
            .spawn();

        match output {
            Ok(_) => log::debug!(
                "Graph of {} generated successfully at {}",
                dot_filename.to_str().unwrap(),
                png_filename.to_str().unwrap()
            ),
            Err(e) => log::debug!("Failed to execute dot command: {}", e),
        }
    }
}

#[cfg(test)]
mod tests;
