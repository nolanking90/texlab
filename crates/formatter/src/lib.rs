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

#[derive(Default)]
struct FormatContext {
    tabstop: u8,
    indent_level: usize,
}

#[derive(Default)]
pub struct Formatter {
    context: FormatContext,
}

fn indent_str(indent_level: usize, tabstop: u8) -> String {
    " ".repeat(indent_level * tabstop as usize)
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
            latex::SyntaxElement::Token(token) => self.visit_token(token),
        }
    }

    fn visit_token(&self, token: &latex::SyntaxToken) -> String {
        token.text().to_string()
    }

    fn visit_node(&mut self, node: &latex::SyntaxNode) -> String {
        match node.kind() {
            SyntaxKind::PREAMBLE => self.visit_preamble(node),
            SyntaxKind::BRACK_GROUP_KEY_VALUE => self.visit_brack_group_key_value(node),
            SyntaxKind::CURLY_GROUP_WORD_LIST => self.visit_curly_group_word_list(node),
            SyntaxKind::KEY_VALUE_BODY => self.visit_key_value_body(node),
            SyntaxKind::KEY_VALUE_PAIR => self.visit_key_value_pair(node),
            SyntaxKind::NEW_COMMAND_DEFINITION => self.visit_new_command(node),
            SyntaxKind::EQUATION => self.visit_equation(node),
            _ => self.visit_children(node),
        }
    }

    fn visit_children(&mut self, node: &latex::SyntaxNode) -> String {
        let mut output = "".to_string();
        for child in node.children_with_tokens() {
            output.push_str(&self.visit(&child));
        }
        output
    }

    fn visit_preamble(&mut self, node: &latex::SyntaxNode) -> String {
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

        output.push('\n');
        output
    }

    fn visit_include(&mut self, node: &latex::SyntaxNode) -> String {
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
        if cmd == "\\documentclass" {
            newline.push('\n');
        }

        format!("{indent}{cmd}{options}{{{package_name}}}{newline}")
    }

    fn visit_key(&self, node: &latex::SyntaxNode) -> String {
        node.text().to_string().trim().to_string()
    }

    fn visit_curly_group_word_list(&mut self, node: &latex::SyntaxNode) -> String {
        let mut words = Vec::new();

        for child in node.children() {
            if child.kind() == SyntaxKind::KEY {
                words.push(self.visit_key(&child));
            }
        }
        let content = words.join(", ");
        content.to_string()
    }

    fn visit_root(&mut self, node: &latex::SyntaxNode) -> String {
        let mut output = "".to_string();
        for child in node.children_with_tokens() {
            output.push_str(&self.visit(&child));
        }
        output
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

    fn visit_key_value_pair(&mut self, node: &latex::SyntaxNode) -> String {
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

    fn visit_key_value_body(&mut self, node: &latex::SyntaxNode) -> String {
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

    fn visit_value(&mut self, node: &latex::SyntaxNode) -> String {
        let mut output = String::new();

        for child in node.children() {
            if child.kind() == SyntaxKind::TEXT {
                output.push_str(child.to_string().trim());
            }
        }
        output
    }

    fn visit_new_command(&mut self, node: &latex::SyntaxNode) -> String {
        let mut output = String::new();
        let mut command = String::new();
        let mut args = String::new();
        let mut content = String::new();

        for child in node.children_with_tokens() {
            match child.kind() {
                SyntaxKind::CURLY_GROUP_COMMAND => {
                    command.push_str(child.to_string().trim());
                }
                SyntaxKind::BRACK_GROUP_WORD => {
                    args.push_str(&self.visit_brack_group_word(child.as_node().unwrap()));
                }
                SyntaxKind::CURLY_GROUP => {
                    content.push_str(self.visit_curly_group(child.as_node().unwrap()).trim());
                }
                _ => {}
            }
        }

        output.push_str("\\newcommand");
        output.push_str(&command);
        if !args.is_empty() {
            output.push_str("[");
            output.push_str(&args);
            output.push_str("]");
        }
        if !content.is_empty() {
            output.push_str("{");
            output.push_str(&content);
            output.push_str("}");
        }
        output.push('\n');
        output
    }

    fn visit_curly_group(&mut self, unwrap: &latex::SyntaxNode) -> String {
        let mut output = String::new();
        for child in unwrap.children() {
            output.push_str(&self.visit(&child.into()));
        }
        output
    }

    fn visit_brack_group_word(&mut self, unwrap: &latex::SyntaxNode) -> String {
        let mut words = Vec::new();

        for child in unwrap.children() {
            if child.kind() == SyntaxKind::KEY {
                words.push(self.visit_key(&child));
            }
        }
        let content = words.join(", ");
        content.to_string()
    }

    fn visit_equation(&self, node: &latex::SyntaxNode) -> String {
        let binding = node.text().to_string();
        let text = binding.trim();
        if text.starts_with("$$") && text.ends_with("$$") {
            let equation = &text[2..text.len() - 2].trim();
            format!("$$\n{}\n$$", equation)
        } else if text.starts_with("\\[") && text.ends_with("\\]") {
            let equation = &text[2..text.len() - 2].trim();
            format!("\\[\n{}\n\\]", equation)
        } else {
            format!("$$\n{}\n$$", text)
        }
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

#[cfg(test)]
mod tests;
