use crate::math::*;
use crate::tex::*;

pub fn indent_str(indent_level: usize, tabstop: usize) -> String {
    " ".repeat(indent_level * tabstop)
}

pub trait Formattable {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String>;
}

// Helper function to flush line and extend output with formatted element
fn flush_and_extend<T: Formattable>(
    output: &mut Vec<String>,
    line: &mut String,
    element: &T,
    indent_level: usize,
    tabstop: usize,
    line_length: usize,
) {
    if !line.is_empty() {
        output.push(line.clone());
        *line = String::new();
    }
    output.extend(element.format(indent_level, tabstop, line_length));
}

// Helper function to format group content with proper indentation
fn format_group_content(
    body: &impl Formattable,
    indent_level: usize,
    tabstop: usize,
    line_length: usize,
    open_delim: &str,
    close_delim: &str,
) -> Vec<String> {
    let mut output = Vec::new();
    
    // Try inline format first
    let bodytext = body.format(indent_level, tabstop, line_length);
    if bodytext.iter().map(|line| line.len()).sum::<usize>() <= line_length {
        output.push(format!("{}{}{}", open_delim, bodytext.join("").trim(), close_delim));
        return output;
    }
    
    // Format with proper indentation
    let bodytext = body.format(indent_level + 1, tabstop, line_length);
    let indent = indent_str(indent_level + 1, tabstop);
    
    output.push(open_delim.to_string());
    output.extend(
        bodytext
            .iter()
            .map(|line| format!("{indent}{}", line.trim())),
    );
    output.push(close_delim.to_string());
    
    output
}

// Helper function to handle spacing after elements in parent containers
fn handle_next_element_spacing(line: &mut String, next_element: Option<&impl ElementType>, current_ends_with_brace: bool) {
    match next_element {
        Some(element) => {
            if !should_skip_space_after(element, line) {
                if current_ends_with_brace {
                    line.push(' ');
                } else {
                    line.push('~');
                }
            }
        },
        None => {}
    }
}

// Trait to represent either TexElement or MathElement for spacing logic
trait ElementType {
    fn first_char(&self) -> Option<char>;
    fn element_type(&self) -> &'static str;
}

impl ElementType for TexElement {
    fn first_char(&self) -> Option<char> {
        match self {
            TexElement::Text(txt) => txt.chars().next(),
            _ => None,
        }
    }
    
    fn element_type(&self) -> &'static str {
        match self {
            TexElement::Command(_) => "command",
            TexElement::Text(_) => "text",
            TexElement::Formula(_) => "formula",
            _ => "other",
        }
    }
}

impl ElementType for MathElement {
    fn first_char(&self) -> Option<char> {
        match self {
            MathElement::Text(txt) => txt.chars().next(),
            _ => None,
        }
    }
    
    fn element_type(&self) -> &'static str {
        match self {
            MathElement::Command(_) => "command",
            MathElement::Text(_) => "text",
            _ => "other",
        }
    }
}

// Helper function to determine if space should be skipped after an element
fn should_skip_space_after<T: ElementType>(element: &T, current_line: &str) -> bool {
    match element.element_type() {
        "text" => {
            match element.first_char() {
                Some('}') | Some(')') | Some(']') | Some('.') | Some(',') | 
                Some(';') | Some('~') | Some('^') | Some('_') | Some('\'') => true,
                _ => false,
            }
        },
        "command" => false,
        _ => false,
    }
}

// Helper function to handle word wrapping for text elements
fn wrap_text(text: &str, line: &mut String, output: &mut Vec<String>, line_length: usize) {
    if text.trim().len() + line.len() < line_length {
        line.push_str(&format!("{} ", text.trim()));
    } else {
        let words = text.trim().split_whitespace();
        for word in words {
            if line.len() + word.len() >= line_length {
                output.push(line.trim().to_string());
                *line = String::new();
            }
            line.push_str(&format!("{word} "));
        }
    }
}

impl Formattable for TexElement {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        match self {
            TexElement::Command(cmd) => cmd.format(indent_level, tabstop, line_length),
            TexElement::Environment(env) => env.format(indent_level, tabstop, line_length),
            TexElement::Text(text) => vec![text.to_string().trim().to_string()],
            TexElement::Parent(p) => p.format(indent_level, tabstop, line_length),
            TexElement::KeyValPair(kvp) => kvp.format(indent_level, tabstop, line_length),
            TexElement::CurlyGroup(group) => group.format(indent_level, tabstop, line_length),
            TexElement::BrackGroup(group) => group.format(indent_level, tabstop, line_length),
            TexElement::CurlyGroupWordList(group) => {
                group.format(indent_level, tabstop, line_length)
            }
            TexElement::MixedGroup(group) => group.format(indent_level, tabstop, line_length),
            TexElement::EnumItem(item) => item.format(indent_level, tabstop, line_length),
            TexElement::Formula(formula) => formula.format(indent_level, tabstop, line_length),
            TexElement::Section(section) => section.format(indent_level, tabstop, line_length),
            TexElement::Comment(comment) => comment.format(indent_level, tabstop, line_length),
            TexElement::Blankline => vec!["\n\n".to_string()],
            TexElement::KeyValBody(keyval) => keyval.format(indent_level, tabstop, line_length),
        }
    }
}

impl Formattable for TexParent {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();
        let indent = indent_str(indent_level, tabstop);
        let mut line = String::new();
        let mut l = line.len();
        
        for (i, child) in self.children.iter().enumerate() {
            if line.len() > line_length {
                output.push(line);
                line = String::new();
            }
            l = line.len();
            
            match child {
                TexElement::Command(cmd) => {
                    let cmdln = cmd.format(0, 0, 0).join("").len();
                    if line.len() + cmdln >= line_length && !line.is_empty() {
                        line = line.trim_end().to_string();
                        if !line.ends_with('%') {
                            line.push('%');
                        }
                        output.push(line.trim().to_string());
                        line = String::new();
                    }

                    if cmd.start_newline {
                        flush_and_extend(
                            &mut output,
                            &mut line,
                            cmd,
                            indent_level,
                            tabstop,
                            line_length,
                        );
                        continue;
                    }

                    if cmd.name.as_str() == "\\\\" {
                        line.push_str("\\\\");
                        output.push(line.trim().to_string());
                        line = String::new();
                        l = line.len();
                        continue;
                    }

                    let fmt = cmd.format(indent_level, tabstop, line_length - line.len());
                    if fmt.len() == 1 && line.len() + fmt[0].len() <= line_length {
                        line.push_str(fmt[0].clone().trim());
                        if let Some(next_elem) = self.children.get(i + 1) {
                            handle_next_element_spacing(&mut line, Some(next_elem), line.ends_with('}'));
                        }
                        continue;
                    }

                    line.push_str(fmt[0].clone().trim());

                    if fmt.len() == 1 {
                        continue;
                    }

                    output.push(line);
                    line = String::new();
                    output.extend(
                        fmt[1..]
                            .iter()
                            .cloned()
                            .map(|line| format!("{indent}{line}")),
                    );
                }
                TexElement::Text(text) if !text.trim().is_empty() => {
                    if line_length == 0 {
                        line.push_str(text);
                        continue;
                    }
                    
                    wrap_text(text, &mut line, &mut output, line_length);
                    
                    if let Some(TexElement::Command(_)) = self.children.get(i + 1) {
                        if line.ends_with("~ ") {
                            line = line.trim().to_string();
                        }
                    }
                }
                TexElement::Environment(env) => {
                    flush_and_extend(
                        &mut output,
                        &mut line,
                        env,
                        indent_level,
                        tabstop,
                        line_length,
                    );
                }
                TexElement::Parent(p) => {
                    flush_and_extend(
                        &mut output,
                        &mut line,
                        p,
                        indent_level,
                        tabstop,
                        line_length,
                    );
                }
                TexElement::KeyValPair(_) => {}
                TexElement::KeyValBody(k) => {
                    line.push_str(&k.format(indent_level, tabstop, line_length).join(""));
                }
                TexElement::CurlyGroup(group) => {
                    let fmt = group.format(indent_level, tabstop, line_length - line.len());
                    line.push_str(&fmt[0]);
                    output.push(line);
                    line = String::new();
                    output.extend(fmt[1..].iter().cloned());
                }
                TexElement::BrackGroup(group) => {
                    line.push_str(&group.format(indent_level, tabstop, line_length).join(""));
                }
                TexElement::CurlyGroupWordList(group) => {
                    line.push_str(&group.format(indent_level, tabstop, line_length).join(""));
                }
                TexElement::MixedGroup(group) => {
                    line.push_str(
                        group
                            .format(indent_level, tabstop, line_length)
                            .join("")
                            .trim(),
                    );
                    if let Some(next_elem) = self.children.get(i + 1) {
                        if let TexElement::Text(txt) = next_elem {
                            if !matches!(
                                txt.chars().next(),
                                Some('.') | Some(',') | Some('^') | Some('_') | Some('-')
            ) {
                line.push(' ');
            }
        }
        if let MathElement::Command(_) = next_elem {
            line.push(' ');
        }
    }
}

impl Formattable for MathBrackGroup {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = Vec::new();
        for child in self.children.iter() {
            output.extend(child.format(indent_level, tabstop, line_length));
        }
        vec![format!("[{}]", output.join(", "))]
    }
}

impl Formattable for MathCurlyGroup {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let bodytext = self.body.format(indent_level, tabstop, line_length);
        let output: Vec<String> = bodytext.iter().map(|line| line.trim().to_string()).collect();
        
        vec![
            "{".to_string(),
            output.join(" ").to_string(),
            "}".to_string(),
        ]
    }
}

impl Formattable for MathMixedGroup {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = String::new();
        output.push_str(&self.open_delim);
        for child in self.children.iter() {
            output.push_str(
                child
                    .format(indent_level, tabstop, line_length)
                    .join("")
                    .trim(),
            );
        }
        output.push_str(&self.close_delim);
        vec![output]
    }
}

impl Formattable for MathCommand {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = String::new();
        output.push_str(&self.name);
        for arg in self.args.iter() {
            output.push_str(
                arg.format(indent_level, tabstop, line_length)
                    .join("")
                    .trim(),
            );
        }
        if self.name.as_str() != "\\\\" {
            output.push(' ');
        }
        vec![output]
    }
}

impl Formattable for MathEnvironment {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut line = String::new();
        let mut lines = Vec::new();
        let indent = indent_str(indent_level, tabstop);

        line.push_str(&format!("{}\\begin{{{}}}", indent, self.name));

        for arg in self.args.iter() {
            line.push_str(&arg.format(indent_level, tabstop, line_length).join(""));
        }

        lines.push(line);

        lines.extend(self.align_at_amp(indent_level + 1, tabstop, line_length));

        lines.push(format!("{}\\end{{{}}}", indent, self.name));

        lines
    }
}')
                            ) {
                                line.push(' ');
                            }
                        }
                    }
                }
                TexElement::EnumItem(item) => {
                    if !line.trim().is_empty() {
                        output.push(line.trim().to_string());
                    }
                    line = String::new();
                    l = line.len();
                    output.extend(item.format(indent_level, tabstop, line_length - line.len()));
                }
                TexElement::Formula(formula) => {
                    if formula.inline {
                        let fmt = formula.format(indent_level, tabstop, line_length);
                        if fmt.join("").len() + line.len() > line_length {
                            output.push(line);
                            line = String::new();
                            if fmt.len() == 1 {
                                line.push_str(fmt.join("").trim());
                                if let Some(next_elem) = self.children.get(i + 1) {
                                    if let TexElement::Text(txt) = next_elem {
                                        if !matches!(
                                            txt.chars().next(),
                                            Some('.') | Some(',') | Some('?') | Some(';') | Some('-')
                                        ) {
                                            line.push(' ');
                                        }
                                    }
                                    if let TexElement::Formula(_) = next_elem {
                                        line.push(' ');
                                    }
                                }
                            } else {
                                output.extend(fmt);
                            }
                        } else {
                            line.push_str(fmt.join("").trim());
                            if let Some(next_elem) = self.children.get(i + 1) {
                                if let TexElement::Text(txt) = next_elem {
                                    if !matches!(
                                        txt.chars().next(),
                                        Some('.') | Some(',') | Some('?') | Some(';') | Some('-')
                                    ) {
                                        line.push(' ');
                                    }
                                }
                                if let TexElement::Formula(_) = next_elem {
                                    line.push(' ');
                                }
                            }
                        }
                    } else {
                        flush_and_extend(
                            &mut output,
                            &mut line,
                            formula,
                            indent_level,
                            tabstop,
                            line_length,
                        );
                    }
                }
                TexElement::Section(section) => {
                    flush_and_extend(
                        &mut output,
                        &mut line,
                        section,
                        indent_level,
                        tabstop,
                        line_length,
                    );
                }
                TexElement::Comment(comment) => {
                    line.push_str(&comment.format(indent_level, tabstop, line_length)[0]);
                    output.push(line);
                    line = indent_str(indent_level, tabstop);
                }
                TexElement::Blankline => {
                    if !line.is_empty() {
                        output.push(line);
                    }
                    output.push(String::new());
                    line = String::new();
                }
                _ => {}
            }
        }

        if line.trim().is_empty() {
            return output;
        }

        if line.len() < line_length {
            output.push(line.trim().to_string());
        } else {
            output.push(line[0..l].to_string());
            output.push(format!(
                "{}{}",
                indent_str(0, tabstop),
                &line[l..].to_string()
            ));
        }
        output
    }
}

impl Formattable for TexCommand {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = Vec::new();
        let mut fmt = String::new();
        fmt.push_str(&self.name);
        for arg in self.args.iter() {
            if line_length == 0 {
                fmt.push_str(
                    arg.format(indent_level, tabstop, line_length)
                        .join("")
                        .trim_end(),
                );
            } else {
                let ln = arg.format(indent_level, tabstop, line_length - self.name.len());
                fmt.push_str(ln.first().unwrap().trim());
                if ln.len() > 1 {
                    output.push(fmt);
                    output.extend(ln[1..ln.len() - 1].iter().cloned());
                    fmt = ln[ln.len() - 1].clone().trim().to_string();
                }
            }
        }
        if !fmt.is_empty() {
            output.push(fmt);
        }
        output
    }
}

impl Formattable for TexCurlyGroup {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        format_group_content(&self.body, indent_level, tabstop, line_length, "{", "}")
    }
}

impl Formattable for TexBrackGroup {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        format_group_content(&self.body, indent_level, tabstop, line_length, "[", "]")
    }
}

impl Formattable for TexMixedGroup {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = String::new();
        output.push_str(&self.open_delim);
        for child in self.children.iter() {
            output.push_str(
                child
                    .format(indent_level, tabstop, line_length)
                    .join("")
                    .split(',')
                    .collect::<Vec<&str>>()
                    .join(", ")
                    .to_string()
                    .trim(),
            );
        }
        output.push_str(&self.close_delim);
        vec![output]
    }
}

impl Formattable for TexCurlyGroupWordList {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = Vec::new();
        for child in self.children.iter() {
            output.extend(child.format(indent_level, tabstop, line_length));
        }
        vec![format!("{{{}}}", output.join(", ").trim().to_string())]
    }
}

impl Formattable for TexEnvironmentParent {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        match self {
            TexEnvironmentParent::Tex(tex) => tex.format(indent_level, tabstop, line_length),
            TexEnvironmentParent::Math(math) => math.format(indent_level, tabstop, line_length),
        }
    }
}

impl Formattable for TexEnvironment {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut output = String::new();
        let indent = indent_str(indent_level, tabstop);
        let mut subindent = 0;
        if !matches!(self.name.as_str(), "document" | "verbatim") {
            subindent = 1;
        }

        output.push_str(&format!("{}\\begin{{{}}}", indent, self.name));

        for arg in self.args.iter() {
            output.push_str(
                arg.format(indent_level, tabstop, line_length)
                    .join("")
                    .trim(),
            );
        }

        lines.push(output);

        lines.extend(
            self.body
                .format(indent_level, tabstop, line_length)
                .iter()
                .map(|line| format!("{}{}", indent_str(subindent, tabstop), line.trim_end())),
        );

        lines.push(format!("{}\\end{{{}}}", indent, self.name));

        lines
    }
}

impl Formattable for TexKeyValParent {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let indent = indent_str(indent_level, tabstop);
        let mut result = String::new();
        let len = self.children.len();
        let newline = format!("\n{}", indent);
        for i in 0..len {
            let child_str = self.children[i].format(indent_level, tabstop, line_length)[0].clone();
            result.push_str(&child_str);
            if i + 1 < len {
                let delimiter = match (&self.children[i], &self.children[i + 1]) {
                    (TexElement::KeyValPair(_), TexElement::Comment(_)) => ", ",
                    (TexElement::Comment(_), TexElement::KeyValPair(_)) => &newline,
                    _ => ", ",
                };
                result.push_str(delimiter);
            }
        }
        vec![result]
    }
}

impl Formattable for TexKeyVal {
    fn format(&self, _indent_level: usize, _tabstop: usize, _line_length: usize) -> Vec<String> {
        match &self.value {
            Some(val) => vec![format!("{} = {val}", self.key)],
            None => vec![format!("{}", self.key)],
        }
    }
}

impl Formattable for TexEnumItem {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let output = self.body.format(indent_level, tabstop, line_length - 6);
        let mut space = String::new();
        if output.first().unwrap().chars().next() != Some('[') {
            space.push(' ');
        }
        let mut fmt = vec![format!(
            "\\item{}{}",
            space,
            output.first().unwrap().trim_end()
        )];
        let indent = indent_str(indent_level + 1, tabstop);
        fmt.extend(
            output[1..]
                .iter()
                .cloned()
                .map(|line| format!("{indent}{line}")),
        );
        fmt
    }
}

impl Formattable for TexFormula {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = Vec::new();
        let mut subindent = 0;
        let indent = indent_str(indent_level, tabstop);
        
        if self.inline {
            output.push("\\( ".to_string());
        } else {
            output.push(format!("{indent}\\["));
            subindent = 1;
        }

        if self.inline {
            format_inline_formula(self, &mut output, indent_level, tabstop, line_length)
        } else {
            let fmt = self.body.format(indent_level + 1, tabstop, line_length);
            output.extend(fmt);
            output.push(format!("{indent}\\]"));
            output
        }
    }
}

// Helper function for formatting inline formula
fn format_inline_formula(
    formula: &TexFormula,
    output: &mut Vec<String>,
    indent_level: usize,
    tabstop: usize,
    line_length: usize,
) -> Vec<String> {
    let fmt = formula.body.format(indent_level, tabstop, line_length);
    
    if fmt.join("").trim().len() + 6 <= line_length {
        output.push(fmt.join("").trim().to_string());
        output.push(" \\)".to_string());
        vec![output.join("").trim().to_string()]
    } else {
        let fmt = formula.body.format(indent_level + 1, tabstop, line_length);
        output.extend(fmt);
        output.push("\\)".to_string());
        output.clone()
    }
}

impl Formattable for TexSection {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = Vec::new();
        output.push(String::new());
        output.push(format!("{}{{{}}}", self.kind, self.name));
        output.extend(self.body.format(indent_level, tabstop, line_length));
        output
    }
}

impl Formattable for TexComment {
    fn format(&self, _indent_level: usize, _tabstop: usize, _line_length: usize) -> Vec<String> {
        vec![self.comment.clone()]
    }
}

impl Formattable for MathParent {
    fn format(&self, indent_level: usize, tabstop: usize, line_length: usize) -> Vec<String> {
        let mut output = Vec::new();
        let mut line = String::new();
        
        for (i, child) in self.children.iter().enumerate() {
            match child {
                MathElement::Command(cmd) => {
                    handle_math_command(cmd, &mut line, &mut output, i, &self.children, indent_level, tabstop, line_length);
                }
                MathElement::Text(text) if !(text.trim().is_empty()) => {
                    line.push_str(text.trim());
                    if i != self.children.len() - 1 && !line.ends_with('^') {
                        if let Some(MathElement::MixedGroup(_)) = self.children.get(i + 1) {
                            // No space needed
                        } else if let Some(MathElement::Text(txt)) = self.children.get(i + 1) {
                            if !matches!(txt.chars().next(), Some('^'))
                                && (!line.ends_with('&')
                                    || !matches!(txt.chars().next(), Some('=')))
                            {
                                line.push(' ');
                            }
                        } else {
                            line.push(' ');
                        }
                    }
                }
                MathElement::Environment(env) => {
                    if !line.trim().is_empty() {
                        output.push(line);
                        line = String::new();
                    }
                    output.extend(env.format(indent_level, tabstop, line_length));
                }
                MathElement::Parent(p) => {
                    if !line.trim().is_empty() {
                        output.push(line.to_string());
                        line = String::new();
                    }
                    let fmt = p.format(indent_level, tabstop, line_length);
                    if !fmt.join("").trim().is_empty() {
                        output.extend(fmt);
                    }
                }
                MathElement::CurlyGroup(group) | MathElement::BrackGroup(group) | MathElement::MixedGroup(group) => {
                    handle_math_group(group, &mut line, i, &self.children, indent_level, tabstop, line_length);
                }
                _ => {}
            }
        }
        
        if !line.trim().is_empty() {
            output.push(line.trim_end().to_string());
        }
        
        output
            .iter()
            .map(|line| format!("{}{}", indent_str(indent_level, tabstop), line))
            .collect()
    }
}

// Helper function for handling math commands in MathParent
fn handle_math_command(
    cmd: &MathCommand,
    line: &mut String,
    output: &mut Vec<String>,
    index: usize,
    children: &[MathElement],
    indent_level: usize,
    tabstop: usize,
    line_length: usize,
) {
    let fmt = cmd
        .format(indent_level, tabstop, line_length - line.len())
        .join("")
        .trim()
        .to_string();
        
    if fmt.len() + line.len() > line_length {
        if !line.trim().is_empty() {
            output.push(line.clone());
            *line = String::new();
        }
        let fmt = cmd.format(indent_level, tabstop, line_length);
        if fmt.len() == 1 {
            line.push_str(fmt[0].trim_end());
        } else {
            output.extend(fmt[..fmt.len() - 1].to_vec());
            line.push_str(&fmt[fmt.len() - 1]);
        }
    } else {
        line.push_str(&fmt);
    }
    
    if line.ends_with("\\\\") {
        output.push(line.trim().to_string());
        *line = String::new();
    } else {
        if let Some(next_elem) = children.get(index + 1) {
            if let MathElement::Text(txt) = next_elem {
                if !matches!(
                    txt.chars().next(),
                    Some(',') | Some('^') | Some('_') | Some('\'') | Some('.')
                ) {
                    line.push(' ');
                }
            }
            if let MathElement::Command(_) = next_elem {
                line.push(' ');
            }
        }
    }
}

// Helper function for handling math groups in MathParent
fn handle_math_group(
    group: &impl Formattable,
    line: &mut String,
    index: usize,
    children: &[MathElement],
    indent_level: usize,
    tabstop: usize,
    line_length: usize,
) {
    line.push_str(
        group
            .format(indent_level, tabstop, line_length)
            .join("")
            .trim(),
    );
    
    if let Some(next_elem) = children.get(index + 1) {
        if let MathElement::Text(txt) = next_elem {
            if !matches!(
                txt.chars().next(),
                Some('.') | Some(',') | Some('^') | Some('_') | Some('-