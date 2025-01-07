#![allow(dead_code, unused_variables)]
use syntax::latex::SyntaxNode;

pub mod lstgraph;
mod tex;
use tex::TexElement;
mod math;

pub struct FormatContext {
    tabstop: usize,
    indent_level: usize,
    math_mode: bool,
    line_length: usize,
}

pub struct Formatter {
    context: FormatContext,
    doc: TexElement,
}

impl Formatter {
    pub fn new() -> Self {
        Self {
            context: FormatContext {
                tabstop: 2,
                indent_level: 0,
                math_mode: false,
                line_length: 80,
            },
            doc: TexElement::Text(String::new()),
        }
    }

    pub fn format(&mut self, root: &SyntaxNode) -> String {
        self.doc = TexElement::from(root);
        self.doc.format(0, self.context.tabstop, self.context.line_length).join("\n")
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

//#[cfg(test)]
//mod tests;
