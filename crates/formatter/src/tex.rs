#![allow(unused_variables, dead_code)]
use syntax::latex::{SyntaxElement, SyntaxKind, SyntaxNode};

use crate::math::{MathEnvironment, MathParent};

pub fn get_blankline(node: &SyntaxNode) -> Option<TexElement> {
    if let Some(token) = node.last_token() {
        if token.to_string().contains("\n\n") || token.to_string().contains("\r\n\r\n") {
            Some(TexElement::BlankLine)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn indent_str(indent_level: usize, tabstop: usize) -> String {
    " ".repeat(indent_level * tabstop)
}

pub struct LineBreak {
    line: usize,
    character: usize,
}

pub enum TexElement {
    Parent(TexParent),
    Command(TexCommand),
    Environment(TexEnvironmentParent),
    Text(String),
    KeyValPair(TexKeyVal),
    CurlyGroup(TexCurlyGroup),
    MixedGroup(TexMixedGroup),
    CurlyGroupWordList(TexCurlyGroupWordList),
    BrackGroup(TexBrackGroup),
    BrackGroupWord(TexBrackGroupWord),
    BrackGroupKeyVal(TexBrackGroupKeyVal),
    EnumItem(TexEnumItem),
    Formula(TexFormula),
    Section(TexSection),
    BlankLine,
    Comment(TexComment),
    BeginEnvironment(TexBeginEnvironment),
    EndEnvironment(TexEndEnvironment),
}

impl TexElement {
    pub fn from(node: &SyntaxNode) -> TexElement {
        match node.kind() {
            SyntaxKind::ROOT | SyntaxKind::PREAMBLE => TexElement::Parent(TexParent::from(node)),
            SyntaxKind::COMMENT => TexElement::Comment(TexComment::from(node)),
            SyntaxKind::SECTION
            | SyntaxKind::SUBSECTION
            | SyntaxKind::SUBSUBSECTION
            | SyntaxKind::PARAGRAPH
            | SyntaxKind::CHAPTER
            | SyntaxKind::PART => TexElement::Section(TexSection::from(node)),
            SyntaxKind::GENERIC_COMMAND
            | SyntaxKind::PACKAGE_INCLUDE
            | SyntaxKind::CLASS_INCLUDE
            | SyntaxKind::IMPORT
            | SyntaxKind::CAPTION
            | SyntaxKind::LATEX_INCLUDE
            | SyntaxKind::MATH_OPERATOR
            | SyntaxKind::GRAPHICS_INCLUDE
            | SyntaxKind::TIKZ_LIBRARY_IMPORT
            | SyntaxKind::CITATION
            | SyntaxKind::COLOR_DEFINITION
            | SyntaxKind::COLOR_REFERENCE
            | SyntaxKind::LABEL_DEFINITION
            | SyntaxKind::LABEL_REFERENCE
            | SyntaxKind::LABEL_REFERENCE_RANGE
            | SyntaxKind::INKSCAPE_INCLUDE
            | SyntaxKind::BIBTEX_INCLUDE
            | SyntaxKind::BIBLATEX_INCLUDE
            | SyntaxKind::NEW_COMMAND_DEFINITION
            | SyntaxKind::GLOSSARY_ENTRY_REFERENCE
            | SyntaxKind::GLOSSARY_ENTRY_DEFINITION => TexElement::Command(TexCommand::from(node)),
            SyntaxKind::ENVIRONMENT => TexElement::Environment(TexEnvironmentParent::from(node)),
            SyntaxKind::HREF | SyntaxKind::TEXT | SyntaxKind::KEY | SyntaxKind::ERROR => {
                let text = node.to_string();
                let words: Vec<String> = text.split("\n\n").map(|word| word.replace("\n", " ")).collect();
                if words.len() > 1 {
                    let mut children = Vec::new();
                    for word in words {
                        children.push(TexElement::Text(format!("{} ", word)));
                        children.push(TexElement::BlankLine);
                    }
                    children.pop();
                    TexElement::Parent(TexParent { children })
                } else {
                    TexElement::Text(node.to_string().replace("\n", " "))
                }
            }
            SyntaxKind::CURLY_GROUP => TexElement::CurlyGroup(TexCurlyGroup::from(node)),
            SyntaxKind::BRACK_GROUP_KEY_VALUE => {
                TexElement::BrackGroupKeyVal(TexBrackGroupKeyVal::from(node))
            }
            SyntaxKind::BRACK_GROUP_WORD => {
                TexElement::BrackGroupWord(TexBrackGroupWord::from(node))
            }
            SyntaxKind::BRACK_GROUP => TexElement::BrackGroup(TexBrackGroup::from(node)),
            SyntaxKind::CURLY_GROUP_WORD | SyntaxKind::CURLY_GROUP_WORD_LIST => {
                TexElement::CurlyGroupWordList(TexCurlyGroupWordList::from(node))
            }
            SyntaxKind::KEY_VALUE_PAIR => TexElement::KeyValPair(TexKeyVal::from(node)),
            SyntaxKind::MIXED_GROUP => TexElement::MixedGroup(TexMixedGroup::from(node)),
            SyntaxKind::ENUM_ITEM => TexElement::EnumItem(TexEnumItem::from(node)),
            SyntaxKind::EQUATION | SyntaxKind::FORMULA => {
                TexElement::Formula(TexFormula::from(node))
            }
            SyntaxKind::BEGIN => TexElement::BeginEnvironment(TexBeginEnvironment::from(node)),
            SyntaxKind::END => TexElement::EndEnvironment(TexEndEnvironment::from(node)),
            _ => TexElement::Text(String::new()),
        }
    }

    pub fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        match self {
            TexElement::Command(cmd) => cmd.format(indent_level, tabstop, line_length, max_length),
            TexElement::Environment(env) => {
                env.format(indent_level, tabstop, line_length, max_length)
            }
            TexElement::Text(text) => vec![text.to_string()],
            TexElement::Parent(p) => p.format(indent_level, tabstop, line_length, max_length),
            TexElement::KeyValPair(kvp) => {
                kvp.format(indent_level, tabstop, line_length, max_length)
            }
            TexElement::CurlyGroup(group) => {
                group.format(indent_level, tabstop, line_length, max_length)
            }
            TexElement::BrackGroup(group) => {
                group.format(indent_level, tabstop, line_length, max_length)
            }
            TexElement::BrackGroupWord(group) => {
                group.format(indent_level, tabstop, line_length, max_length)
            }
            TexElement::BrackGroupKeyVal(group) => {
                group.format(indent_level, tabstop, line_length, max_length)
            }
            TexElement::CurlyGroupWordList(group) => {
                group.format(indent_level, tabstop, line_length, max_length)
            }
            TexElement::MixedGroup(group) => {
                group.format(indent_level, tabstop, line_length, max_length)
            }
            TexElement::EnumItem(item) => {
                item.format(indent_level, tabstop, line_length, max_length)
            }
            TexElement::Formula(formula) => {
                formula.format(indent_level, tabstop, line_length, max_length)
            }
            TexElement::Section(section) => {
                section.format(indent_level, tabstop, line_length, max_length)
            }
            TexElement::BlankLine => vec![String::new()],
            TexElement::Comment(text) => vec![text.format()],
            TexElement::BeginEnvironment(begin) => {
                begin.format(indent_level, tabstop, line_length, max_length)
            }
            TexElement::EndEnvironment(end) => {
                end.format(indent_level, tabstop, line_length, max_length)
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            TexElement::Command(cmd) => cmd.len(),
            TexElement::Environment(env) => env.len(),
            TexElement::Text(text) => text.len(),
            TexElement::Parent(p) => p.len(),
            TexElement::KeyValPair(kvp) => kvp.len(),
            TexElement::CurlyGroup(group) => group.len(),
            TexElement::BrackGroup(group) => group.len(),
            TexElement::BrackGroupWord(group) => group.len(),
            TexElement::BrackGroupKeyVal(group) => group.len(),
            TexElement::CurlyGroupWordList(group) => group.len(),
            TexElement::MixedGroup(group) => group.len(),
            TexElement::EnumItem(item) => item.len(),
            TexElement::Formula(formula) => formula.len(),
            TexElement::Section(section) => section.len(),
            TexElement::BlankLine => 0,
            TexElement::Comment(text) => text.len(),
            TexElement::BeginEnvironment(begin) => begin.len(),
            TexElement::EndEnvironment(end) => end.len(),
        }
    }

    pub fn line_breaks(&self) -> Vec<LineBreak> {
        todo!()
    }
}

pub struct TexParent {
    children: Vec<TexElement>,
}

impl TexParent {
    fn from(node: &SyntaxNode) -> Self {
        let mut children = Vec::new();
        for child in node.children() {
            children.push(TexElement::from(&child));
            if let Some(element) = get_blankline(&child) {
                children.push(element);
            }
        }
        Self { children }
    }

    fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();
        let indent = indent_str(indent_level, tabstop);
        let mut line = indent.clone();
        for (i, child) in self.children.iter().enumerate() {
            if line.len() >= line_length {
                output.push(line);
                line = String::new();
            }
            match child {
                TexElement::Command(cmd) => {
                    let cmdln = cmd.len();
                    if line.len() + cmdln >= line_length - 1 && !line.trim().is_empty() {
                        line = line.trim_end().to_string();
                        if !line.trim().ends_with('%') {
                            line.push('%');
                        }
                        output.push(line.trim_end().to_string());
                        line = indent.clone();
                    }
                    if matches!(
                        cmd.name.as_str(),
                        "\\newline"
                            | "\\newpage"
                            | "\\usepackage"
                            | "\\documentclass"
                            | "\\setlength"
                            | "\\pagestyle"
                            | "\\newcommand"
                            | "\\renewcommand"
                            | "\\author"
                            | "\\title"
                            | "\\maketitle"
                            | "\\date"
                            | "\\institute"
                            | "\\usetheme"
                            | "\\usecolortheme"
                    ) {
                        if !line.trim().is_empty() {
                            output.push(line.trim_end().to_string());
                            line = String::new();
                        }
                        output.extend(
                            cmd.format(indent_level, tabstop, line_length, max_length)
                                .iter()
                                .map(|string| string.trim_end().to_string()),
                        );
                    } else if cmd.name.as_str() == "\\\\" {
                        line.push_str("\\\\");
                        output.push(line.trim().to_string());
                        line = String::new();
                    } else {
                        let fmt =
                            cmd.format(indent_level, tabstop, line_length - line.len(), max_length);
                        if fmt.len() == 1 && line.len() + fmt[0].len() < line_length {
                            line.push_str(fmt[0].clone().trim());
                            if let Some(TexElement::Text(txt)) = self.children.get(i + 1) {
                                if !txt.trim().is_empty()
                                    && !matches!(
                                        txt.chars().nth(0),
                                        Some('}')
                                            | Some(')')
                                            | Some(']')
                                            | Some('.')
                                            | Some(',')
                                            | Some(';')
                                            | Some('~')
                                    )
                                {
                                    if !line.ends_with('}') {
                                        line.push('~');
                                    } else {
                                        line.push(' ');
                                    }
                                }
                            }
                            if let Some(TexElement::Formula(_)) = self.children.get(i + 1) {
                                line.push(' ');
                            }
                        } else {
                            let fmt = cmd.format(
                                indent_level,
                                tabstop,
                                line_length - line.len(),
                                max_length,
                            );
                            if fmt.len() == 1 {
                                line.push_str(fmt[0].clone().trim());
                            } else {
                                line.push_str(fmt[0].clone().trim());
                                output.push(line);
                                line = String::new();
                                output.extend(
                                    fmt[1..]
                                        .iter()
                                        .cloned()
                                        .map(|line| format!("{indent}{line}")),
                                );
                            }
                        }
                    }
                }

                TexElement::Text(text) if !text.trim().is_empty() => {
                    if line_length == 0 {
                        line.push_str(text);
                        continue;
                    }
                    if text.trim().len() + line.len() <= line_length {
                        line.push_str(&format!("{} ", text.trim()));
                        if let Some(TexElement::Command(cmd)) = self.children.get(i + 1) {
                            if line.ends_with("~ ") {
                                line = line.trim().to_string();
                            } else if cmd.args.is_empty() {
                                line.pop();
                            }
                        }
                        continue;
                    }
                    let words = text.split(" ").filter(|word| !word.is_empty() && *word != " ").collect::<Vec<&str>>();
                    for word in words {
                        if line.len() + word.trim_end().len() > line_length {
                            output.push(line.trim_end().to_string());
                            line = indent.clone();
                        }
                        line.push_str(&format!("{word} "));
                    }
                }

                TexElement::Environment(_)
                | TexElement::BeginEnvironment(_)
                | TexElement::EndEnvironment(_)
                | TexElement::BlankLine
                | TexElement::Section(_)
                | TexElement::EnumItem(_) => {
                    if !line.trim().is_empty() {
                        output.push(line);
                        line = indent.clone();
                    }
                    output.extend(
                        child
                            .format(indent_level, tabstop, line_length, max_length)
                            .iter()
                            .map(|line| line.trim_end().to_string()),
                    );
                }

                TexElement::Parent(p) => {
                    let fmt = p.format(indent_level, tabstop, line_length, max_length);
                    line.push_str(&format!("{}{}", indent, fmt[0]));
                    output.push(line.clone());
                    if fmt.len() > 1 {
                        line = format!("{}{}", indent, fmt[fmt.len() - 1].clone());
                        output.extend(
                            fmt[1..fmt.len() - 1]
                                .iter()
                                .cloned()
                                .map(|line| format!("{indent}{line}")),
                        );
                    } else {
                        line = indent.clone();
                    }
                }

                TexElement::CurlyGroup(_)
                | TexElement::BrackGroup(_)
                | TexElement::BrackGroupKeyVal(_)
                | TexElement::BrackGroupWord(_)
                | TexElement::CurlyGroupWordList(_)
                | TexElement::MixedGroup(_) => {
                    let fmt =
                        child.format(indent_level, tabstop, line_length - line.len(), max_length);
                    if line.trim().len() + fmt[0].len() > line_length {
                        output.push(line.to_string());
                        line = indent.clone();
                        line.push_str(&fmt[0]);
                    } else {
                        line.push_str(&fmt[0]);
                    }
                    if fmt.len() > 1 {
                        output.push(line);
                        output.extend(fmt[1..fmt.len() - 1].iter().cloned());
                        line = fmt[fmt.len() - 1].clone();
                        if line.is_empty() {
                            output.push(line);
                            line = indent.clone();
                        }
                        continue;
                    }
                    if let Some(TexElement::Text(txt)) = self.children.get(i + 1) {
                        if !matches!(
                            txt.chars().nth(0),
                            Some('.') | Some(',') | Some('^') | Some('_') | Some('-')
                        ) {
                            line.push(' ');
                        }
                    }
                }

                TexElement::Formula(formula) => {
                    if formula.inline {
                        let fmt = formula.format(indent_level, tabstop, line_length, max_length);
                        if fmt.join("").len() + line.len() > line_length {
                            output.push(line);
                            line = String::new();
                            if fmt.len() == 1 {
                                line.push_str(fmt.join("").trim());
                                if let Some(TexElement::Text(txt)) = self.children.get(i + 1) {
                                    if !matches!(
                                        txt.chars().nth(0),
                                        Some('.') | Some(',') | Some('?') | Some(';') | Some('-')
                                    ) {
                                        line.push(' ');
                                    }
                                }
                                if let Some(TexElement::Formula(_)) = self.children.get(i + 1) {
                                    line.push(' ');
                                }
                            } else {
                                output.extend(fmt);
                            }
                        } else {
                            line.push_str(fmt[0].trim_end());
                            if fmt.len() > 1 {
                                output.push(line);
                                output.extend(fmt[1..fmt.len() - 1].iter().cloned());
                                line = fmt[fmt.len() - 1].clone();
                                if line.is_empty() {
                                    output.push(line);
                                    line = indent.clone();
                                }
                            }
                            if let Some(TexElement::Text(txt)) = self.children.get(i + 1) {
                                if !matches!(
                                    txt.chars().nth(0),
                                    Some('.') | Some(',') | Some('?') | Some(';') | Some('-')
                                ) {
                                    line.push(' ');
                                }
                            }
                            if let Some(TexElement::Formula(_)) = self.children.get(i + 1) {
                                line.push(' ');
                            }
                        }
                    } else {
                        if !line.trim().is_empty() {
                            output.push(line.trim_end().to_string());
                            line = indent.clone();
                        }
                        output.extend(formula.format(
                            indent_level,
                            tabstop,
                            line_length,
                            max_length,
                        ));
                    }
                }

                TexElement::Comment(comment) => {
                    line.push_str(&comment.format());
                    output.push(line);
                    line = indent.clone();
                }

                TexElement::KeyValPair(_) => {} // KeyValPair should only be children of delimeter groups
                _ => {}
            }
        }

        if line.trim().is_empty() {
            return output;
        }
        output.push(line.to_string());
        output
    }

    fn len(&self) -> usize {
        self.children.iter().map(|child| child.len()).sum()
    }
}

pub struct TexCommand {
    name: String,
    args: Vec<TexElement>,
}

impl TexCommand {
    pub fn from(node: &SyntaxNode) -> Self {
        let name = node.first_token().unwrap().to_string();
        let mut args = Vec::new();

        for child in node.children() {
            match child.kind() {
                SyntaxKind::CURLY_GROUP_COMMAND => {
                    args.push(TexElement::CurlyGroupWordList(TexCurlyGroupWordList {
                        children: child
                            .children_with_tokens()
                            .filter(|t| matches!(t.kind(), SyntaxKind::COMMAND_NAME))
                            .map(|c| TexElement::Text(c.to_string().trim().to_string()))
                            .collect(),
                    }));
                }
                SyntaxKind::CURLY_GROUP => {
                    args.push(TexElement::CurlyGroup(TexCurlyGroup::from(&child)));
                }
                SyntaxKind::MIXED_GROUP => {
                    args.push(TexElement::MixedGroup(TexMixedGroup::from(&child)));
                }
                SyntaxKind::CURLY_GROUP_WORD | SyntaxKind::CURLY_GROUP_WORD_LIST => {
                    args.push(TexElement::CurlyGroupWordList(TexCurlyGroupWordList::from(
                        &child,
                    )));
                }
                SyntaxKind::BRACK_GROUP_KEY_VALUE => {
                    args.push(TexElement::BrackGroupKeyVal(TexBrackGroupKeyVal::from(
                        &child,
                    )));
                }
                SyntaxKind::BRACK_GROUP_WORD => {
                    args.push(TexElement::BrackGroupWord(TexBrackGroupWord::from(&child)));
                }
                _ => {}
            }
        }

        Self { name, args }
    }

    fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let mut output = Vec::new();
        let mut fmt = String::new();
        fmt.push_str(&self.name);
        for arg in self.args.iter() {
            if line_length == 0 {
                fmt.push_str(
                    arg.format(indent_level, tabstop, line_length, max_length)
                        .join("")
                        .trim_end(),
                );
                return vec![fmt];
            }
            let ln = arg.format(indent_level, tabstop, line_length - fmt.len(), max_length);
            if ln.first().unwrap().len() + fmt.len() > line_length {
                output.push(fmt.trim_end().to_string());
                fmt = String::new();
            }
            if ln.first().unwrap().trim().is_empty() {
                output.push(fmt.clone());
                output.push(String::new());
                fmt = String::new();
            } else {
                fmt.push_str(ln.first().unwrap().trim_end());
            }
            if ln.len() > 1 {
                if !fmt.trim().is_empty() {
                    output.push(fmt);
                }
                output.extend(ln[1..ln.len() - 1].iter().cloned());
                fmt = ln[ln.len() - 1].clone().trim_end().to_string();
                if fmt.trim().is_empty() {
                    output.push(String::new());
                }
            }
        }
        if !fmt.is_empty() {
            output.push(fmt);
        }
        output
    }

    pub fn len(&self) -> usize {
        let name_len = self.name.len();
        let arg_len = self.args.iter().map(|arg| arg.len()).sum::<usize>();
        name_len + arg_len
    }
}

pub struct TexCurlyGroup {
    body: TexParent,
}

impl TexCurlyGroup {
    fn from(node: &SyntaxNode) -> Self {
        Self {
            body: TexParent::from(node),
        }
    }

    fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let mut output = Vec::new();
        if line_length == 0 {
            let bodytext = self
                .body
                .format(indent_level, tabstop, line_length, max_length);
            output.push(format!("{{{}}}", bodytext.join("").trim()));
            return output;
        }

        let mut line = "{".to_string();
        if line.len() + self.body.len() > line_length {
            output.push(line);
        } else {
            let bodytext = self
                .body
                .format(indent_level, tabstop, line_length, max_length);
            line.push_str(bodytext.join("").trim());
            line.push('}');
            output.push(line);
            return output;
        }
        let bodytext = self.body.format(
            0,
            tabstop,
            max_length - 1 - (indent_level + 1) * tabstop,
            max_length,
        );
        for l in bodytext {
            let mut line = indent_str(indent_level + 1, tabstop);
            line.push_str(l.trim_end());
            if l.ends_with('%') {
                output.push(line.to_string());
            } else {
                line.push('%');
                output.push(line);
            }
        }
        output.push("}".to_string());

        output
    }

    fn len(&self) -> usize {
        2 + self.body.len()
    }
}

pub struct TexBrackGroup {
    children: Vec<TexElement>,
}

impl TexBrackGroup {
    fn from(node: &SyntaxNode) -> Self {
        let mut children = Vec::new();
        for child in node.children_with_tokens() {
            match child {
                SyntaxElement::Node(node) => {
                    if matches!(node.kind(), SyntaxKind::TEXT) {
                        let words = node.to_string();
                        let words = words.split(',').map(|word| word.trim());
                        for word in words {
                            children.push(TexElement::Text(word.to_string()))
                        }
                    } else {
                        children.push(TexElement::from(&node));
                    }
                }
                SyntaxElement::Token(token) => {
                    if token.kind() == SyntaxKind::EQUALITY_SIGN {
                        children.push(TexElement::Text(token.to_string()));
                    }
                }
            }
        }
        Self { children }
    }

    fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();

        for child in self.children.iter() {
            let fmt = child.format(indent_level, tabstop, line_length, max_length);
            if !matches!(fmt.join("").as_str(), "=") {
                if let Some(last) = output.last() {
                    if last.ends_with("=") {
                        let mut last = output.pop().unwrap();
                        last.push_str(fmt.join("").as_str());
                        output.push(last.to_string());
                    } else {
                        output.extend(fmt);
                    }
                } else {
                    output.extend(fmt);
                }
            } else if !output.is_empty() {
                let mut last = output.pop().unwrap();
                last.push('=');
                output.push(last.to_string());
            }
        }
        output = vec![format!("[{}]", output.join(", "))];
        output
    }

    fn len(&self) -> usize {
        self.children
            .iter()
            .map(|child| child.len() + 2)
            .sum::<usize>()
    }
}

pub struct TexBrackGroupKeyVal {
    children: Vec<TexElement>,
}

impl TexBrackGroupKeyVal {
    fn from(node: &SyntaxNode) -> Self {
        let mut children = Vec::new();
        for child in node.children() {
            if child.kind() == SyntaxKind::KEY_VALUE_BODY {
                children.extend(child.children().map(|c| TexElement::from(&c)));
            }
        }
        Self { children }
    }

    fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let mut output = Vec::new();
        for child in self.children.iter() {
            output.extend(child.format(indent_level, tabstop, line_length, max_length));
        }
        output = vec![format!("[{}]", output.join(", "))];
        output
    }

    fn len(&self) -> usize {
        2 + self.children.iter().map(|child| child.len()).sum::<usize>()
    }
}

pub struct TexBrackGroupWord {
    children: Vec<TexElement>,
}

impl TexBrackGroupWord {
    fn from(node: &SyntaxNode) -> Self {
        let mut children = Vec::new();
        for child in node.children() {
            match node.kind() {
                SyntaxKind::KEY => {
                    children.push(TexElement::Text(child.to_string()));
                }
                _ => {
                    children.push(TexElement::from(&child));
                }
            }
        }
        Self { children }
    }

    fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let mut output = Vec::new();
        for child in self.children.iter() {
            output.extend(child.format(indent_level, tabstop, line_length, max_length));
        }
        output = vec![format!("[{}]", output.join(", "))];
        output
    }

    fn len(&self) -> usize {
        self.children
            .iter()
            .map(|child| child.len() + 2)
            .sum::<usize>()
    }
}

pub struct TexMixedGroup {
    children: Vec<TexElement>,
    open_delim: String,
    close_delim: String,
}

impl TexMixedGroup {
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
                .map(|child| TexElement::from(&child))
                .collect(),
            open_delim,
            close_delim,
        }
    }

    fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let mut line = String::new();
        line.push_str(&self.open_delim);
        for child in self.children.iter() {
            line.push_str(
                child
                    .format(indent_level, tabstop, line_length, max_length)
                    .join("")
                    .split(",")
                    .collect::<Vec<&str>>()
                    .join(", ")
                    .to_string()
                    .trim(),
            );
        }
        line.push_str(&self.close_delim);
        let output = vec![line];
        output
    }

    fn len(&self) -> usize {
        2 + self.children.iter().map(|child| child.len()).sum::<usize>()
    }
}

pub struct TexCurlyGroupWordList {
    children: Vec<TexElement>,
}

impl TexCurlyGroupWordList {
    fn from(node: &SyntaxNode) -> Self {
        Self {
            children: node
                .children()
                .map(|child| TexElement::from(&child))
                .collect(),
        }
    }

    fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let mut output = Vec::new();
        for child in self.children.iter() {
            output.extend(child.format(indent_level, tabstop, line_length, max_length));
        }
        output = vec![format!("{{{}}}", output.join(", ").trim().to_string())];
        output
    }

    fn len(&self) -> usize {
        2 + self.children.iter().map(|child| child.len()).sum::<usize>()
    }
}

pub enum TexEnvironmentParent {
    Tex(TexEnvironment),
    Math(MathEnvironment),
}

impl TexEnvironmentParent {
    fn from(node: &SyntaxNode) -> Self {
        let name = node
            .first_child()
            .unwrap()
            .first_child()
            .unwrap()
            .first_child()
            .unwrap()
            .to_string();

        match name.as_str() {
            "math" | "displaymath" | "displaymath*" | "equation" | "equation*" | "align"
            | "align*" | "alignat" | "alignat*" | "gather" | "gather*" | "multline"
            | "multline*" | "flalign" | "flalign*" | "xalignat" | "xalignat*" | "xxalignat"
            | "xxalignat*" | "array" | "cases" | "cases*" | "split" | "IEEEeqnarray"
            | "IEEEeqnarray*" | "dcases" | "rcases" | "tcases" | "empheq" | "dmath"
            | "eqnarray" => TexEnvironmentParent::Math(MathEnvironment::from(node)),
            _ => TexEnvironmentParent::Tex(TexEnvironment::from(node)),
        }
    }

    fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        match self {
            TexEnvironmentParent::Tex(tex) => {
                tex.format(indent_level, tabstop, line_length, max_length)
            }
            TexEnvironmentParent::Math(math) => {
                math.format(indent_level, tabstop, line_length, max_length)
            }
        }
    }

    fn len(&self) -> usize {
        match self {
            TexEnvironmentParent::Tex(tex) => tex.len(),
            TexEnvironmentParent::Math(math) => math.len(),
        }
    }
}

pub struct TexBeginEnvironment {
    name: String,
    args: Vec<TexElement>,
}

impl TexBeginEnvironment {
    pub fn from(node: &SyntaxNode) -> Self {
        let name = node
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::KEY)
            .collect::<Vec<SyntaxNode>>()
            .first()
            .unwrap()
            .to_string();
        let args: Vec<TexElement> = node
            .children()
            .filter(|child| {
                matches!(
                    child.kind(),
                    SyntaxKind::CURLY_GROUP
                        | SyntaxKind::CURLY_GROUP_WORD
                        | SyntaxKind::CURLY_GROUP_COMMAND
                        | SyntaxKind::CURLY_GROUP_WORD_LIST
                        | SyntaxKind::CURLY_GROUP_KEY_VALUE
                        | SyntaxKind::BRACK_GROUP
                        | SyntaxKind::BRACK_GROUP_WORD
                        | SyntaxKind::BRACK_GROUP_KEY_VALUE
                        | SyntaxKind::MIXED_GROUP
                )
            })
            .map(|node: SyntaxNode| TexElement::from(&node))
            .collect();
        Self { name, args }
    }

    pub fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let mut line = String::new();
        let mut output = Vec::new();
        let mut indent = 0;
        if indent_level > 0 {
            indent = indent_level - 1;
        }
        line.push_str(&format!("{}\\begin", indent_str(indent, tabstop)));
        for arg in &self.args {
            let fmt = arg.format(indent_level, tabstop, line_length, max_length);
            line.push_str(&fmt[0]);
        }
        if !line.trim().is_empty() {
            output.push(line);
        }
        output
    }

    fn len(&self) -> usize {
        2 + self.name.len() + self.args.iter().map(|arg| arg.len()).sum::<usize>()
    }
}

pub struct TexEndEnvironment {
    name: String,
}

impl TexEndEnvironment {
    pub fn from(node: &SyntaxNode) -> Self {
        let name = node
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::KEY)
            .collect::<Vec<SyntaxNode>>()
            .first()
            .unwrap()
            .to_string();
        Self { name }
    }

    pub fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let mut line = String::new();
        let mut output = Vec::new();
        let mut indent = 0;
        if indent_level > 0 {
            indent = indent_level - 1;
        }
        line.push_str(&format!(
            "{}\\end{{{}}}",
            indent_str(indent, tabstop),
            self.name
        ));
        if !line.trim().is_empty() {
            output.push(line);
        }
        output
    }

    fn len(&self) -> usize {
        2 + self.name.len()
    }
}

pub struct TexEnvironment {
    body: TexParent,
}

impl TexEnvironment {
    pub fn from(node: &SyntaxNode) -> Self {
        let body = TexParent::from(node);
        Self { body }
    }

    fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let mut subindent = 1;
        if let Some(TexElement::BeginEnvironment(child)) = self.body.children.first() {
            if matches!(child.name.as_str(), "document" | "verbatim") {
                subindent = 0;
            }
        }

        self.body
            .format(indent_level + subindent, tabstop, line_length, max_length)
    }

    fn len(&self) -> usize {
        self.body.len()
    }
}

pub struct TexKeyVal {
    key: String,
    value: Option<String>,
}

impl TexKeyVal {
    fn from(node: &SyntaxNode) -> Self {
        let keys: Vec<String> = node
            .children()
            .filter(|c| c.kind() == SyntaxKind::KEY)
            .map(|k| k.to_string())
            .collect();
        let vals: Vec<String> = node
            .children()
            .filter(|c| c.kind() == SyntaxKind::VALUE)
            .map(|k| k.to_string())
            .collect();
        let key = keys.join("").trim().to_string();
        let value = match vals.is_empty() {
            true => None,
            false => Some(vals.join("").trim().to_string()),
        };
        Self { key, value }
    }

    fn format(
        &self,
        _indent_level: usize,
        _tabstop: usize,
        _line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        match &self.value {
            Some(val) => vec![format!("{}={val}", self.key)],
            None => vec![format!("{}", self.key)],
        }
    }

    fn len(&self) -> usize {
        if let Some(val) = &self.value {
            self.key.len() + 1 + val.len()
        } else {
            self.key.len()
        }
    }
}

pub struct TexEnumItem {
    body: TexParent,
}

impl TexEnumItem {
    fn from(node: &SyntaxNode) -> Self {
        Self {
            body: TexParent::from(node),
        }
    }

    fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let output = self
            .body
            .format(indent_level, tabstop, line_length - 6, max_length);
        let mut space = String::new();
        let indent = indent_str(indent_level, tabstop);
        if output.first().unwrap().chars().nth(0) != Some('[') {
            space.push(' ');
        }
        let mut fmt = vec![format!(
            "{}\\item{}{}",
            indent,
            space,
            output.first().unwrap().trim()
        )];
        let indent = indent_str(indent_level + 1, tabstop);
        fmt.extend(
            output[1..]
                .iter()
                .cloned()
                .map(|line| format!("{indent}{line}")),
        );
        fmt
    }

    fn len(&self) -> usize {
        5 + self.body.len()
    }
}

pub struct TexFormula {
    inline: bool,
    body: MathParent,
}

impl TexFormula {
    fn from(node: &SyntaxNode) -> Self {
        let inline = matches!(node.first_token().unwrap().text(), "$" | "\\(");
        let body = MathParent::from(node);
        Self { inline, body }
    }

    fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let mut output = Vec::new();
        let indent = indent_str(indent_level, tabstop);
        if self.inline {
            output.push("\\( ".to_string());
        } else {
            output.push(format!("{indent}\\["));
        }

        if self.inline {
            let fmt = self
                .body
                .format(indent_level, tabstop, line_length, max_length);
            if fmt.join("").trim().len() + 6 <= line_length {
                output.push(fmt.join("").trim().to_string());
                output.push(" \\)".to_string());
                output = vec![output.join("").trim_end().to_string()];
                output
            } else {
                let fmt = self
                    .body
                    .format(indent_level, tabstop, line_length, max_length);
                output.extend(fmt);
                output.push("\\)".to_string());
                output
            }
        } else {
            let fmt = self
                .body
                .format(indent_level + 1, tabstop, line_length, max_length);
            output.extend(fmt);
            output.push(format!("{indent}\\]"));
            output
        }
    }

    fn len(&self) -> usize {
        6 + self.body.len()
    }
}

pub struct TexSection {
    kind: String,
    name: String,
    body: TexParent,
}

impl TexSection {
    fn from(node: &SyntaxNode) -> Self {
        let kind = node.first_token().unwrap().to_string();
        let name = node
            .first_child()
            .unwrap()
            .first_child()
            .unwrap()
            .to_string();
        let first_child = node.first_child().unwrap().clone();
        let children: Vec<SyntaxNode> = node
            .children()
            .filter(|child| *child != first_child)
            .collect();
        let body: TexParent = TexParent {
            children: children.iter().map(TexElement::from).collect(),
        };

        Self { kind, name, body }
    }

    fn format(
        &self,
        indent_level: usize,
        tabstop: usize,
        line_length: usize,
        max_length: usize,
    ) -> Vec<String> {
        let mut output = Vec::new();
        output.push(String::new());
        output.push(format!("{}{{{}}}", self.kind, self.name));
        output.push(String::new());
        output.extend(
            self.body
                .format(indent_level, tabstop, line_length, max_length),
        );
        output
    }

    fn len(&self) -> usize {
        self.name.len() + self.body.len()
    }
}

pub struct TexComment {
    body: String,
}

impl TexComment {
    pub fn from(node: &SyntaxNode) -> Self {
        Self {
            body: node.to_string().trim().to_string(),
        }
    }

    fn format(&self) -> String {
        self.body.clone()
    }

    fn len(&self) -> usize {
        self.body.len()
    }
}
