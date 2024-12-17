#![allow(dead_code)]
use std::io::prelude::*;
use std::process::Command;
use std::{fs::File, path::PathBuf};
use syntax::latex::{self, SyntaxKind};

use petgraph::{
    dot::{Config as DotConfig, Dot},
    graph::NodeIndex,
    Graph,
};

struct CodeBlock {
    lines: Vec<String>,
}

enum Combinator {
    Horizontal,
    Veritical,
    CodeBlock(CodeBlock),
}

struct FormattedDocument {
    lines: Vec<Combinator>
}

struct FormatContext {
    tabstop: u8,
    indent_level: usize,
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
            },
        }
    }

    pub fn visit(&mut self, element: &latex::SyntaxElement) -> String {
        match element {
            latex::SyntaxElement::Node(node) => self.visit_node(node),
            latex::SyntaxElement::Token(token) => self.visit_token(token)
        }
    }

    fn visit_token(&self, token: &latex::SyntaxToken) -> String {
        token.text().to_string()
    }

    fn visit_node(&mut self, node: &latex::SyntaxNode) -> String {
        match node.kind() {
           SyntaxKind::KEY => self.visit_key(node), 
            _ => self.visit_children(node)
        }
    }

    fn visit_children(&mut self, node: &latex::SyntaxNode) -> String {
        let mut output = "".to_string();
        for child in node.children_with_tokens() {
            output.push_str(&self.visit(&child));
        }
        output
    }

    fn visit_begin(&mut self, _node: &latex::SyntaxNode) -> String {
        todo!()
    }

    fn visit_end(&mut self, _node: &latex::SyntaxNode) -> String {
        todo!()
    }

    fn visit_curly_group_word(&self, node: &latex::SyntaxNode) -> String {
        let child = node.first_child();
        let word = match child {
            Some(child) => child.text().to_string(),
            None => "".to_string(),
        };
        format!("{{{}}}", word)
    }

    fn visit_environment(&mut self, node: &latex::SyntaxNode) -> String {
        let mut output = "".to_string();
        for child in node.children_with_tokens() {
            output.push_str(&self.visit(&child));
        }
        output
    }

    fn visit_label_definition(&mut self, node: &latex::SyntaxNode) -> String {
        let mut output = "".to_string();
        for child in node.children_with_tokens() {
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

    fn visit_curly_group(&mut self, node: &latex::SyntaxNode) -> String {
        let mut output = "{".to_string();
        for child in node.children_with_tokens() {
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

    fn visit_text(&self, node: &latex::SyntaxNode) -> String {
        // This might be more robust if we use regex
        node.text()
            .to_string()
            .trim_start_matches(['\n', '\t'])
            .trim_end_matches(['\n', '\t'])
            .to_string()
    }

    fn visit_equality_sign(&self, node: &latex::SyntaxNode) -> String {
        node.text().to_string()
    }

    fn visit_section(&mut self, node: &latex::SyntaxNode) -> String {
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
        for child in node.children_with_tokens() {
            output.push_str(&self.visit(&child));
        }
        self.context.indent_level -= 1;
        format!("\n{newline}{indent}\\section{}", output)
    }

    fn visit_subsection(&mut self, node: &latex::SyntaxNode) -> String {
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
        for child in node.children_with_tokens() {
            output.push_str(&self.visit(&child));
        }
        self.context.indent_level -= 1;
        format!("\n{newline}{indent}\\subsection{}", output)
    }

    fn visit_whitespace(&self, _node: &latex::SyntaxNode) -> String {
        todo!()
    }

    fn visit_comment(&self, _node: &latex::SyntaxNode) -> String {
        todo!()
    }

    fn visit_command_name(&self, _node: &latex::SyntaxNode) -> String {
        todo!()
    }

    fn visit_preamble(&mut self, node: &latex::SyntaxNode) -> String {
        let mut output = "".to_string();
        for child in node.children_with_tokens() {
            output.push_str(&self.visit(&child));
        }
        output
    }

    fn visit_key(&self, node: &latex::SyntaxNode) -> String {
        node.text().to_string()
    }

    fn visit_curly_group_word_list(&mut self, node: &latex::SyntaxNode) -> String {
        let mut output = "{".to_string();
        for child in node.children_with_tokens() {
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

    fn visit_class_include(&mut self, node: &latex::SyntaxNode) -> String {
        let mut output = "".to_string();
        for child in node.children_with_tokens() {
            output.push_str(&self.visit(&child));
        }
        format!("\\documentclass{}", output)
    }

    fn visit_package_include(&mut self, node: &latex::SyntaxNode) -> String {
        let mut output = "".to_string();
        for child in node.children_with_tokens() {
            output.push_str(&self.visit(&child));
        }
        format!("\n\\usepackage{}", output)
    }

    fn visit_root(&mut self, node: &latex::SyntaxNode) -> String {
        let mut output = "".to_string();
        for child in node.children_with_tokens() {
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
