use crate::Formatter;
use expect_test::{expect, Expect};
use parser::{parse_latex, SyntaxConfig};
use syntax::latex;

fn check(input: &str, expect: Expect) {
    let root = syntax::latex::SyntaxNode::new_root(parse_latex(input, &SyntaxConfig::default()));
    let mut formatter = Formatter::new();
    let output = formatter.visit(&latex::SyntaxElement::from(root));

    expect.assert_debug_eq(&output);
}

#[test]
fn test_environment() {
    check(
        r#"
\begin{center}some text.\end{center}
        "#,
        expect![[r#"
            "\\begin{center}some text.\\end{center}\n"
        "#]],
    );
}
