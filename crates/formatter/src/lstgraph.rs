use petgraph::{
    dot::{Config as DotConfig, Dot},
    graph::NodeIndex,
    Graph,
};
use std::io::{self, prelude::*};
use std::process::Command;
use std::{fs::File, path::PathBuf};
use syntax::latex::{SyntaxElement, SyntaxNode, SyntaxToken};

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
                output.push_str(format!("Kind: {:?} -- {}", token.kind(), token).as_str());
                self.graph.add_node(output)
            }
        }
    }

    fn walk(
        node: &SyntaxNode,
        counter: &mut usize,
        output: &mut dyn Write,
        group: Option<&str>,
    ) -> io::Result<usize> {
        let my_id = *counter;
        *counter += 1;
        let group_attr = if let Some(g) = group {
            format!(", group=\"{}\"", g)
        } else {
            "".to_string()
        };

        writeln!(
            output,
            "    node{} [shape=plaintext, label={}{}];",
            my_id,
            Self::html_label_for_node(node),
            group_attr
        )?;

        for child in node.children_with_tokens() {
            match child {
                SyntaxElement::Node(ref child_node) => {
                    let child_group = format!("group{}", *counter);
                    let child_id = Self::walk(child_node, counter, output, Some(&child_group))?;
                    writeln!(output, "    node{} -> node{};", my_id, child_id)?;
                }
                SyntaxElement::Token(ref token) => {
                    let token_group = format!("group{}", *counter);
                    let token_id = *counter;
                    *counter += 1;
                    writeln!(
                        output,
                        "    node{} [shape=plaintext, label={} , group=\"{}\"];",
                        token_id,
                        Self::html_label_for_token(token),
                        token_group
                    )?;
                    writeln!(output, "    node{} -> node{};", my_id, token_id)?;
                }
            }
        }
        Ok(my_id)
    }

    fn write_dot(root: &SyntaxNode, output: &mut dyn Write) -> io::Result<()> {
        writeln!(output, "digraph G {{")?;
        let mut counter = 0;
        Self::walk(root, &mut counter, output, None)?;
        writeln!(output, "}}")?;
        Ok(())
    }

    fn html_label_for_node(node: &SyntaxNode) -> String {
        format!(
            r#"<
<TABLE BORDER="0" CELLBORDER="1" CELLSPACING="0">
  <TR><TD>Node</TD></TR>
  <TR><TD>{:?}</TD></TR>
  <TR><TD>&nbsp;</TD></TR>
</TABLE>
>"#,
            node.kind()
        )
    }

    fn html_label_for_token(token: &SyntaxToken) -> String {
        format!(
            r#"<
<TABLE BORDER="0" CELLBORDER="1" CELLSPACING="0">
  <TR><TD>Token</TD></TR>
  <TR><TD>{:?}</TD></TR>
  <TR><TD>{}</TD></TR>
</TABLE>
>"#,
            token.kind(),
            token
        )
    }

    pub fn print_tree(&self, root: &SyntaxNode, path: &Option<PathBuf>) {
        let dot_filename = match path.clone() {
            Some(mut path) => {
                path.set_file_name("tree.dot");
                path
            }
            None => PathBuf::from("tree.dot"),
        };

        let mut file = File::create(dot_filename.clone()).unwrap();
        _ = Self::write_dot(root, &mut file);

        let png_filename = match path.clone() {
            Some(mut path) => {
                path.set_file_name("tree.svg");
                path
            }
            None => PathBuf::from("tree.svg"),
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
                path.set_file_name("graphCustom.svg");
                path
            }
            None => PathBuf::from("graphCustom.svg"),
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
