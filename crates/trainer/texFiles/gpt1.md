```latex
\usepackage{amsthm}
\usepackage{hyperref}
\usepackage{biblatex}
\newtheorem{theorem}{Theorem}
\addbibresource{references.bib}
\newcommand{\R}{\mathbb{R}}
\frac{a}{b}
\sqrt{a^2 + b^2}
\sum_{n=1}^{\infty} \frac{1}{n^2}
\int_{0}^{1} x^2 \, dx
\begin{pmatrix}
a & b \\
c & d
\end{pmatrix}
\begin{align}
a &= b + c \\
d &= e - f
\end{align}
\href{https://www.openai.com}{OpenAI Website}
\printbibliography
As shown in \cite{einstein1905}.
\begin{theorem}
Every even integer greater than 2 can be expressed as the sum of two primes.
\end{theorem}
\begin{proof}
This is the proof of the theorem.
\end{proof}
```

---

### **37. Using Colors**

```latex
\usepackage{color}

\textcolor{red}{This text is red.}
```

---

### **38. Changing Font Size**

```latex
{\large This is large text.}
```

---

### **39. Creating a Block Quote**

```latex
\begin{quote}
This is a block quote.
\end{quote}
```

---

### **40. Adding a Line Break**

```latex
First line.\\
Second line.
```

---

### **41. Creating a Page Break**

```latex
\newpage
```

---

### **42. Adding a Header and Footer**

```latex
\usepackage{fancyhdr}
\pagestyle{fancy}
\fancyhf{}
\fancyhead[L]{Left Header}
\fancyhead[R]{Right Header}
\fancyfoot[C]{\thepage}
```

---

### **43. Creating a List with Custom Labels**

```latex
\begin{itemize}
    \item[$\star$] Starred item
    \item[$\bullet$] Bullet item
\end{itemize}
```

---

### **44. Nesting Lists**

```latex
\begin{enumerate}
    \item First item
    \begin{itemize}
        \item Subitem
    \end{itemize}
    \item Second item
\end{enumerate}
```

---

### **45. Creating a Minipage**

```latex
\begin{minipage}{0.5\textwidth}
This is a minipage.
\end{minipage}
```

---

### **46. Using the TikZ Package for Graphics**

```latex
\usepackage{tikz}

\begin{tikzpicture}
    \draw (0,0) -- (1,1);
\end{tikzpicture}
```

---

### **47. Creating a Code Block with Listings Package**

```latex
\usepackage{listings}

\begin{lstlisting}[language=Python]
def hello_world():
    print("Hello, World!")
\end{lstlisting}
```

---

### **48. Adding Comments**

```latex
% This is a comment and will not appear in the document
```

---

### **49. Using the Multirow Package in Tables**

```latex
\usepackage{multirow}

\begin{tabular}{|c|c|}
\hline
\multirow{2}{*}{A} & B \\
\cline{2-2}
                   & C \\
\hline
\end{tabular}
```

---

### **50. Creating a Sideways Table with Rotating Package**

```latex
\usepackage{rotating}

\begin{sidewaystable}
\centering
\begin{tabular}{|c|c|}
\hline
A & B \\
\hline
1 & 2 \\
\hline
\end{tabular}
\caption{Rotated Table}
\end{sidewaystable}
```

---

### **51. Using the Caption Package**

```latex
\usepackage{caption}

\captionsetup{font=small, labelfont=bf}
```

---

### **52. Creating a Float Barrier with Placeins Package**

```latex
\usepackage{placeins}

\FloatBarrier
```

---

### **53. Creating a Landscape Page with lscape Package**

```latex
\usepackage{lscape}

\begin{landscape}
Content in landscape mode.
\end{landscape}
```

---

### **54. Adding a Watermark with draftwatermark Package**

```latex
\usepackage{draftwatermark}

\SetWatermarkText{DRAFT}
\SetWatermarkScale{1}
```

---

### **55. Creating a Cover Page**

```latex
\begin{titlepage}
    \centering
    {\Huge\bfseries My Thesis Title\par}
    \vspace{2cm}
    {\Large Author Name\par}
    \vfill
    {\large \today\par}
\end{titlepage}
```

---

### **56. Including External Documents with Subfiles Package**

```latex
\usepackage{subfiles}

\subfile{chapter1.tex}
```

---

### **57. Creating a Bibliography Entry in BibTeX**

```bibtex
@book{knuth1984,
  title={The TeXbook},
  author={Knuth, Donald E},
  year={1984},
  publisher={Addison-Wesley}
}
```

---

### **58. Adding a URL with Hyperref Package**

```latex
\usepackage{hyperref}

\url{https://www.openai.com}
```

---

### **59. Creating a Label and Referencing It**

```latex
\label{sec:intro}

As discussed in Section~\ref{sec:intro}, ...
```

---

### **60. Creating a Custom Color**

```latex
\usepackage{xcolor}

\definecolor{myblue}{RGB}{0, 0, 255}

\textcolor{myblue}{Custom blue text}
```

---

### **61. Using the Fancyhdr Package for Custom Headers**

```latex
\usepackage{fancyhdr}
\pagestyle{fancy}
\fancyhead[L]{Left Header}
\fancyhead[C]{Center Header}
\fancyhead[R]{Right Header}
```

---

### **62. Creating a Double Column Document**

```latex
\documentclass[twocolumn]{article}
```

---

### **63. Using the Multicol Package for Multiple Columns**

```latex
\usepackage{multicol}

\begin{multicols}{3}
Text in three columns.
\end{multicols}
```

---

### **64. Adding a Drop Cap with lettrine Package**

```latex
\usepackage{lettrine}

\lettrine{A}{ beginning} of the paragraph with a drop cap.
```

---

### **65. Creating a Glossary with glossaries Package**

```latex
\usepackage{glossaries}

\makeglossaries

\newglossaryentry{latex}{
    name=LaTeX,
    description={A typesetting system}
}

...

\printglossaries
```

---

### **66. Adding Index with makeidx Package**

```latex
\usepackage{makeidx}
\makeindex

...

\index{LaTeX}

...

\printindex
```

---

### **67. Creating a Colored Box with tcolorbox Package**

```latex
\usepackage{tcolorbox}

\begin{tcolorbox}[colback=blue!5!white,colframe=blue!75!black]
This is a colored box.
\end{tcolorbox}
```

---

### **68. Using the Siunitx Package for Units**

```latex
\usepackage{siunitx}

The length is \SI{10}{\meter}.
```

---

### **69. Creating a Beamer Presentation**

```latex
\documentclass{beamer}

\begin{document}

\begin{frame}
    \frametitle{Introduction}
    Welcome to the presentation.
\end{frame}

\end{document}
```

---

### **70. Adding a Logo to Beamer**

```latex
\usepackage{graphicx}

\logo{\includegraphics[height=1cm]{logo.png}}
```

---

### **71. Creating a Slide with Columns in Beamer**

```latex
\begin{frame}{Two Columns}
    \begin{columns}
        \column{0.5\textwidth}
        Left column content.

        \column{0.5\textwidth}
        Right column content.
    \end{columns}
\end{frame}
```

---

### **72. Using Themes in Beamer**

```latex
\usetheme{Madrid}
```

---

### **73. Adding Transitions in Beamer**

```latex
\transfade
```

---

### **74. Creating a Frame with a Title in Beamer**

```latex
\begin{frame}{Frame Title}
    Frame content goes here.
\end{frame}
```

---

### **75. Using the Xcolor Package for Extended Colors**

```latex
\usepackage[table]{xcolor}

\rowcolors{1}{gray!25}{white}
```

---

### **76. Creating a Custom Footer with Fancyhdr**

```latex
\fancyfoot[L]{Left Footer}
\fancyfoot[C]{Center Footer}
\fancyfoot[R]{\thepage}
```

---

### **77. Inserting an External PDF Page with pdfpages Package**

```latex
\usepackage{pdfpages}

\includepdf[pages=-]{document.pdf}
```

---

### **78. Creating a Rotated Text with rotating Package**

```latex
\usepackage{rotating}

\begin{sideways}
Rotated text.
\end{sideways}
```

---

### **79. Using the Booktabs Package for Tables**

```latex
\usepackage{booktabs}

\begin{tabular}{ll}
\toprule
Header1 & Header2 \\
\midrule
Data1 & Data2 \\
\bottomrule
\end{tabular}
```

---

### **80. Creating a Landscape Table with adjustbox Package**

```latex
\usepackage{adjustbox}

\begin{table}
\centering
\adjustbox{angle=90}{
    \begin{tabular}{|c|c|}
    \hline
    A & B \\
    \hline
    1 & 2 \\
    \hline
    \end{tabular}
}
\caption{Rotated Table}
\end{table}
```

---

### **81. Adding a Background Image with wallpaper Package**

```latex
\usepackage{wallpaper}

\CenterWallPaper{1.0}{background.jpg}
```

---

### **82. Creating a Custom Header Line**

```latex
\usepackage{fancyhdr}

\pagestyle{fancy}
\fancyhead{} % clear all header fields
\fancyhead[L]{Left Header}
\fancyhead[C]{Center Header}
\fancyhead[R]{Right Header}
```

---

### **83. Using the Subcaption Package for Subfigures**

```latex
\usepackage{subcaption}

\begin{figure}
    \centering
    \begin{subfigure}{0.45\textwidth}
        \includegraphics[width=\linewidth]{image1.png}
        \caption{First Image}
    \end{subfigure}
    \begin{subfigure}{0.45\textwidth}
        \includegraphics[width=\linewidth]{image2.png}
        \caption{Second Image}
    \end{subfigure}
    \caption{Two Subfigures}
\end{figure}
```

---

### **84. Creating a Custom Environment**

```latex
\newenvironment{customenv}
{
    \begin{center}
    \begin{tabular}{|c|}
    \hline
}
{
    \hline
    \end{tabular}
    \end{center}
}
```

---

### **85. Using the Wrapfig Package for Wrapped Figures**

```latex
\usepackage{wrapfig}

\begin{wrapfigure}{r}{0.3\textwidth}
    \centering
    \includegraphics[width=0.28\textwidth]{image.png}
    \caption{Wrapped Figure}
\end{wrapfigure}
```

---

### **86. Creating a Split Equation with Split Environment**

```latex
\begin{equation}
\begin{split}
a &= b + c \\
  &= d + e
\end{split}
\end{equation}
```

---

### **87. Using the Hyperref Package for PDF Metadata**

```latex
\usepackage{hyperref}

\hypersetup{
    pdftitle={My Document},
    pdfauthor={John Doe},
    pdfsubject={LaTeX Examples},
    pdfkeywords={LaTeX, examples, snippets}
}
```

---

### **88. Creating a Colored Page Background with Pagecolor Package**

```latex
\usepackage{pagecolor}

\pagecolor{yellow!10}
```

---

### **89. Adding a Frame to Text with mdframed Package**

```latex
\usepackage{mdframed}

\begin{mdframed}
This text is inside a frame.
\end{mdframed}
```

---

### **90. Creating a Side Note with marginnote Package**

```latex
\usepackage{marginnote}

This is some text.\marginnote{This is a side note.}
```

---

### **91. Using the Cleveref Package for Enhanced References**

```latex
\usepackage{cleveref}

As shown in \cref{fig:sample}, ...
```

---

### **92. Creating a Custom Title Style with titlesec Package**

```latex
\usepackage{titlesec}

\titleformat{\section}
  {\normalfont\Large\bfseries}{\thesection}{1em}{}
```

---

### **93. Adding Line Numbers with lineno Package**

```latex
\usepackage{lineno}

\linenumbers
```

---

### **94. Creating a Shadowed Box with shadowbox Package**

```latex
\usepackage{shadowbox}

\shadowbox{This is a shadowed box.}
```

---

### **95. Using the Float Package for Custom Float Positions**

```latex
\usepackage{float}

\begin{figure}[H]
    \centering
    \includegraphics{image.png}
    \caption{Fixed Position Figure}
\end{figure}
```

---

### **96. Creating a Titled Box with tcolorbox Package**

```latex
\usepackage{tcolorbox}

\begin{tcolorbox}[title=My Box]
This is a titled box.
\end{tcolorbox}
```

---

### **97. Adding a Signature Line**

```latex
\noindent\rule{6cm}{0.4pt} \\
Signature
```

---

### **98. Creating a Page Header with Page Style**

```latex
\pagestyle{headings}
```

---

### **99. Using the Datatool Package for Data Management**

```latex
\usepackage{datatool}

\DTLloaddb{mydata}{data.csv}
```
```latex
\bibliographystyle{apalike}
\bibliography{references}
```
