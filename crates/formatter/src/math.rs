#![allow(dead_code)]
use syntax::latex::{SyntaxElement, SyntaxKind, SyntaxNode};

use crate::formattable::{indent_str, Formattable};

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

pub enum MathElement {
    Parent(MathParent),
    Command(MathCommand),
    Environment(MathEnvironment),
    Text(String),
    CurlyGroup(MathCurlyGroup),
    MixedGroup(MathMixedGroup),
    BrackGroup(MathBrackGroup),
}

impl MathElement {
    pub fn from(element: &SyntaxElement) -> MathElement {
        match element {
            SyntaxElement::Node(node) => match node.kind() {
                SyntaxKind::MATH_OPERATOR
                | SyntaxKind::COLOR_REFERENCE
                | SyntaxKind::GENERIC_COMMAND
                | SyntaxKind::LABEL_DEFINITION => MathElement::Command(MathCommand::from(node)),
                SyntaxKind::ENVIRONMENT => MathElement::Environment(MathEnvironment::from(node)),
                SyntaxKind::TEXT | SyntaxKind::KEY => {
                    let mut txt = node.to_string().trim().to_string();
                    if txt.len() > 1 {
                        for op in ["+", "-", "*", "/", ">", "<"] {
                            let mut words: Vec<&str> = txt.split(op).collect();
                            words = words.iter().map(|w| w.trim()).collect();
                            txt = words.join([" ", op, " "].join("").as_str()).to_string();
                        }
                        let mut words: Vec<&str> = txt.split(",").collect();
                        words = words.iter().map(|w| w.trim()).collect();
                        txt = words.join(", ").to_string();
                    }
                    MathElement::Text(txt)
                }
                SyntaxKind::CURLY_GROUP => MathElement::CurlyGroup(MathCurlyGroup::from(node)),
                SyntaxKind::BRACK_GROUP => MathElement::BrackGroup(MathBrackGroup::from(node)),
                SyntaxKind::CURLY_GROUP_WORD | SyntaxKind::CURLY_GROUP_WORD_LIST => {
                    MathElement::CurlyGroup(MathCurlyGroup::from(node))
                }
                SyntaxKind::MIXED_GROUP => MathElement::MixedGroup(MathMixedGroup::from(node)),
                _ => MathElement::Text(String::new()),
            },
            SyntaxElement::Token(token) => {
                if matches!(token.to_string().as_str(), "=") {
                    MathElement::Text(token.to_string())
                } else {
                    MathElement::Text(String::new())
                }
            }
        }
    }

}

pub struct MathParent {
    pub children: Vec<MathElement>,
}

impl MathParent {
    pub fn from(node: &SyntaxNode) -> Self {
        Self {
            children: node
                .children_with_tokens()
                .map(|child| MathElement::from(&child))
                .collect(),
        }
    }
}

pub struct MathBrackGroup {
    pub children: Vec<MathElement>,
}

impl MathBrackGroup {
    fn from(node: &SyntaxNode) -> Self {
        let mut children = Vec::new();
        for child in node.children() {
            match child.kind() {
                SyntaxKind::KEY_VALUE_BODY => {
                    children.extend(
                        child
                            .children()
                            .map(|c| MathElement::from(&SyntaxElement::Node(c))),
                    );
                }
                _ => {
                    children.push(MathElement::from(&SyntaxElement::Node(child)));
                }
            }
        }
        Self { children }
    }

}

pub struct MathCurlyGroup {
    pub body: MathParent,
}

impl MathCurlyGroup {
    fn from(node: &SyntaxNode) -> Self {
        Self {
            body: MathParent::from(node),
        }
    }

}

pub struct MathMixedGroup {
    pub children: Vec<MathElement>,
    pub open_delim: String,
    pub close_delim: String,
}

impl MathMixedGroup {
    fn from(node: &SyntaxNode) -> Self {
        let open_delim = node.first_token().unwrap().to_string();
        let close_delim = match open_delim.as_str() {
            "{" => "}",
            "(" => ")",
            "[" => "]",
            _ => "",
        }
        .to_string();

        Self {
            children: node
                .children()
                .map(|child| MathElement::from(&SyntaxElement::Node(child)))
                .collect(),
            open_delim,
            close_delim,
        }
    }
}

pub struct MathCommand {
    pub name: String,
    pub args: Vec<MathElement>,
}

impl MathCommand {
    pub fn from(node: &SyntaxNode) -> Self {
        let name = node.first_token().unwrap().to_string();
        let mut args = Vec::new();

        for child in node.children() {
            match child.kind() {
                SyntaxKind::CURLY_GROUP => {
                    args.push(MathElement::CurlyGroup(MathCurlyGroup::from(&child)));
                }
                SyntaxKind::MIXED_GROUP => {
                    args.push(MathElement::MixedGroup(MathMixedGroup::from(&child)));
                }
                SyntaxKind::CURLY_GROUP_WORD | SyntaxKind::CURLY_GROUP_WORD_LIST => {
                    args.push(MathElement::CurlyGroup(MathCurlyGroup::from(&child)));
                }
                SyntaxKind::BRACK_GROUP_KEY_VALUE | SyntaxKind::BRACK_GROUP_WORD => {
                    args.push(MathElement::BrackGroup(MathBrackGroup::from(&child)));
                }
                _ => {}
            }
        }

        Self { name, args }
    }

}

pub struct MathEnvironment {
    pub name: String,
    pub args: Vec<MathElement>,
    pub body: MathParent,
}

impl MathEnvironment {
    pub fn from(node: &SyntaxNode) -> Self {
        let name = node
            .first_child()
            .unwrap()
            .first_child()
            .unwrap()
            .first_child()
            .unwrap()
            .to_string();

        let mut args = Vec::new();
        let first_child = node.first_child().unwrap();
        let opt_args: Vec<SyntaxNode> = first_child
            .children()
            .filter(|child| {
                matches!(
                    child.kind(),
                    SyntaxKind::BRACK_GROUP | SyntaxKind::CURLY_GROUP 
                )
            })
            .collect();
        let opt_args = MathElement::Text(
            opt_args
                .iter()
                .map(|node| node.to_string())
                .collect::<Vec<String>>()
                .join("")
                .split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>()
                .join(", "),
        );
        args.push(opt_args);

        let mut second_child = first_child.next_sibling().unwrap();
        if matches!(second_child.kind(), SyntaxKind::CURLY_GROUP) {
            args.push(MathElement::Text(
                second_child
                    .to_string()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
            ));
        } else {
            second_child = first_child.clone();
        }

        let children: Vec<SyntaxElement> = node
            .children_with_tokens()
            .filter(|child| {
                *child != SyntaxElement::Node(first_child.clone())
                    && *child != SyntaxElement::Node(second_child.clone())
            })
            .collect();
        let body: MathParent = MathParent {
            children: children
                .iter()
                .map(|node: &SyntaxElement| MathElement::from(node))
                .filter(|e| !e.format(0, 0, 100).join("").is_empty())
                .collect(),
        };

        Self { name, args, body }
    }

    pub fn align_at_amp(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let lines = self.body.format(indent_level, tabstop, line_length);

        if lines.iter().filter(|line| line.contains('&')).count() == 0 {
            return lines;
        }

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

        let inner_indent = indent_str(indent_level, tabstop);

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
            .filter(|line| !line.trim().is_empty())
            .collect();

        aligned_lines
    }
}
