//! SC2086 heredoc-body detection (issue #236): a heredoc body is data.

/// Parse a heredoc opener on `line` (`<<EOF`, `<<-EOF`, `<<'EOF'`, `<<"EOF"`).
/// Returns the delimiter and whether it was quoted (body is then fully literal).
/// Here-strings (`<<<`) are not heredocs.
fn heredoc_opener(line: &str) -> Option<(String, bool)> {
    let mut search = 0;
    while let Some(rel) = line[search..].find("<<") {
        let pos = search + rel;
        let rest = &line[pos + 2..];
        if rest.starts_with('<') {
            // here-string `<<<`: skip all three chars
            search = pos + 3;
            continue;
        }
        let rest = rest.strip_prefix('-').unwrap_or(rest).trim_start();
        let (quoted, rest) = match rest.chars().next() {
            Some('\'' | '"') => (true, &rest[1..]),
            _ => (false, rest),
        };
        let delim: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !delim.is_empty() {
            return Some((delim, quoted));
        }
        search = pos + 2;
    }
    None
}

/// For each line of `source`, true when the line is part of a heredoc body
/// where word splitting cannot occur (SC2086 must not fire there). Body lines
/// of an unquoted heredoc that contain a command substitution (`$(` or a
/// backtick) are NOT marked: the substitution body is shell code.
/// The opener line and the terminator line are never marked.
pub fn heredoc_body_lines(source: &str) -> Vec<bool> {
    let mut marks = Vec::new();
    let mut open: Option<(String, bool)> = None;
    for line in source.lines() {
        match &open {
            Some((delim, quoted)) => {
                if line.trim_start_matches('\t').trim_end() == delim.as_str() {
                    marks.push(false);
                    open = None;
                } else {
                    let has_subst = line.contains("$(") || line.contains('`');
                    marks.push(*quoted || !has_subst);
                }
            }
            None => {
                marks.push(false);
                open = heredoc_opener(line);
            }
        }
    }
    marks
}
