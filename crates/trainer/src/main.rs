use parser::{parse_latex, SyntaxConfig};
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::tree::decision_tree_classifier::{
    DecisionTreeClassifier, DecisionTreeClassifierParameters, SplitCriterion,
};
use std::fs;
use std::io;
use std::path::PathBuf;
use syntax::latex::{SyntaxElement, SyntaxKind, SyntaxNode};
use xxhash_rust::xxh3::xxh3_64;

fn main() -> io::Result<()> {
    let examples_dir = "texExamples";
    let mut training_data = Vec::new();
    let mut input_data = Vec::new();
    let mut total = 0;

    for entry in fs::read_dir(examples_dir)? {
        let entry = entry?;
        let path: PathBuf = entry.path();

        if let Some(ext) = path.extension() {
            if ext == "tex" {
                let input = fs::read_to_string(&path)?;
                let root = SyntaxNode::new_root(parse_latex(&input, &SyntaxConfig::default()));
                training_data.extend(build_training_data(&SyntaxElement::Node(root)));
                total += 1;
            }
        }
    }

    println!("We parsed {total} examples.");

    println!("Starting training.");
    let classifier = train(&training_data);

    println!("Trained.");
    for entry in fs::read_dir("testinput")? {
        let entry = entry?;
        let path: PathBuf = entry.path();

        if let Some(ext) = path.extension() {
            if ext == "tex" {
                let input = fs::read_to_string(&path)?;
                let root = SyntaxNode::new_root(parse_latex(&input, &SyntaxConfig::default()));
                input_data.extend(build_training_data(&SyntaxElement::Node(root.clone())));
                let (input_matrix, _) = dataset_to_matrix(&input_data);
                let mut prediction = predict(&input_matrix, &classifier);
                let formatted_doc = visit(&SyntaxElement::Node(root), &mut prediction);
                println!("{formatted_doc}");
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormatDecision {
    NoBreak,      // no space at all
    Space,        // a single space
    NewLine(u32), // break + indent
    SkipLine(u32),
}

fn decision_to_integer(d: &FormatDecision) -> u32 {
    match d {
        FormatDecision::NoBreak => 0,
        FormatDecision::Space => 1,
        FormatDecision::NewLine(num) => 2 * (*num + 1), // even numbers >= 2
        FormatDecision::SkipLine(num) => 2 * (*num + 1) + 1, // odd numbers >= 3
    }
}

fn label_to_decision(label: u32) -> FormatDecision {
    match label {
        0 => FormatDecision::NoBreak,
        1 => FormatDecision::Space,
        _ => {
            if label % 2 == 0 {
                let num = (label / 2) - 1;
                FormatDecision::NewLine(num)
            } else {
                let num = (label - 1) / 2 - 1;
                FormatDecision::SkipLine(num)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Features {
    kind: SyntaxKind,
    name: String,
    parent_kind: Option<SyntaxKind>,
    l_sibling_kind: Option<SyntaxKind>,
    r_sibling_kind: Option<SyntaxKind>,
    is_section_command: bool,
    token_length: usize, // 0 for a node
    nesting_depth: usize,
    is_math_mode: bool,
    is_aligned_env: bool, // align, array, and matrix envs., and tabular type
    num_children: usize,
    num_siblings: usize,
}

fn features_to_vec(f: &Features) -> Vec<f32> {
    vec![
        kind_to_f32(&f.kind),
        string_to_hash(&f.name) as f32,
        optional_kind_to_f32(f.parent_kind),
        optional_kind_to_f32(f.l_sibling_kind),
        optional_kind_to_f32(f.r_sibling_kind),
        f.is_section_command as u8 as f32,
        f.token_length as f32,
        f.nesting_depth as f32,
        f.is_math_mode as u8 as f32,
        f.is_aligned_env as u8 as f32,
        f.num_children as f32,
        f.num_siblings as f32,
    ]
}

fn extract_features(element: &SyntaxElement) -> Features {
    let parent_kind = element.parent().map(|parent| parent.kind());
    let r_sibling_kind = element.next_sibling_or_token().map(|sib| sib.kind());
    let l_sibling_kind = element.prev_sibling_or_token().map(|sib| sib.kind());
    let num_siblings = match element.parent() {
        Some(parent) => parent.children().count(),
        None => 0
    };
    let num_children = match element.as_node() {
        Some(node) => node.children().count(),
        None => 0
    };

    let is_section_command = matches!(
        element.kind(),
        SyntaxKind::SECTION
            | SyntaxKind::SUBSECTION
            | SyntaxKind::SUBSUBSECTION
            | SyntaxKind::CHAPTER
            | SyntaxKind::PART
    );

    let token_length = match element {
        SyntaxElement::Token(token) => usize::from(token.text_range().len()),
        SyntaxElement::Node(_) => 0,
    };

    let nesting_depth = element.ancestors().count();
    let mut name = String::new();
    if matches!(element.kind(), SyntaxKind::ENVIRONMENT) {
       name = get_env_name(element.as_node().unwrap());
    }
    if matches!(element.kind(), SyntaxKind::GENERIC_COMMAND) {
        name = get_command_name(element.as_node().unwrap());
    }

    Features {
        kind: element.kind(),
        name,
        parent_kind,
        l_sibling_kind,
        r_sibling_kind,
        is_section_command,
        token_length,
        nesting_depth,
        is_math_mode: is_math_mode(element),
        is_aligned_env: is_aligned_env(element),
        num_children,
        num_siblings,
    }
}

fn get_command_name(node: &SyntaxNode) -> String {
    let name = node.first_child_or_token().unwrap().to_string();
    name
}

fn string_to_hash(input: &str) -> u64 {
    xxh3_64(input.as_bytes())
}

fn is_math_mode(element: &SyntaxElement) -> bool {
    let eq_count = element
        .ancestors()
        .filter(|anc| matches!(anc.kind(), SyntaxKind::EQUATION))
        .count();

    eq_count > 0
        || element
            .ancestors()
            .filter(|anc| anc.kind() == SyntaxKind::ENVIRONMENT)
            .filter(|env| {
                matches!(
                    get_env_name(env).as_str(),
                    "equation"
                        | "equation*"
                        | "align"
                        | "align*"
                        | "aligned"
                        | "alignat"
                        | "alignat*"
                        | "alignedat"
                        | "gather"
                        | "gather*"
                        | "math"
                        | "displaymath"
                        | "frac"
                        | "eqnarray"
                        | "matrix"
                        | "bmatrix"
                        | "pmatrix"
                        | "multiline"
                        | "array"
                )
            })
            .count()
            > 0
}

fn get_env_name(node: &SyntaxNode) -> String {
    node.first_child()
        .unwrap()
        .first_child()
        .unwrap()
        .first_child()
        .unwrap()
        .to_string()
}

fn is_aligned_env(element: &SyntaxElement) -> bool {
    let eq_count = element
        .ancestors()
        .filter(|anc| matches!(anc.kind(), SyntaxKind::EQUATION))
        .count();

    eq_count > 0
        || element
            .ancestors()
            .filter(|anc| anc.kind() == SyntaxKind::ENVIRONMENT)
            .filter(|env| {
                matches!(
                    get_env_name(env).as_str(),
                    "align"
                        | "align*"
                        | "aligned"
                        | "alignat"
                        | "alignat*"
                        | "alignedat"
                        | "eqnarray"
                        | "matrix"
                        | "bmatrix"
                        | "pmatrix"
                        | "array"
                        | "table"
                        | "tabular"
                        | "tabularx"
                )
            })
            .count()
            > 0
}

fn extract_label(element: &SyntaxElement) -> FormatDecision {
    match element.next_sibling_or_token() {
        Some(SyntaxElement::Token(token)) => {
            if token.to_string() == *"\n" {
                let mut indents = 0;
                let mut lines = 0;
                let mut tkn = token.next_token();
                while tkn.is_some() {
                    if matches!(tkn.clone().unwrap().text(), "\t" | " ") {
                        indents += 1;
                    } else if matches!(tkn.clone().unwrap().text(), "\n") {
                        lines += 1;
                    }
                    tkn = tkn.unwrap().next_token();
                }
                if indents > 0 {
                    FormatDecision::NewLine(2 * (indents + 1))
                } else {
                    FormatDecision::SkipLine(2 * (lines + 1) + 1)
                }
            } else if token.to_string() == *" " {
                FormatDecision::Space
            } else {
                FormatDecision::NoBreak
            }
        }
        _ => FormatDecision::NoBreak,
    }
}

#[derive(Debug)]
pub struct TrainingExample {
    pub features: Features,
    pub label: FormatDecision,
}

fn training_helper(element: &SyntaxElement) -> TrainingExample {
    TrainingExample {
        features: extract_features(element),
        label: extract_label(element),
    }
}

fn build_training_data(root: &SyntaxElement) -> Vec<TrainingExample> {
    let mut data = Vec::new();
    data.push(training_helper(&root.clone()));
    if let SyntaxElement::Node(node) = root {
        for child in node.children_with_tokens() {
            data.extend(build_training_data(&child));
        }
    }
    data
}

fn kind_to_f32(kind: &SyntaxKind) -> f32 {
    *kind as u16 as f32 // stable numeric ID
}

fn optional_kind_to_f32(kind: Option<SyntaxKind>) -> f32 {
    match kind {
        Some(kind) => kind_to_f32(&kind),
        None => -1.0_f32,
    }
}

fn dataset_to_matrix(dataset: &[TrainingExample]) -> (DenseMatrix<f32>, Vec<u32>) {
    let mut data = Vec::new();
    let mut labels = Vec::new();

    for example in dataset {
        let feature_vec = features_to_vec(&example.features);
        data.extend(feature_vec);

        let lbl = decision_to_integer(&example.label);
        labels.push(lbl);
    }

    // We know how many features per example we used:
    let n_features = 12; // in our example (kind_val, parent_val, etc.)
    let n_samples = dataset.len();

    let x = DenseMatrix::new(n_samples, n_features, data, false).unwrap();
    (x, labels)
}

pub struct Classifier(DecisionTreeClassifier<f32, u32, DenseMatrix<f32>, Vec<u32>>);

pub fn train(dataset: &[TrainingExample]) -> Classifier {
    let (x, y) = dataset_to_matrix(dataset);

    let params = DecisionTreeClassifierParameters::default().with_criterion(SplitCriterion::Gini);

    Classifier(DecisionTreeClassifier::fit(&x, &y, params).expect("Failed to train decision tree"))
}

pub fn predict(x: &DenseMatrix<f32>, classifier: &Classifier) -> Vec<FormatDecision> {
    classifier
        .0
        .predict(x)
        .unwrap()
        .iter()
        .map(|&pred| label_to_decision(pred))
        .collect()
}

pub fn visit(element: &SyntaxElement, prediction: &mut Vec<FormatDecision>) -> String {
    let mut output = String::new();
    let pred = prediction.pop().unwrap();
    match element {
        SyntaxElement::Token(token) => {
            output.push_str(&token.to_string());
            //println!("{}", &token.to_string());
            //println!("{:?}", pred);
            match pred {
                FormatDecision::Space => {
                    output.push(' ');
                }
                FormatDecision::NoBreak => {}
                FormatDecision::NewLine(indent_level) => {
                    output.push_str("\n");
                    let indents = "\t".to_string().repeat((indent_level / 2 - 1) as usize);
                    output.push_str(&indents);
                }
                FormatDecision::SkipLine(lines) => {
                    output.push_str(&"\n".repeat(((lines - 1) / 2 - 1) as usize));
                }
            }
        }
        SyntaxElement::Node(node) => {
            for child in node.children_with_tokens() {
                output.push_str(&visit(&child, prediction));
            }
        }
    }
    output
}
