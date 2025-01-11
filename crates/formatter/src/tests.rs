use crate::Formatter;
use crate::tex::TexCommand;
use expect_test::{expect, Expect};
use parser::{parse_latex, SyntaxConfig};

fn check_len(input: &str, expect: Expect) {
    let root = syntax::latex::SyntaxNode::new_root(parse_latex(input, &SyntaxConfig::default()));
    let len = TexCommand::from(&root.first_child().unwrap().first_child().unwrap()).len();
    expect.assert_eq(&len.to_string());
}

fn check(input: &str, expect: Expect) {
    let root = syntax::latex::SyntaxNode::new_root(parse_latex(input, &SyntaxConfig::default()));
    let mut formatter = Formatter::new(2, 80);
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

#[test]
fn test_environment_with_arguments() {
    check(
        r"\begin{theorem}[label=thm:example,another=option]\textbf{A bold statement}\end{theorem}",
        expect![[r#"
            [
                "\\begin{theorem}[label=thm:example, another=option]",
                "  \\textbf{A bold statement}",
                "\\end{theorem}",
            ]
        "#]],
    );
}

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
        [
            "\\documentclass{article}\\begin{document}\\begin{center} This is a long line. This is",
            "a long line. This is a long line. This is a long line. This is a long line. This is",
            "a long line. This is a long line. This is a long line. This is a long line. This is",
            "a long line. This is a long line.\\end{center}\\end{document}",
        ]
        .join(" ")
        .as_str(),
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

#[test]
fn test_long_doc() {
    check(
        r#"\documentclass{article}

\begin{document}

Matching brackets on a line do nothing (like this).

Matching brackets on two lines also do nothing (like this
longer example).

Matching brackets on three lines get an indent (like this
much much longer example
right here on these lines).

Matching brackets on more lines also get an indent (like this
much much
much much
much longer example
here).

The brackets could start at the beginning of the line
(so maybe
they look
like this).

[They could
be any shape
of bracket]

{Even braces get
the same
indents too}

What about equations? They are the same:
$(1 + 2 + 3)$

$(1 + 2
+ 3 + 4
+ 5 + 7
+ 8 + 9)$

And the dollars can go anywhere as expected:

$
(1 + 2
+ 3 + 4
+ 5 + 7
+ 8 + 9)
$

Note that dollars themselves are not indented

\end{document}
        "#,
        expect![[r#"
            [
                "\\documentclass{article}",
                "",
                "\\begin{document}",
                "",
                "Matching brackets on a line do nothing (like this).",
                "",
                "Matching brackets on two lines also do nothing (like this longer example).",
                "",
                "Matching brackets on three lines get an indent",
                "(like this much much longer example right here on these lines).",
                "",
                "Matching brackets on more lines also get an indent",
                "(like this much much much much much longer example here).",
                "",
                "The brackets could start at the beginning of the line",
                "(so maybe they look like this).",
                "",
                "[They could be any shape of bracket]",
                "",
                "{Even braces get the same indents too}",
                "",
                "What about equations? They are the same: \\( (1 + 2 + 3) \\)",
                "",
                "\\( (1 + 2 + 3 + 4 + 5 + 7 + 8 + 9) \\)",
                "",
                "And the dollars can go anywhere as expected:",
                "",
                "\\( (1 + 2 + 3 + 4 + 5 + 7 + 8 + 9) \\)",
                "",
                "Note that dollars themselves are not indented",
                "",
                "\\end{document}",
            ]
        "#]],
    )
}

#[test]
fn test_new_command_long() {
    check(
        r#"\newcommand{\headandfoot}[3]{\lhead{#1}\chead{#2}\rhead{Section 16 {\hspace{.25in}}}\lfoot{\copyright Pennsylvania State University}\cfoot{\thepage}\rfoot{#3}}"#,
        expect![[r#"
            [
                "\\newcommand{\\headandfoot}[3]{",
                "  \\lhead{#1}\\chead{#2}\\rhead{Section 16 {\\hspace{.25in}}}%",
                "  \\lfoot{\\copyright~Pennsylvania State University}\\cfoot{\\thepage}\\rfoot{#3}%",
                "}",
            ]
        "#]],
    )
}

#[test]
fn test_new_command_respect_line_breaks() {
    check(
r#"\newcommand{\headandfoot}[3]{
  \lhead{#1}\chead{#2}\rhead{Section 16 {\hspace{.25in}}}%
  \lfoot{\copyright~Pennsylvania State University}\cfoot{\thepage}%
  \rfoot{#3}
}"#,
        expect![[r#"
            [
                "\\newcommand{\\headandfoot}[3]{",
                "  \\lhead{#1}\\chead{#2}\\rhead{Section 16 {\\hspace{.25in}}}%",
                "  \\lfoot{\\copyright~Pennsylvania State University}\\cfoot{\\thepage}%",
                "  \\rfoot{#3}%",
                "}",
            ]
        "#]],
    )
}

#[test]
fn test_command_len() {
    check_len(
        r#"\rfoot{#3}"#,
        expect![[r#"10"#]],
    )
}
