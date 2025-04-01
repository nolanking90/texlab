use syntax::latex::{SyntaxElement, SyntaxKind, SyntaxNode};

use crate::math::{MathEnvironment, MathParent};

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
    Comment(TexComment),
    Blankline,
    KeyValBody(TexKeyValParent),
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
            | SyntaxKind::HYPERSETUP
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
                TexElement::Text(node.to_string().replace("\n", " "))
            }
            SyntaxKind::CURLY_GROUP_KEY_VALUE | SyntaxKind::CURLY_GROUP => {
                TexElement::CurlyGroup(TexCurlyGroup::from(node))
            }
            SyntaxKind::BRACK_GROUP | SyntaxKind::BRACK_GROUP_KEY_VALUE => {
                TexElement::BrackGroup(TexBrackGroup::from(node))
            }
            SyntaxKind::CURLY_GROUP_COMMAND
            | SyntaxKind::CURLY_GROUP_WORD
            | SyntaxKind::CURLY_GROUP_WORD_LIST => {
                TexElement::CurlyGroupWordList(TexCurlyGroupWordList::from(node))
            }
            SyntaxKind::KEY_VALUE_PAIR => TexElement::KeyValPair(TexKeyVal::from(node)),
            SyntaxKind::MIXED_GROUP => TexElement::MixedGroup(TexMixedGroup::from(node)),
            SyntaxKind::ENUM_ITEM => TexElement::EnumItem(TexEnumItem::from(node)),
            SyntaxKind::EQUATION | SyntaxKind::FORMULA => {
                TexElement::Formula(TexFormula::from(node))
            }
            SyntaxKind::COMMENT => TexElement::Comment(TexComment::from(node)),
            SyntaxKind::KEY_VALUE_BODY => TexElement::KeyValBody(TexKeyValParent::from(node)),
            SyntaxKind::BLANKLINE => TexElement::Blankline,
            _ => TexElement::Text(String::new()),
        }
    }
}

pub struct TexParent {
    pub children: Vec<TexElement>,
}

impl TexParent {
    fn from(node: &SyntaxNode) -> Self {
        let children = node
            .children_with_tokens()
            .map(|child| match child {
                SyntaxElement::Node(n) => TexElement::from(&n),
                SyntaxElement::Token(t) => match t.kind() {
                    SyntaxKind::WHITESPACE => {
                        if t.to_string() == *"\\n\\n" {
                            TexElement::Blankline
                        } else {
                            TexElement::Text(String::new())
                        }
                    }
                    SyntaxKind::HREF => TexElement::Text(t.to_string()),
                    _ => TexElement::Text(String::new()),
                },
            })
            .collect();
        Self { children }
    }
}

pub struct TexCommand {
    pub name: String,
    pub args: Vec<TexElement>,
    pub start_newline: bool,
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
                SyntaxKind::CURLY_GROUP | SyntaxKind::CURLY_GROUP_KEY_VALUE => {
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

        let start_newline = matches!(name.as_str(), "\\newline" | "\\newpage" | "\\documentclass")
            || (node.parent().is_some() && node.parent().unwrap().kind() == SyntaxKind::PREAMBLE);

        Self {
            name,
            args,
            start_newline,
        }
    }
}

pub struct TexCurlyGroup {
    pub body: TexParent,
}

impl TexCurlyGroup {
    fn from(node: &SyntaxNode) -> Self {
        Self {
            body: TexParent::from(node),
        }
    }
}

pub struct TexBrackGroup {
    pub body: TexParent,
}

impl TexBrackGroup {
    fn from(node: &SyntaxNode) -> Self {
        Self {
            body: TexParent::from(node),
        }
    }
}

pub struct TexMixedGroup {
    pub children: Vec<TexElement>,
    pub open_delim: String,
    pub close_delim: String,
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
}

pub struct TexCurlyGroupWordList {
    pub children: Vec<TexElement>,
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
}

pub struct TexEnvironment {
    pub name: String,
    pub args: Vec<TexElement>,
    pub body: TexParent,
}

impl TexEnvironment {
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
                child.kind() == SyntaxKind::BRACK_GROUP || child.kind() == SyntaxKind::CURLY_GROUP
            })
            .collect();
        let opt_args = TexElement::Text(
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
            args.push(TexElement::Text(
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

        let children: Vec<SyntaxNode> = node
            .children()
            .filter(|child| *child != first_child && *child != second_child)
            .collect();
        let body: TexParent = TexParent {
            children: children.iter().map(TexElement::from).collect(),
        };

        Self { name, args, body }
    }
}

pub struct TexKeyValParent{
    pub children: Vec<TexElement>,
}

impl TexKeyValParent {
    pub fn from(node: &SyntaxNode) -> Self {
        Self {
            children: node
                .children()
                .map(|child| TexElement::from(&child))
                .collect(),
        }
    }

}

pub struct TexKeyVal {
    pub key: String,
    pub value: Option<String>,
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
}

pub struct TexEnumItem {
    pub body: TexParent,
}

impl TexEnumItem {
    fn from(node: &SyntaxNode) -> Self {
        Self {
            body: TexParent::from(node),
        }
    }
}


pub struct TexFormula {
    pub inline: bool,
    pub body: MathParent,
}

impl TexFormula {
    fn from(node: &SyntaxNode) -> Self {
        let inline = matches!(node.first_token().unwrap().text(), "$" | "\\(");
        let body = MathParent::from(node);
        Self { inline, body }
    }
}

pub struct TexSection {
    pub kind: String,
    pub name: String,
    pub body: TexParent,
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
}

pub struct TexComment {
    pub comment: String,
}

impl TexComment {
    fn from(node: &SyntaxNode) -> Self {
        TexComment {
            comment: node.to_string().trim().to_string(),
        }
    }
}
