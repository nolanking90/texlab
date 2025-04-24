# LaTeX Formatter

This crate provides a $\LaTeX{}$ formatter for the texlab language server. 

## Formatting Example

```latex
\section{Intro}Text here.\begin{itemize}\item One\item Two\end{itemize}
```

becomes:

```latex
\section{Intro}
Text here.
\begin{itemize}
  \item One
  \item Two
\end{itemize}
```

## Usage 

Clone the repository:
```
$ git clone https://github.com/nolanking90/texlab.git
```

Checkout the formatter branch:
```
$ git checkout feature/formatter
```

Compile the project:
```
cargo build
```

Set the texlab setting `texlab.latexFormatter = 'texlab'` in your editor's
configuration.

The following is an example configuration for Neovim using the `lsp-config`
plugin and `cmp`.

```lua
local lspconfig = require('lspconfig')
local capabilities = require('cmp_nvim_lsp').default_capabilities()

lspconfig.texlab.setup {
    cmd = {
        "/Users/username/projects/texlab/target/debug/texlab",
    },
    capabilities = capabilities,
    settings = {
        texlab = {
            latexFormatter = "texlab", -- use this custom formatter
            lineLength = 80,
            chktex = {
                onEdit = true,
                onOpenAndSave = true,
            },
        },
    }
}
```

## Troubleshooting
When creating an issue for problems with the formatter it would be helpful to
submit a unit test demonstrating the desired behavior of the formatter. Existing
unit tests can be found in `tests.rs`.

When somethings goes wrong it's either because the parser has parsed the
document in an unexpected way or because the formatter doesn't handle a
particular type of `SyntaxKind` variant. When the formatter *incorrectly*
formats something, it's generally the formatting rules that are the problem.
If the formatter removes something entirely, it's generally because that code
was attached to a `SyntaxNode` as a `SyntaxToken` and the intermediate
representation did not know to check for this case. This might require
implementing a new `SyntaxKind` in the parser, handling an additional case in
the constructor of the appropriate `TexElement` variant, or implementing a new
`TexElement` variant.

To troubleshoot the former, the formatter can generate an `svg` of the
syntax tree (before an intermediate representation is constructed). Uncomment
the following lines in `texlab/crates/texlab/src/features/formatting.rs`

```rust
//For troubleshooting
let mut lstgraph = LSTGraph::new();
let _ = lstgraph.visit(&latex::SyntaxElement::Node(root_node.clone()));
let path = &document.path;
lstgraph.print_graph(path);
```

This creates `graph.dot` and `graph.svg` using the `dot` tool from Graphviz.
It automatically calls the `dot` executable and compiles dot file into an `svg`.
Visit `https://graphviz.org/` for instructions on installing graphviz. Ensure
the `dot` executable is in your `$PATH`. This tree is generated using the
`petgraph` crate.

For the latter, any of the following could be necessary.
- A `SyntaxKind` variant may need to be added to the `TexElement` constructor
- A new `TexElement` variant with it's own formatting rules may need to be
implemented. Unless the variant corresponds to a single `SyntaxKind` variant it
may be difficult to determine when an instance of the new `TexElement` variant
should be constructed.
- An existing formatting rule in `formattable.rs` may need to be modified.

## Formatter Design

### Parsing 
Texlab uses the Rowan crate to construct a Red/Green tree. The tree consists of
`SyntaxNode`s. Each node has children either of type `SyntaxNode` or
`SyntaxToken`. `SyntaxToken`s store the plain text from the document that
typically is lost during construction of an abstract syntax tree.

### Lowering and Intermediate Representation
The formatter recursively walks the syntax tree and constructs an intermediate
representation (IR). The IR is a decorated syntax tree, where additional
information necessary for determining formatting rules are added to the nodes.
The IR tree is made of `TexElement` variants representing various `latex`
constructions. Environments are separated into standard environments (e.g.
`center`, `figure`, etc.) and math environments. 

```rust
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
    Verbatim(TexVerbatim),
}
```

```rust
pub enum MathElement {
    Parent(MathParent),
    Command(MathCommand),
    Environment(MathEnvironment),
    Text(String),
    CurlyGroup(MathCurlyGroup),
    MixedGroup(MathMixedGroup),
    BrackGroup(MathBrackGroup),
}
```

Each variant is a tuple variant containing a struct representing the latex
construction and the information necessary to format it. The struct definitions
and their constructors are found in `tex.rs` and `math.rs`. 

### Formatting Rules and Code Generation
Each struct implements the `Formattable` trait, which defines a common signature
for a `format` function.
```rust
pub trait Formattable {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String>;
}
```
The `format` function returns a `Vec` of strings representing lines of the
formatted document. Each struct determines the "internal" formatting of the code
it represents, and `TexParent` nodes determine how to layout adjacent document
elements. That is, `TexParent` determines if elements should be concatenated,
separated with a space, separated with a `~`, or if a line should be wrapped.
Some line wrapping also takes place in the `format` implementation of other
structs with their own children, such as `TexEnvironment` and `TexCurlyGroup`.
Implementations of `Formattable` are contained in `formattable.rs`.

## Roadmap
We have the following goals before the fork is merged upstream.
- Greater configurability, e.g. specifying if `$ math $` and `$$ math $$`
should be replaced with `\( \)` and `\[ \]`.
- Stateless formatting. Currently the formatter progressively builds up a vector
of strings representing the formatted document. This resembles the use of an
accumulator, and in the future should be replaced with an actual accumulator to
remove and manual managing of document state.
- Increased modularity. The current formatting logic is over complicated and
many cases are handled inside the `TexParent::format` function. It would be
easier to add new rules and handle special cases if formatting rules were
refactored. This may require changing the intermediate representation to use
combinators like typical pretty printers (see
[here]{https://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf}).
- More tests.
- Benchmarks against other latex formatters (e.g. latexindent).
