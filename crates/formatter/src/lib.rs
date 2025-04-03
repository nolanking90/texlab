#![allow(dead_code)]
use syntax::latex::SyntaxNode;

pub mod lstgraph;
mod tex;
use tex::TexElement;
mod math;
mod formattable;
use crate::formattable::Formattable;

pub struct FormatContext {
    tabstop: usize,
    line_length: usize,
}

pub struct Formatter {
    context: FormatContext,
    doc: TexElement,
}

impl Formatter {
    pub fn new(tabstop: usize, line_length: usize) -> Self {
        Self {
            context: FormatContext {
                tabstop,
                line_length,
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
        Self::new(2, 80)
    }
}

#[cfg(test)]
mod tests;
