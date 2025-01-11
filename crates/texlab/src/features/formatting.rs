mod bibtex_internal;
mod latexindent;

use crate::util::line_index_ext::LineIndexExt;
use base_db::{Document, Formatter, Workspace};
use distro::Language;
use formatter::{Formatter as TexFormatter, lstgraph::LSTGraph};
use rowan::TextLen;
use syntax::latex;
use tempfile::tempdir;

use self::{bibtex_internal::format_bibtex_internal, latexindent::format_with_latexindent};

pub fn format_source_code(
    workspace: &Workspace,
    uri: &lsp_types::Url,
    options: &lsp_types::FormattingOptions,
) -> Option<Vec<lsp_types::TextEdit>> {
    let document = workspace.lookup(uri)?;
    match document.language {
        Language::Tex => match workspace.config().formatting.tex_formatter {
            Formatter::Null => None,
            Formatter::Server => format_with_texlab(workspace, document, options),
            Formatter::LatexIndent => format_with_latexindent(workspace, document),
        },
        Language::Bib => match workspace.config().formatting.bib_formatter {
            Formatter::Null => None,
            Formatter::Server => format_bibtex_internal(workspace, document, options),
            Formatter::LatexIndent => format_with_latexindent(workspace, document),
        },
        Language::Aux
        | Language::Log
        | Language::Root
        | Language::Latexmkrc
        | Language::Tectonic
        | Language::FileList => None,
    }
}

pub fn format_with_texlab(
    workspace: &Workspace,
    document: &Document,
    options: &lsp_types::FormattingOptions,
) -> Option<Vec<lsp_types::TextEdit>> {
    let root_node = document.data.as_tex()?.root_node();
    let config = workspace.config();
    let line_length = config.formatting.line_length;
    let mut formatter = TexFormatter::new(options.tab_size as usize, line_length);
    let output = formatter.format(&root_node);

    // For troubleshooting
    //let mut lstgraph = LSTGraph::new();
    //let _ = lstgraph.visit(&latex::SyntaxElement::Node(root_node));
    //let path = &document.path;
    //lstgraph.print_graph(path);

    let target_dir = tempdir().ok()?;

    let target_file = target_dir
        .path()
        .join(if document.language == Language::Bib {
            "file.bib"
        } else {
            "file.tex"
        });

    std::fs::write(&target_file, &document.text).ok()?;

    let old_text = &document.text;
    let new_text = output;
    if new_text.is_empty() {
        None
    } else {
        let line_index = &document.line_index;
        let start = lsp_types::Position::new(0, 0);
        let end = line_index.line_col_lsp(old_text.text_len())?;
        Some(vec![lsp_types::TextEdit {
            range: lsp_types::Range::new(start, end),
            new_text,
        }])
    }
}
