use crate::Formatter;
use expect_test::{expect, Expect};
use parser::{parse_latex, SyntaxConfig};

fn check(input: &str, expect: Expect) {
    let root = syntax::latex::SyntaxNode::new_root(parse_latex(input, &SyntaxConfig::default()));
    let mut formatter = Formatter::new();
    let output = formatter.format(&root);

    expect.assert_debug_eq(&output);
}

#[test]
fn test_environment() {
    check(
        r#"\begin{center}some text.\end{center}"#,
        expect![[r#"
            "\n\\begin{center}\n  some text.\n\\end{center}"
        "#]],
    );
}

#[test]
fn test_nested_environment() {
    check(
        r#"
\begin{itemize}\begin{itemize}
    \begin{center}Centered item\end{center}
\end{itemize}
        "#,
        expect![[r#"
            "\n\\begin{itemize}\n\n  \\begin{itemize}\n\n    \\begin{center}\n      Centered item\n    \\end{center}\n  \\end{itemize}\n\\end{itemize}"
        "#]],
    );
}

#[test]
fn test_command_with_arguments() {
    check(
        r#"\newcommand{\vect}[1]{\begin{pmatrix}#1\end{pmatrix}}"#,
        expect![[r#"
            "\\newcommand{\\vect}[1]{\\begin{pmatrix}#1\\end{pmatrix}}\n"
        "#]],
    );
}

#[test]
fn test_math_environment() {
    check(
        r#"\begin{equation}E=mc^2\end{equation}"#,
        expect![[r#"
            "\n\\begin{equation}\n  E = mc^2\n\\end{equation}"
        "#]],
    );
}

#[test]
fn test_display_math_environment() {
    check(
        r#"\[E=mc^2\]"#,
        expect![[r#"
            "\\[\n  E = mc^2\n\\]"
        "#]],
    );
}

#[test]
fn test_comment_handling() {
    check(
        r#"
% This is a comment
\begin{itemize}\item Item 1 % Comment after item
\item Item 2\end{itemize}
        "#,
        expect![[r#"
            "\n\\begin{itemize}\n  \\item Item 1 % Comment after item\n  \\item Item 2\n\\end{itemize}"
        "#]],
    );
}

#[test]
fn test_inline_math() {
    check(
        r#"Here is some inline math: $E=mc^2$."#,
        expect![[r#"
            "Here is some inline math:$ E = mc^2 $."
        "#]],
    );
}

#[test]
fn test_section_heading() {
    check(
        r#"\section{Introduction}"#,
        expect![[r#"
            "\n\n\\section{Introduction}"
        "#]],
    );
}
