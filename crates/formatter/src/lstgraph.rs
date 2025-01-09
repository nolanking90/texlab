use std::io::prelude::*;
use std::process::Command;
use std::{fs::File, path::PathBuf};
use syntax::latex::SyntaxElement;
use petgraph::{
    dot::{Config as DotConfig, Dot},
    graph::NodeIndex,
    Graph,
};

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
                output.push_str(format!("Kind: {:?} -- {}", token.kind(), token.to_string()).as_str());
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
