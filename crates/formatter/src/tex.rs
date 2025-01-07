use syntax::latex::{SyntaxElement, SyntaxKind, SyntaxNode};

use crate::math::{MathEnvironment, MathParent};

pub fn indent_str(indent_level: usize, tabstop: usize) -> String {
    " ".repeat(indent_level * tabstop)
}

fn blanklines_and_comments(node: &SyntaxNode) -> Vec<TexElement> {
    let mut output = Vec::new();
    for desc in node.descendants_with_tokens() {
        if let Some(token) = desc.as_token() {
            if token.to_string() == "\n\n" {
                output.push(TexElement::BlankLine);
            }
            if token.kind() == SyntaxKind::COMMENT {
                output.push(TexElement::Comment(token.to_string()));
            }
        }
    }
    output
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
    EnumItem(TexEnumItem),
    Formula(TexFormula),
    Section(TexSection),
    BlankLine,
    Comment(String),
    BeginEnvironment(TexBeginEnvironment),
    EndEnvironment(TexEndEnvironment),
}

impl TexElement {
    pub fn from(node: &SyntaxNode) -> TexElement {
        match node.kind() {
            SyntaxKind::ROOT | SyntaxKind::PREAMBLE => TexElement::Parent(TexParent::from(node)),
            SyntaxKind::SECTION
            | SyntaxKind::SUBSECTION
            | SyntaxKind::SUBSUBSECTION
            | SyntaxKind::PARAGRAPH
            | SyntaxKind::CHAPTER
            | SyntaxKind::PART => TexElement::Section(TexSection::from(node)),
            SyntaxKind::GENERIC_COMMAND
            | SyntaxKind::PACKAGE_INCLUDE
            | SyntaxKind::CLASS_INCLUDE
            | SyntaxKind::HREF
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
            SyntaxKind::TEXT | SyntaxKind::KEY | SyntaxKind::ERROR => {
                let text = node.to_string();
                let words: Vec<&str> = text.split("\n\n").collect();
                if words.len() > 1 {
                    let mut children = Vec::new();
                    for word in words {
                        children.push(TexElement::Text(word.to_string()));
                        children.push(TexElement::BlankLine);
                    }
                    TexElement::Parent(TexParent { children })
                } else {
                    TexElement::Text(node.to_string().replace("\n", " "))
                }
            }
            SyntaxKind::CURLY_GROUP => TexElement::CurlyGroup(TexCurlyGroup::from(node)),
            SyntaxKind::BRACK_GROUP | SyntaxKind::BRACK_GROUP_KEY_VALUE => {
                TexElement::BrackGroup(TexBrackGroup::from(node))
            }
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

    pub fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        match self {
            TexElement::Command(cmd) => cmd.format(indent_level, tabstop, line_length),
            TexElement::Environment(env) => env.format(indent_level, tabstop, line_length),
            TexElement::Text(text) => vec![text.to_string().trim().to_string()],
            TexElement::Parent(p) => p.format(indent_level, tabstop, line_length),
            TexElement::KeyValPair(kvp) => kvp.format(indent_level, tabstop, line_length),
            TexElement::CurlyGroup(group) => group.format(indent_level, tabstop, line_length),
            TexElement::BrackGroup(group) => group.format(indent_level, tabstop, line_length),
            TexElement::CurlyGroupWordList(group) => {
                group.format(indent_level, tabstop, line_length)
            }
            TexElement::MixedGroup(group) => group.format(indent_level, tabstop, line_length),
            TexElement::EnumItem(item) => item.format(indent_level, tabstop, line_length),
            TexElement::Formula(formula) => formula.format(indent_level, tabstop, line_length),
            TexElement::Section(section) => section.format(indent_level, tabstop, line_length),
            TexElement::BlankLine => vec![String::new()],
            TexElement::Comment(text) => vec![text.to_string().trim().to_string()],
            TexElement::BeginEnvironment(begin) => begin.format(indent_level, tabstop, line_length),
            TexElement::EndEnvironment(end) => end.format(indent_level, tabstop, line_length),
        }
    }
}

pub struct TexParent {
    children: Vec<TexElement>,
}

impl TexParent {
    fn from(node: &SyntaxNode) -> Self {
        let children = node
            .children_with_tokens()
            .filter(|child| match child {
                SyntaxElement::Node(_) => true,
                SyntaxElement::Token(t) => {
                    matches!(t.kind(), SyntaxKind::HREF | SyntaxKind::COMMENT)
                        || t.to_string() == "\n\n"
                }
            })
            .map(|child| match child {
                SyntaxElement::Node(n) => TexElement::from(&n),
                SyntaxElement::Token(t) => TexElement::Text(t.to_string()),
            })
            .collect();
        Self { children }
    }

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();
        let indent = indent_str(indent_level, tabstop);
        let mut line = indent.clone();
        let mut l = line.len();
        for (i, child) in self.children.iter().enumerate() {
            if line.len() > line_length {
                output.push(line);
                line = String::new();
            }
            l = line.len();
            match child {
                TexElement::Command(cmd) => {
                    let cmdln = cmd.format(0, 0, 0).join("").len();
                    if line.len() + cmdln >= line_length && !line.trim().is_empty() {
                        line = line.trim_end().to_string();
                        if !line.ends_with('%') {
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
                            cmd.format(indent_level, tabstop, line_length)
                                .iter()
                                .map(|string| string.trim_end().to_string()),
                        );
                    } else if cmd.name.as_str() == "\\\\" {
                        line.push_str("\\\\");
                        output.push(line.trim().to_string());
                        line = String::new();
                        l = line.len();
                    } else {
                        let fmt = cmd.format(indent_level, tabstop, line_length - line.len());
                        if fmt.len() == 1 && line.len() + fmt[0].len() <= line_length {
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
                            let fmt = cmd.format(indent_level, tabstop, line_length - line.len());
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
                    if text.trim().len() + line.len() < line_length {
                        line.push_str(&format!("{} ", text.trim()));
                        if let Some(TexElement::Command(cmd)) = self.children.get(i + 1) {
                            if line.ends_with("~ ") {
                                line = line.trim().to_string();
                            } else if cmd.args.is_empty() {
                                line.pop();
                            }
                        }
                    } else {
                        let words = text.trim().split(" ");
                        for word in words {
                            if line.len() + word.trim_end().len() > line_length {
                                output.push(line.trim_end().to_string());
                                line = indent.clone();
                                l = line.len();
                            }
                            line.push_str(&format!("{word} "));
                        }
                    }
                }
                TexElement::Environment(env) => {
                    if !line.trim().is_empty() {
                        output.push(line);
                        line = indent.clone();
                        l = line.len();
                    }
                    output.extend(
                        env.format(indent_level, tabstop, line_length)
                            .iter()
                            .map(|line| line.trim_end().to_string()),
                    );
                }
                TexElement::BeginEnvironment(env) => {
                    if !line.trim().is_empty() {
                        output.push(line);
                        line = String::new();
                        l = line.len();
                    }
                    output.extend(env.format(indent_level, tabstop, line_length));
                }
                TexElement::EndEnvironment(env) => {
                    if !line.trim().is_empty() {
                        output.push(line);
                        line = indent.clone();
                        l = line.len();
                    }
                    output.extend(env.format(indent_level, tabstop, line_length));
                }
                TexElement::Parent(p) => {
                    let fmt = p.format(indent_level, tabstop, line_length);
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
                TexElement::KeyValPair(_) => {
                    // KeyValPair should only be children of delimeter groups
                }
                TexElement::CurlyGroup(group) => {
                    let fmt = group.format(indent_level, tabstop, line_length - line.len());
                    line.push_str(&fmt[0]);
                    output.push(line);
                    line = String::new();
                    output.extend(fmt[1..].iter().cloned());
                }
                TexElement::BrackGroup(group) => {
                    line.push_str(&group.format(indent_level, tabstop, line_length).join(""));
                }
                TexElement::CurlyGroupWordList(group) => {
                    line.push_str(&group.format(indent_level, tabstop, line_length).join(""));
                }
                TexElement::MixedGroup(group) => {
                    line.push_str(
                        group
                            .format(indent_level, tabstop, line_length)
                            .join("")
                            .trim_end(),
                    );
                    if let Some(TexElement::Text(txt)) = self.children.get(i + 1) {
                        if !matches!(
                            txt.chars().nth(0),
                            Some('.') | Some(',') | Some('^') | Some('_') | Some('-')
                        ) {
                            line.push(' ');
                        }
                    }
                }
                TexElement::EnumItem(item) => {
                    if !line.trim().is_empty() {
                        output.push(line.trim().to_string());
                    }
                    line = String::new();
                    l = line.len();
                    output.extend(item.format(indent_level, tabstop, line_length - line.len()));
                }
                TexElement::Formula(formula) => {
                    if formula.inline {
                        let fmt = formula.format(indent_level, tabstop, line_length);
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
                                if let Some(TexElement::Formula(formula)) = self.children.get(i + 1)
                                {
                                    line.push(' ');
                                }
                            } else {
                                output.extend(fmt);
                            }
                        } else {
                            line.push_str(fmt.join("").trim());
                            if let Some(TexElement::Text(txt)) = self.children.get(i + 1) {
                                if !matches!(
                                    txt.chars().nth(0),
                                    Some('.') | Some(',') | Some('?') | Some(';') | Some('-')
                                ) {
                                    line.push(' ');
                                }
                            }
                            if let Some(TexElement::Formula(formula)) = self.children.get(i + 1) {
                                line.push(' ');
                            }
                        }
                    } else {
                        if !line.trim().is_empty() {
                            output.push(line.trim_end().to_string());
                            line = indent.clone();
                            l = line.len();
                        }
                        output.extend(formula.format(indent_level, tabstop, line_length));
                    }
                }
                TexElement::Section(section) => {
                    if !line.trim().is_empty() {
                        output.push(line.trim_end().to_string());
                        line = String::new();
                    }
                    output.extend(section.format(indent_level, tabstop, line_length));
                }
                TexElement::BlankLine => {
                    if !line.trim().is_empty() {
                        output.push(line.trim().to_string());
                    }
                    line = String::new();
                    l = line.len();
                    output.push(String::new());
                }
                _ => {}
            }
        }

        if line.trim().is_empty() {
            return output;
        }

        if line.len() < line_length {
            output.push(line.trim_end().to_string());
        } else {
            output.push(line[0..l].to_string());
            output.push(format!(
                "{}{}",
                indent_str(0, tabstop),
                &line[l..].to_string()
            ));
        }
        output
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
                SyntaxKind::BRACK_GROUP_KEY_VALUE | SyntaxKind::BRACK_GROUP_WORD => {
                    args.push(TexElement::BrackGroup(TexBrackGroup::from(&child)));
                }
                _ => {}
            }
        }
        args.extend(blanklines_and_comments(node));

        Self { name, args }
    }

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = Vec::new();
        let mut fmt = String::new();
        fmt.push_str(&self.name);
        for arg in self.args.iter() {
            if line_length == 0 {
                fmt.push_str(
                    arg.format(indent_level, tabstop, line_length)
                        .join("")
                        .trim_end(),
                );
            } else {
                let ln = arg.format(indent_level, tabstop, line_length - self.name.len());
                if ln.first().unwrap().len() + fmt.len() + self.name.len() > line_length {
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
        }
        if !fmt.is_empty() {
            output.push(fmt);
        }
        output
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

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = Vec::new();
        if line_length == 0 {
            let bodytext = self.body.format(indent_level, tabstop, line_length);
            output.push(format!("{{{}}}", bodytext.join("").trim()));
            return output;
        }

        let bodytext = self.body.format(indent_level, tabstop, line_length);
        if bodytext.iter().map(|line| line.len()).sum::<usize>() <= line_length {
            output.push(format!("{{{}}}", bodytext.join("").trim()));
            return output;
        }

        let bodytext = self.body.format(indent_level + 1, tabstop, line_length);
        let l = bodytext.len();
        let indent = indent_str(indent_level + 1, tabstop);
        output.push("{".to_string());
        output.extend(bodytext.iter().map(|line| format!("{indent}{line}")));
        output.push("}".to_string());
        output
    }
}

pub struct TexBrackGroup {
    children: Vec<TexElement>,
}

impl TexBrackGroup {
    fn from(node: &SyntaxNode) -> Self {
        let mut children = Vec::new();
        for child in node.children() {
            match child.kind() {
                SyntaxKind::KEY_VALUE_BODY => {
                    children.extend(child.children().map(|c| TexElement::from(&c)));
                }
                _ => {
                    children.push(TexElement::from(&child));
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

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = String::new();
        output.push_str(&self.open_delim);
        for child in self.children.iter() {
            output.push_str(
                child
                    .format(indent_level, tabstop, line_length)
                    .join("")
                    .split(",")
                    .collect::<Vec<&str>>()
                    .join(", ")
                    .to_string()
                    .trim(),
            );
        }
        output.push_str(&self.close_delim);
        vec![output]
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

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = Vec::new();
        for child in self.children.iter() {
            output.extend(child.format(indent_level, tabstop, line_length));
        }
        vec![format!("{{{}}}", output.join(", ").trim().to_string())]
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

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        match self {
            TexEnvironmentParent::Tex(tex) => {
                let mut output = tex.format(indent_level, tabstop, line_length);
                for child in &tex.body.children {
                    if matches!(child, TexElement::BlankLine | TexElement::Comment(_)) {
                        output.extend(child.format(indent_level, tabstop, line_length));
                    }
                }
                output
            }
            TexEnvironmentParent::Math(math) => math.format(indent_level, tabstop, line_length),
        }
    }
}

pub struct TexBeginEnvironment {
    name: String,
    args: Vec<TexElement>,
    tail: Vec<TexElement>,
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
        let tail = blanklines_and_comments(node);
        Self { name, args, tail }
    }

    pub fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut line = String::new();
        let mut output = Vec::new();
        let mut indent = 0;
        if indent_level > 0 {
            indent = indent_level - 1;
        }
        line.push_str(&format!("{}\\begin", indent_str(indent, tabstop)));
        for arg in &self.args {
            let fmt = arg.format(indent_level, tabstop, line_length);
            line.push_str(&fmt[0]);
        }
        if !line.trim().is_empty() {
            output.push(line);
        }
        for line in &self.tail {
            output.extend(line.format(indent_level, tabstop, line_length));
        }
        output
    }
}

pub struct TexEndEnvironment {
    name: String,
    tail: Vec<TexElement>,
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
        let tail = blanklines_and_comments(node);
        Self { name, tail }
    }

    pub fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
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
        for line in &self.tail {
            output.extend(line.format(indent, tabstop, line_length));
        }
        output
    }
}

pub struct TexEnvironment {
    body: TexParent,
}

impl TexEnvironment {
    pub fn from(node: &SyntaxNode) -> Self {
        let body: TexParent = TexParent {
            children: node
                .children()
                .map(|node: SyntaxNode| TexElement::from(&node))
                .collect(),
        };

        //if let Some(last_child) = node.last_child() {
        //body.children.extend(blanklines_and_comments(&last_child));
        //}

        Self { body }
    }

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut subindent = 1;
        if let Some(TexElement::BeginEnvironment(child)) = self.body.children.first() {
            if matches!(child.name.as_str(), "document" | "verbatim") {
                subindent = 0;
            }
        }

        self.body
            .format(indent_level + subindent, tabstop, line_length)
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

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        match &self.value {
            Some(val) => vec![format!("{}={val}", self.key)],
            None => vec![format!("{}", self.key)],
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

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let output = self.body.format(indent_level, tabstop, line_length - 6);
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

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = Vec::new();
        let indent = indent_str(indent_level, tabstop);
        if self.inline {
            output.push("\\( ".to_string());
        } else {
            output.push(format!("{indent}\\["));
        }

        if self.inline {
            let fmt = self.body.format(indent_level, tabstop, line_length);
            if fmt.join("").trim().len() + 6 <= line_length {
                output.push(fmt.join("").trim().to_string());
                output.push(" \\)".to_string());
                vec![output.join("").trim().to_string()]
            } else {
                let fmt = self.body.format(indent_level, tabstop, line_length);
                output.extend(fmt);
                output.push("\\)".to_string());
                output
            }
        } else {
            let fmt = self.body.format(indent_level + 1, tabstop, line_length);
            output.extend(fmt);
            output.push(format!("{indent}\\]"));
            output
        }
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

    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = Vec::new();
        output.push(String::new());
        output.push(format!("{}{{{}}}", self.kind, self.name));
        output.push(String::new());
        output.extend(self.body.format(indent_level, tabstop, line_length));
        output
    }
}

#[cfg(test)]
mod test {

    use crate::Formatter;
    use expect_test::{expect, Expect};
    use parser::{parse_latex, SyntaxConfig};

    fn check(input: &str, expect: Expect) {
        let root =
            syntax::latex::SyntaxNode::new_root(parse_latex(input, &SyntaxConfig::default()));
        let mut formatter = Formatter::new();
        let output = formatter.format(&root);
        let output: Vec<String> = output.split('\n').map(|s| s.to_string()).collect();

        expect.assert_debug_eq(&output);
    }

    #[test]
    fn test_simple_command() {
        check(
            r"\textbf{Hello, world!}",
            expect![[r#"
                [
                    "\\textbf{Hello, world!}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_environment() {
        check(
            r"\begin{itemize}\item First item\item Second item\end{itemize}",
            expect![[r#"
                [
                    "\\begin{itemize}",
                    "  \\item First item",
                    "  \\item Second item",
                    "\\end{itemize}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_section() {
        check(
            r"\section{Introduction}This is the introduction.",
            expect![[r#"
                [
                    "",
                    "\\section{Introduction}",
                    "",
                    "This is the introduction.",
                ]
            "#]],
        );
    }

    #[test]
    fn test_formula() {
        check(
            r"$E = mc^2$",
            expect![[r#"
                [
                    "\\( E = mc^2 \\)",
                ]
            "#]],
        );
    }

    #[test]
    fn test_nested_environment() {
        check(
            r"\begin{figure}\begin{center}\includegraphics{image.png}\end{center}\end{figure}",
            expect![[r#"
                [
                    "\\begin{figure}",
                    "  \\begin{center}",
                    "    \\includegraphics{image.png}",
                    "  \\end{center}",
                    "\\end{figure}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_key_val_pair() {
        check(
            r"\usepackage[utf8]{inputenc}",
            expect![[r#"
                [
                    "\\usepackage[utf8]{inputenc}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_curly_group() {
        check(
            r"\textit{This is italic text.}",
            expect![[r#"
                [
                    "\\textit{This is italic text.}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_brack_group() {
        check(
            r"\documentclass[a4paper,12pt]{article}",
            expect![[r#"
                [
                    "\\documentclass[a4paper, 12pt]{article}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_url() {
        check(
            r"\href{https://www.example.com}{Example}",
            expect![[r#"
                [
                    "\\href{https://www.example.com}{Example}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_enum_item() {
        check(
            r"\begin{enumerate}\item First item\item Second item\end{enumerate}",
            expect![[r#"
                [
                    "\\begin{enumerate}",
                    "  \\item First item",
                    "  \\item Second item",
                    "\\end{enumerate}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_nested_curly_group() {
        check(
            r"\textbf{Bold \textit{and italic} text}",
            expect![[r#"
                [
                    "\\textbf{Bold \\textit{and italic} text}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_complex_command() {
        check(
            r"\newcommand{\mycommand}[1]{This is #1}",
            expect![[r#"
                [
                    "\\newcommand{\\mycommand}[1]{This is #1}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_multiline_formula() {
        check(
            r"\[
                E = mc^2
                \]",
            expect![[r#"
                [
                    "\\[",
                    "  E = mc^2",
                    "\\]",
                ]
            "#]],
        );
    }

    #[test]
    fn test_with_comment() {
        check(
            r"This is some text % with a comment on the same line",
            expect![[r#"
                    [
                        "This is some text % with a comment on the same line",
                    ]
                "#]],
        );
    }

    #[test]
    fn test_label_and_ref() {
        check(
            r"\label{sec:intro}\ref{sec:intro}",
            expect![[r#"
                [
                    "\\label{sec:intro}\\ref{sec:intro}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_text_inside_formula() {
        check(
            r"$E = mc^2 \text{(Energy Equation)}$",
            expect![[r#"
                [
                    "\\( E = mc^2 \\text{(Energy Equation)} \\)",
                ]
            "#]],
        );
    }

    #[test]
    fn test_command_sequence() {
        check(
            r"\textbf{Bold} \textit{Italic} \underline{Underlined}",
            expect![[r#"
                [
                    "\\textbf{Bold}\\textit{Italic}\\underline{Underlined}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_complex_nested_environment() {
        check(
            r"\begin{center}\begin{minipage}{0.5\textwidth}\textbf{Nested content}\end{minipage}\end{center}",
            expect![[r#"
                [
                    "\\begin{center}",
                    "  \\begin{minipage}{0.5\\textwidth}",
                    "    \\textbf{Nested content}",
                    "  \\end{minipage}",
                    "\\end{center}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_command_with_multiple_brack_args() {
        check(
            r"\command[arg1,arg2,arg3]{Some text}",
            expect![[r#"
                [
                    "\\command[arg1, arg2, arg3]{Some text}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_section_with_subsection() {
        check(
            r"\section{Main Section}\subsection{Sub Section}Content here",
            expect![[r#"
                [
                    "",
                    "\\section{Main Section}",
                    "",
                    "",
                    "\\subsection{Sub Section}",
                    "",
                    "Content here",
                ]
            "#]],
        );
    }

    #[test]
    fn test_nested_brack_and_curly_args() {
        check(
            r"\command[option1,option2]{\textbf{Bold text} and \textit{Italic text}}",
            expect![[r#"
                [
                    "\\command[option1, option2]{\\textbf{Bold text} and \\textit{Italic text}}",
                ]
            "#]],
        );
    }

    //#[test]
    //fn test_environment_with_arguments() {
    //check(
    //r"\begin{theorem}[label=thm:example,another=option]\textbf{A bold statement}\end{theorem}",
    //expect![[r#"
    //[
    //"\\begin{theorem}[label=thm:example, another=option]",
    //"  \\textbf{A bold statement}",
    //"\\end{theorem}",
    //]
    //"#]],
    //);
    //}

    #[test]
    fn test_command_with_multiple_required_args() {
        check(
            r"\newcommand{\cmd}[1]{Arg one is #1}",
            expect![[r#"
                    [
                        "\\newcommand{\\cmd}[1]{Arg one is #1}",
                    ]
                "#]],
        );
    }

    #[test]
    fn test_multiple_environments_in_sequence() {
        check(
            r"\begin{center}Centered text\end{center}\begin{flushright}Right text\end{flushright}",
            expect![[r#"
                    [
                        "\\begin{center}",
                        "  Centered text",
                        "\\end{center}",
                        "\\begin{flushright}",
                        "  Right text",
                        "\\end{flushright}",
                    ]
                "#]],
        );
    }

    #[test]
    fn test_verbatim_environment() {
        check(
            r"\begin{verbatim}Some verbatim text\end{verbatim}",
            expect![[r#"
                [
                    "\\begin{verbatim}",
                    "Some verbatim text",
                    "\\end{verbatim}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_command_with_semicolon_args() {
        check(
            r"\command[option1;option2]{\textbf{Hello}}",
            expect![[r#"
                [
                    "\\command[option1;option2]{\\textbf{Hello}}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_multiple_inline_formulas() {
        check(
            r"$E=mc^2$ and $a^2 + b^2 = c^2$",
            expect![[r#"
                [
                    "\\( E = mc^2 \\) and \\( a^2 + b^2 = c^2 \\)",
                ]
            "#]],
        );
    }

    #[test]
    fn test_word_wrap() {
        check(
            ["\\documentclass{article}\\begin{document}\\begin{center} This is a long line. This is",
                "a long line. This is a long line. This is a long line. This is a long line. This is",
                "a long line. This is a long line. This is a long line. This is a long line. This is",
                "a long line. This is a long line.\\end{center}\\end{document}"].join(" ").as_str(),
            expect![[r#"
                [
                    "\\documentclass{article}",
                    "\\begin{document}",
                    "\\begin{center}",
                    "  This is a long line. This is a long line. This is a long line. This is a long",
                    "  line. This is a long line. This is a long line. This is a long line. This is a",
                    "  long line. This is a long line. This is a long line. This is a long line.",
                    "\\end{center}",
                    "\\end{document}",
                ]
            "#]],
        );
    }

    #[test]
    fn test_enum_with_formulas() {
        check(
            [
                "\\documentclass{article}\\begin{document}\\begin{enumerate}",
                "\\item $\\overrightarrow{a}(t)=\\langle t,t^2,t^3\\rangle$ with $\\overrightarrow{v}(2)=\\langle 2,3,-4\\rangle$ and $\\overrightarrow{r}(1)=\\langle 1,1,1\\rangle$",
                "\\item $\\overrightarrow{a}(t)=\\langle \\cos(t),\\frac{-2t}{(1+t^2)^2},\\frac{t}{(1-t^2)^{3/2}}\\rangle$ with $\\overrightarrow{v}(0)=\\langle 0,1,1\\rangle$", 
                "and $\\overrightarrow{r}(0)=\\langle -1,0,0\\rangle$",
                "\\end{enumerate}\\end{document}"
            ].join("").as_str(),
            expect![[r#"
                [
                    "\\documentclass{article}",
                    "\\begin{document}",
                    "\\begin{enumerate}",
                    "  \\item \\( \\overrightarrow{a}(t) = \\langle t, t^2, t^3 \\rangle \\) with",
                    "    \\( \\overrightarrow{v}(2) = \\langle 2, 3, - 4 \\rangle \\) and",
                    "    \\( \\overrightarrow{r}(1) = \\langle 1, 1, 1 \\rangle \\)",
                    "  \\item",
                    "    \\(",
                    "      \\overrightarrow{a}(t) = \\langle \\cos(t), \\frac{- 2t}{(1 + t^2)^2},",
                    "      \\frac{t}{(1 - t^2)^{3 / 2}} \\rangle",
                    "    \\)",
                    "    with \\( \\overrightarrow{v}(0) = \\langle 0, 1, 1 \\rangle \\) and",
                    "    \\( \\overrightarrow{r}(0) = \\langle - 1, 0, 0 \\rangle \\)",
                    "\\end{enumerate}",
                    "\\end{document}",
                ]
            "#]]

        );
    }

    #[test]
    fn test_env_end_with_command() {
        check(
            r#"\begin{frame}\titlepage\end{frame}"#,
            expect![[r#"
            [
                "\\begin{frame}",
                "  \\titlepage",
                "\\end{frame}",
            ]
        "#]],
        )
    }

    #[test]
    fn test_respect_blanklines_after_commands() {
        check(
            r#"\documentclass[aspectratio=169]{beamer}
\usetheme{Madrid}
\usecolortheme{dove}

\usepackage{graphicx,amsmath,amssymb,amsfonts}
\usepackage{physics}
\usepackage{hyperref}

\title[2D Vectors]{2D Vectors: Definitions, Equations, and Problems}
\author{Your Name}
\institute{Your Institution}
\date{\today}
"#,
            expect![[r#"
                [
                    "\\documentclass[aspectratio=169]{beamer}",
                    "\\usetheme{Madrid}",
                    "\\usecolortheme{dove}",
                    "",
                    "\\usepackage{graphicx, amsmath, amssymb, amsfonts}",
                    "\\usepackage{physics}",
                    "\\usepackage{hyperref}",
                    "",
                    "\\title[2D Vectors]{2D Vectors: Definitions, Equations, and Problems}",
                    "\\author{Your Name}",
                    "\\institute{Your Institution}",
                    "\\date{\\today}",
                ]
            "#]],
        )
    }
}
