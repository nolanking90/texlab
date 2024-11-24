# TODO

1. Read Paper
2. Impl combinators

# Format Preamble and Imports 
- order imports based on https://github.com/mhelvens/latex-pkgloader/blob/master/pkgloader-recommended.sty
- Is it better to organize macros with imports? eg \geometry with \up{geometry}
- Things like Titlesec may get overwritten by new packages?

# Code Actions 

- Refactor Tools
    - Generate sty file
    - extract to new command (new command or renewcommand)
    - identify when commands are being used for snippets instead of formatting

- Fix overful/underful hbox?
- Detect poor style? Like manual numbering or newlines?
- suggest idiomatic solutions
- Detect missing delimeters/environments and autofix?
