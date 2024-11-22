use std::io::prelude::*;
use std::process::Command;
use std::{fs::File, path::PathBuf};
use syntax::latex::{self, LatexLanguage, SyntaxKind};

use petgraph::{
    dot::{Config as DotConfig, Dot},
    graph::NodeIndex,
    Graph,
};

#[allow(dead_code)]
#[derive(Debug)]
struct CodeBlock {
    lines: Vec<String>
}

#[allow(dead_code)]
#[derive(Debug)]
struct FormattedDocument {
    doc: Vec<CodeBlock>
}

#[allow(dead_code)]
#[derive(Debug)]
enum Combinator {
    Juxtapose,
    Stack,
}

#[allow(dead_code)]
#[derive(Debug)]
struct FormatContext {
    tabstop: u8,
    indent_level: usize,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Formatter {
    context: FormatContext,
}

#[allow(dead_code)]
impl Formatter {
    pub fn new() -> Self {
        Self {
            context: FormatContext {
                tabstop: 0,
                indent_level: 0,
            },
        }
    }

    // Change to SyntaxElement? Need to get Tokens too.
    pub fn visit(&mut self, node: &latex::SyntaxNode) -> String {
        match node.kind() {
            SyntaxKind::ROOT => self.visit_root(node),
            SyntaxKind::ENVIRONMENT => self.visit_environment(node),
            SyntaxKind::BEGIN => self.visit_begin(node),
            SyntaxKind::END => self.visit_end(node),
            SyntaxKind::SECTION => self.visit_section(node),
            SyntaxKind::SUBSECTION => self.visit_subsection(node),
            SyntaxKind::LABEL_DEFINITION => self.visit_label_definition(node),
            SyntaxKind::WHITESPACE => self.visit_whitespace(node),
            SyntaxKind::COMMENT => self.visit_comment(node),
            SyntaxKind::EQUALITY_SIGN => self.visit_equality_sign(node),
            SyntaxKind::COMMAND_NAME => self.visit_command_name(node),
            SyntaxKind::PREAMBLE => self.visit_preamble(node),
            SyntaxKind::TEXT => self.visit_text(node),
            SyntaxKind::KEY => self.visit_key(node),
            SyntaxKind::CURLY_GROUP => self.visit_curly_group(node),
            SyntaxKind::CURLY_GROUP_WORD => self.visit_curly_group_word(node),
            SyntaxKind::CURLY_GROUP_WORD_LIST => self.visit_curly_group_word_list(node),
            SyntaxKind::PACKAGE_INCLUDE => self.visit_package_include(node),
            SyntaxKind::CLASS_INCLUDE => self.visit_class_include(node),

            SyntaxKind::EQUATION => todo!(),
            SyntaxKind::PART => todo!(),
            SyntaxKind::CHAPTER => todo!(),
            SyntaxKind::SUBSUBSECTION => todo!(),
            SyntaxKind::PARAGRAPH => todo!(),
            SyntaxKind::SUBPARAGRAPH => todo!(),
            SyntaxKind::ERROR => todo!(),
            SyntaxKind::VERBATIM => todo!(),
            SyntaxKind::L_CURLY => todo!(),
            SyntaxKind::R_CURLY => todo!(),
            SyntaxKind::L_BRACK => todo!(),
            SyntaxKind::R_BRACK => todo!(),
            SyntaxKind::L_PAREN => todo!(),
            SyntaxKind::R_PAREN => todo!(),
            SyntaxKind::COMMA => todo!(),
            SyntaxKind::WORD => todo!(),
            SyntaxKind::DOLLAR => todo!(),
            SyntaxKind::VALUE => todo!(),
            SyntaxKind::KEY_VALUE_PAIR => todo!(),
            SyntaxKind::KEY_VALUE_BODY => todo!(),
            SyntaxKind::CURLY_GROUP_COMMAND => todo!(),
            SyntaxKind::CURLY_GROUP_KEY_VALUE => todo!(),
            SyntaxKind::BRACK_GROUP => todo!(),
            SyntaxKind::BRACK_GROUP_WORD => todo!(),
            SyntaxKind::BRACK_GROUP_KEY_VALUE => todo!(),
            SyntaxKind::PAREN_GROUP => todo!(),
            SyntaxKind::MIXED_GROUP => todo!(),
            SyntaxKind::GENERIC_COMMAND => todo!(),
            SyntaxKind::ENUM_ITEM => todo!(),
            SyntaxKind::FORMULA => todo!(),
            SyntaxKind::CAPTION => todo!(),
            SyntaxKind::CITATION => todo!(),
            SyntaxKind::LATEX_INCLUDE => todo!(),
            SyntaxKind::BIBLATEX_INCLUDE => todo!(),
            SyntaxKind::BIBTEX_INCLUDE => todo!(),
            SyntaxKind::GRAPHICS_INCLUDE => todo!(),
            SyntaxKind::SVG_INCLUDE => todo!(),
            SyntaxKind::INKSCAPE_INCLUDE => todo!(),
            SyntaxKind::VERBATIM_INCLUDE => todo!(),
            SyntaxKind::IMPORT => todo!(),
            SyntaxKind::LABEL_REFERENCE => todo!(),
            SyntaxKind::LABEL_REFERENCE_RANGE => todo!(),
            SyntaxKind::LABEL_NUMBER => todo!(),
            SyntaxKind::OLD_COMMAND_DEFINITION => todo!(),
            SyntaxKind::NEW_COMMAND_DEFINITION => todo!(),
            SyntaxKind::MATH_OPERATOR => todo!(),
            SyntaxKind::GLOSSARY_ENTRY_DEFINITION => todo!(),
            SyntaxKind::GLOSSARY_ENTRY_REFERENCE => todo!(),
            SyntaxKind::ACRONYM_DEFINITION => todo!(),
            SyntaxKind::ACRONYM_DECLARATION => todo!(),
            SyntaxKind::ACRONYM_REFERENCE => todo!(),
            SyntaxKind::THEOREM_DEFINITION_AMSTHM => todo!(),
            SyntaxKind::THEOREM_DEFINITION_THMTOOLS => todo!(),
            SyntaxKind::COLOR_REFERENCE => todo!(),
            SyntaxKind::COLOR_DEFINITION => todo!(),
            SyntaxKind::COLOR_SET_DEFINITION => todo!(),
            SyntaxKind::TIKZ_LIBRARY_IMPORT => todo!(),
            SyntaxKind::ENVIRONMENT_DEFINITION => todo!(),
            SyntaxKind::GRAPHICS_PATH => todo!(),
            SyntaxKind::BLOCK_COMMENT => todo!(),
            SyntaxKind::BIBITEM => todo!(),
            SyntaxKind::TOC_CONTENTS_LINE => todo!(),
            SyntaxKind::TOC_NUMBER_LINE => todo!(),
        }
    }

    fn visit_begin(&mut self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        let indent = "\t".repeat(self.context.indent_level);
        let child = node.first_child();
        let word = match child {
            Some(child) => self.visit(&child),
            None => "".to_string(),
        };
        let mut newline = "".to_string();
        if word == *"{document}" {
            newline.push_str("\n\n");
        } else {
            self.context.indent_level += 1;
        }

        format!("{newline}{indent}\\begin{}", word)
    }

    fn visit_end(&mut self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        if self.context.indent_level > 0 {
            self.context.indent_level -= 1;
        }
        let indent = "\t".repeat(self.context.indent_level);
        let child = node.first_child();
        let word = match child {
            Some(child) => self.visit(&child),
            None => "".to_string(),
        };
        format!("\n{indent}\\end{}", word)
    }

    fn visit_curly_group_word(&self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        let child = node.first_child();
        let word = match child {
            Some(child) => child.text().to_string(),
            None => "".to_string(),
        };
        format!("{{{}}}", word)
    }

    fn visit_environment(&mut self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        let mut output = "".to_string();
        for child in node.children() {
            output.push_str(&self.visit(&child));
        }
        output
    }

    fn visit_label_definition(&mut self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        let mut output = "".to_string();
        for child in node.children() {
            output.push_str(&self.visit(&child));
        }
        if let Some(sibling) = node.next_sibling() {
            if sibling.kind() == latex::TEXT {
                let indent = "\t".repeat(self.context.indent_level);
                output.push_str(format!("\n{indent}").as_str());
            } else {
                output.push('\n');
            }
        }
        format!("\\label{}", output)
    }

    fn visit_curly_group(&mut self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        let mut output = "{".to_string();
        for child in node.children() {
            output.push_str(&self.visit(&child));
        }
        output.push('}');
        if let Some(sibling) = node.next_sibling() {
            if sibling.kind() == latex::TEXT {
                let indent = "\t".repeat(self.context.indent_level);
                output.push_str(format!("\n{indent}").as_str());
            }
        }
        output
    }

    fn visit_text(&self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        // This might be more robust if we use regex
        node.text()
            .to_string()
            .trim_start_matches(['\n', '\t'])
            .trim_end_matches(['\n', '\t'])
            .to_string()
    }

    fn visit_equality_sign(&self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        node.text().to_string()
    }

    fn visit_section(&mut self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        //TODO: Check for star version
        let mut output = "".to_string();
        let mut newline = "".to_string();
        let indent = "\t".repeat(self.context.indent_level);
        self.context.indent_level += 1;
        if let Some(sibling) = node.prev_sibling() {
            if sibling.kind() != SyntaxKind::END {
                newline.push('\n');
            }
        }
        for child in node.children() {
            output.push_str(&self.visit(&child));
        }
        self.context.indent_level -= 1;
        format!("\n{newline}{indent}\\section{}", output)
    }

    fn visit_subsection(&mut self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        //TODO: Check for star version
        let mut output = "".to_string();
        let mut newline = "".to_string();
        let indent = "\t".repeat(self.context.indent_level);
        self.context.indent_level += 1;
        if let Some(sibling) = node.prev_sibling() {
            if sibling.kind() != SyntaxKind::END {
                newline.push('\n');
            }
        }
        for child in node.children() {
            output.push_str(&self.visit(&child));
        }
        self.context.indent_level -= 1;
        format!("\n{newline}{indent}\\subsection{}", output)
    }

    fn visit_whitespace(&self, _node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        todo!()
    }

    fn visit_comment(&self, _node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        todo!()
    }

    fn visit_command_name(&self, _node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        todo!()
    }

    fn visit_preamble(&mut self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        let mut output = "".to_string();
        for child in node.children() {
            output.push_str(&self.visit(&child));
        }
        output
    }

    fn visit_key(&self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        node.text().to_string()
    }

    fn visit_curly_group_word_list(&mut self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        let mut output = "{".to_string();
        for child in node.children() {
            output.push_str(&self.visit(&child));
        }
        output.push('}');
        if let Some(sibling) = node.next_sibling() {
            if sibling.kind() == latex::TEXT {
                let indent = "\t".repeat(self.context.indent_level);
                output.push_str(format!("\n{indent}").as_str());
            }
        }
        output
    }

    fn visit_class_include(&mut self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        let mut output = "".to_string();
        for child in node.children() {
            output.push_str(&self.visit(&child));
        }
        format!("\\documentclass{}", output)
    }

    fn visit_package_include(&mut self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        let mut output = "".to_string();
        for child in node.children() {
            output.push_str(&self.visit(&child));
        }
        format!("\n\\usepackage{}", output)
    }

    fn visit_root(&mut self, node: &rowan::SyntaxNode<LatexLanguage>) -> String {
        let mut output = "".to_string();
        for child in node.children() {
            output.push_str(&self.visit(&child));
        }
        output
    }
}

pub struct LSTGraph {
    graph: Graph<String, String>,
}

impl LSTGraph {
    pub fn new() -> Self {
        let graph = Graph::<String, String>::new();
        LSTGraph { graph }
    }

    pub fn visit(&mut self, element: &latex::SyntaxElement) -> NodeIndex {
        let mut output = "".to_string();
        match element {
            latex::SyntaxElement::Node(node) => {
                output.push_str(format!("Kind: {:?}", node.kind()).as_str());
                let graph_node = self.graph.add_node(output);
                for child in node.children_with_tokens() {
                    let child_node = self.visit(&child);
                    self.graph.add_edge(graph_node, child_node, "".to_string());
                }
                graph_node
            }
            latex::SyntaxElement::Token(token) => {
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
                path.set_file_name("graph.png");
                path
            }
            None => PathBuf::from("graph.png"),
        };

        let output = Command::new("dot")
            .args([
                "-Tpng",
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
