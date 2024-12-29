use parser::{parse_latex, SyntaxConfig};
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::tree::decision_tree_classifier::{
    DecisionTreeClassifier, DecisionTreeClassifierParameters,
};
use std::fs;
use std::io;
use std::path::PathBuf;
use syntax::latex::{SyntaxElement, SyntaxKind, SyntaxNode};

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
    NoBreak,       // no space at all
    Space,         // a single space
    NewLine(bool), // break + indent
}

fn decision_to_integer(d: &FormatDecision) -> u32 {
    match d {
        FormatDecision::NoBreak => 0,
        FormatDecision::Space => 1,
        FormatDecision::NewLine(true) => 2,
        FormatDecision::NewLine(false) => 3,
    }
}

fn label_to_decision(label: u32) -> FormatDecision {
    match label {
        0 => FormatDecision::NoBreak,
        1 => FormatDecision::Space,
        2 => FormatDecision::NewLine(true),
        3 => FormatDecision::NewLine(false),
        // fallback
        _ => FormatDecision::NoBreak,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Features {
    kind: SyntaxKind,
    parent_kind: Option<SyntaxKind>,
    is_section_command: bool,
    token_length: usize, // 0 for a node
    nesting_depth: usize,
    is_math_mode: bool,
    is_aligned_env: bool, // align, array, and matrix envs., and tabular type
}

fn features_to_vec(f: &Features) -> Vec<f32> {
    vec![
        kind_to_f32(&f.kind),
        optional_kind_to_f32(f.parent_kind),
        f.is_section_command as u8 as f32,
        f.token_length as f32,
        f.nesting_depth as f32,
        f.is_math_mode as u8 as f32,
        f.is_aligned_env as u8 as f32,
    ]
}

fn extract_features(element: &SyntaxElement) -> Features {
    let parent_kind = element.parent().map(|parent| parent.kind());

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

    Features {
        kind: element.kind(),
        parent_kind,
        is_section_command,
        token_length,
        nesting_depth,
        is_math_mode: is_math_mode(element),
        is_aligned_env: is_aligned_env(element),
    }
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
                match token.next_token() {
                    Some(tkn) => match tkn.to_string().as_str() {
                        "\t" => FormatDecision::NewLine(true),
                        _ => FormatDecision::NewLine(false),
                    },
                    None => FormatDecision::NewLine(false),
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
    let n_features = 7; // in our example (kind_val, parent_val, etc.)
    let n_samples = dataset.len();

    let x = DenseMatrix::new(n_samples, n_features, data, false).unwrap();
    (x, labels)
}

pub struct Classifier(DecisionTreeClassifier<f32, u32, DenseMatrix<f32>, Vec<u32>>);

pub fn train(dataset: &[TrainingExample]) -> Classifier {
    let (x, y) = dataset_to_matrix(dataset);

    let params = DecisionTreeClassifierParameters::default().with_max_depth(10);

    Classifier(DecisionTreeClassifier::fit(&x, &y, params).expect("Failed to train decision tree"))
}

pub fn predict(x: &DenseMatrix<f32>, classifier: &Classifier) -> Vec<FormatDecision> {
    classifier.0.predict(x).unwrap().iter().map(|&pred| label_to_decision(pred)).collect()
}

pub fn visit(element: &SyntaxElement, prediction: &mut Vec<FormatDecision>) -> String {
    let mut output = String::new();
    let pred = prediction.pop().unwrap();
    match element {
        SyntaxElement::Token(token) => {
            output.push_str(&token.to_string());
            println!("{}", &token.to_string());
            println!("{:?}", pred);
            match pred {
                FormatDecision::Space => { output.push(' '); },
                FormatDecision::NoBreak => {},
                FormatDecision::NewLine(true) => { output.push_str("\n  "); }
                FormatDecision::NewLine(false) => { output.push('\n'); }
            }
        },
        SyntaxElement::Node(node) => {
            for child in node.children_with_tokens() {
                output.push_str(&visit(&child, prediction));
            }
        }
    }
    output
}
