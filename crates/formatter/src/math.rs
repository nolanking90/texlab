use syntax::latex::{SyntaxElement, SyntaxKind, SyntaxNode};

use crate::tex;

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

    pub fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        match self {
            MathElement::Command(cmd) => cmd.format(indent_level, tabstop, line_length),
            MathElement::Environment(env) => env.format(indent_level, tabstop, line_length),
            MathElement::Text(text) => vec![text.to_string().trim().to_string()],
            MathElement::Parent(p) => p.format(indent_level, tabstop, line_length),
            MathElement::CurlyGroup(group) => group.format(indent_level, tabstop, line_length),
            MathElement::BrackGroup(group) => group.format(indent_level, tabstop, line_length),
            MathElement::MixedGroup(group) => group.format(indent_level, tabstop, line_length),
        }
    }
}

pub struct MathParent {
    children: Vec<MathElement>,
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

    pub fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = Vec::new();
        let mut line = String::new();
        for (i, child) in self.children.iter().enumerate() {
            match child {
                MathElement::Command(cmd) => {
                    let fmt = cmd
                        .format(indent_level, tabstop, line_length - line.len())
                        .join("")
                        .trim()
                        .to_string();
                    if fmt.len() + line.len() > line_length {
                        if !line.trim().is_empty() {
                            output.push(line);
                            line = String::new();
                        }
                        let fmt = cmd.format(indent_level, tabstop, line_length);
                        if fmt.len() == 1 {
                            line.push_str(fmt[0].trim_end());
                        } else { 
                            output.extend(fmt[..fmt.len() - 1].to_vec());
                            line.push_str(&fmt[fmt.len() - 1]);
                        }
                    } else {
                        line.push_str(&fmt);
                    }
                    if line.ends_with("\\\\") {
                        output.push(line.trim().to_string());
                        line = String::new();
                    } else {
                        if let Some(MathElement::Text(txt)) = self.children.get(i + 1) {
                            if !matches!(
                                txt.chars().nth(0),
                                Some(',') | Some('^') | Some('_') | Some('\'') | Some('.')
                            ) {
                                line.push(' ');
                            }
                        }
                        if let Some(MathElement::Command(_)) = self.children.get(i + 1) {
                            line.push(' ');
                        }
                    }
                }
                MathElement::Text(text) if !(text.trim().is_empty()) => {
                    line.push_str(text.trim());
                    if i != self.children.len() - 1 && !line.ends_with('^') {
                        if let Some(MathElement::MixedGroup(_)) = self.children.get(i + 1) {
                        } else if let Some(MathElement::Text(txt)) = self.children.get(i + 1) {
                            if !matches!(txt.chars().nth(0), Some('^'))
                                && (!line.ends_with('&')
                                    || !matches!(txt.chars().nth(0), Some('=')))
                            {
                                line.push(' ');
                            }
                        } else {
                            line.push(' ');
                        }
                    }
                }
                MathElement::Environment(env) => {
                    if !line.trim().is_empty() {
                        output.push(line);
                        line = String::new();
                    }
                    output.extend(env.format(indent_level, tabstop, line_length));
                }
                MathElement::Parent(p) => {
                    if !line.trim().is_empty() {
                        output.push(line.to_string());
                        line = String::new();
                    }
                    let fmt = p.format(indent_level, tabstop, line_length);
                    if !fmt.join("").trim().is_empty() {
                        output.extend(fmt);
                    }
                }
                MathElement::CurlyGroup(group) => {
                    line.push_str(
                        group
                            .format(indent_level, tabstop, line_length)
                            .join("")
                            .trim(),
                    );
                    if let Some(MathElement::Text(txt)) = self.children.get(i + 1) {
                        if !matches!(
                            txt.chars().nth(0),
                            Some('.') | Some(',') | Some('^') | Some('_') | Some('-')
                        ) {
                            line.push(' ');
                        }
                    }
                    if let Some(MathElement::Command(_)) = self.children.get(i + 1) {
                        line.push(' ');
                    }
                }
                MathElement::BrackGroup(group) => {
                    line.push_str(
                        group
                            .format(indent_level, tabstop, line_length)
                            .join("")
                            .trim(),
                    );
                    if let Some(MathElement::Text(txt)) = self.children.get(i + 1) {
                        if !matches!(
                            txt.chars().nth(0),
                            Some('.') | Some(',') | Some('^') | Some('_') | Some('-')
                        ) {
                            line.push(' ');
                        }
                    }
                }
                MathElement::MixedGroup(group) => {
                    line.push_str(
                        group
                            .format(indent_level, tabstop, line_length)
                            .join("")
                            .trim(),
                    );
                    if let Some(MathElement::Text(txt)) = self.children.get(i + 1) {
                        if !matches!(
                            txt.chars().nth(0),
                            Some('.') | Some(',') | Some('^') | Some('_') | Some('-')
                        ) {
                            line.push(' ');
                        }
                    }
                    if let Some(MathElement::Command(_)) = self.children.get(i + 1) {
                        line.push(' ');
                    }
                }
                _ => {}
            }
        }
        if !line.trim().is_empty() {
            output.push(line.trim_end().to_string());
        }
        output
            .iter()
            .map(|line| format!("{}{}", tex::indent_str(indent_level, tabstop), line))
            .collect()
    }

    pub fn len(&self) -> usize {
        todo!()
    }
}

pub struct MathBrackGroup {
    children: Vec<MathElement>,
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

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = Vec::new();
        for child in self.children.iter() {
            output.extend(child.format(indent_level, tabstop, line_length));
        }
        vec![format!("[{}]", output.join(", "))]
    }
}

pub struct MathCurlyGroup {
    body: MathParent,
}

impl MathCurlyGroup {
    fn from(node: &SyntaxNode) -> Self {
        Self {
            body: MathParent::from(node),
        }
    }

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = Vec::new();
        let bodytext = self.body.format(indent_level, tabstop, line_length);
        for line in bodytext {
            output.push(line.trim().to_string());
        }
        vec![
            "{".to_string(),
            output.join(" ").to_string(),
            "}".to_string(),
        ]
    }
}

pub struct MathMixedGroup {
    children: Vec<MathElement>,
    open_delim: String,
    close_delim: String,
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

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = String::new();
        output.push_str(&self.open_delim);
        for child in self.children.iter() {
            output.push_str(
                child
                    .format(indent_level, tabstop, line_length)
                    .join("")
                    .trim(),
            );
        }
        output.push_str(&self.close_delim);
        vec![output]
    }
}

pub struct MathCommand {
    name: String,
    args: Vec<MathElement>,
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

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = String::new();
        output.push_str(&self.name);
        for arg in self.args.iter() {
            output.push_str(
                arg.format(indent_level, tabstop, line_length)
                    .join("")
                    .trim(),
            );
        }
        if self.name.as_str() != "\\\\" {
            output.push(' ');
        }
        vec![output]
    }
}

pub struct MathEnvironment {
    name: String,
    args: Vec<MathElement>,
    body: MathParent,
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
            .filter(|child| child.kind() == SyntaxKind::BRACK_GROUP)
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

    pub fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut line = String::new();
        let mut lines = Vec::new();
        let indent = tex::indent_str(indent_level, tabstop);

        line.push_str(&format!("{}\\begin{{{}}}", indent, self.name));

        for arg in self.args.iter() {
            line.push_str(
                &arg.format(indent_level, tabstop, line_length)
                    .join("")
            );
        }

        lines.push(line);

        lines.extend(self.align_at_amp(indent_level + 1, tabstop, line_length));

        lines.push(format!("{}\\end{{{}}}", indent, self.name));

        lines
    }

    fn align_at_amp(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
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

        let inner_indent = tex::indent_str(indent_level, tabstop);

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

    pub fn len(&self) -> usize {
        todo!()
    }
}
