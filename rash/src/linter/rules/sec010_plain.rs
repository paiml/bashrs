//! SEC010 (GH-528): child-shell positional parameters in a "plain" file.
//! In `sh -c '... "$1" ...'` the `$1` is the child's own parameter. Hand-reading shell
//! quoting is error-prone, so the mask applies only when the WHOLE file is "plain"
//! (none of the constructs below); any other file is linted exactly as before.

/// Substrings that make a file not plain, wherever they appear.
const NOT_PLAIN: &[&str] = &["`", "<<", "$(", "${", "$'", "$\"", "\\", "\r"];

/// True when no quoting construct in the whole file can be misread.
pub(super) fn is_plain(source: &str) -> bool {
    if !source.is_ascii() || NOT_PLAIN.iter().any(|p| source.contains(p)) {
        return false;
    }
    source
        .lines()
        .enumerate()
        .all(|(i, line)| plain_line(i, line))
}

fn plain_line(index: usize, line: &str) -> bool {
    if !line.matches('\'').count().is_multiple_of(2) || !line.matches('"').count().is_multiple_of(2)
    {
        return false;
    }
    if quote_inside_double(line) {
        return false;
    }
    match line.find('#') {
        None => true,
        Some(pos) => line[..pos].trim().is_empty() || (index == 0 && line.starts_with("#!")),
    }
}

/// A `'` between an opening and closing `"` (a `"` inside `'...'` is literal).
fn quote_inside_double(line: &str) -> bool {
    let (mut single, mut double) = (false, false);
    for c in line.chars() {
        match c {
            '\'' if double => return true,
            '\'' => single = !single,
            '"' if !single => double = !double,
            _ => {}
        }
    }
    false
}

/// Replace the `$` of a positional parameter inside the single-quoted argument
/// that directly follows `sh -c ` or `bash -c ` with `_`.
pub(super) fn mask_child_shell(line: &str) -> String {
    let mut out = line.to_string();
    for marker in ["sh -c '", "bash -c '"] {
        let mut from = 0;
        while let Some(rel) = out[from..].find(marker) {
            let start = from + rel;
            let body = start + marker.len();
            let boundary = start == 0 || b" \t;&|(".contains(&out.as_bytes()[start - 1]);
            let Some(len) = out[body..].find('\'') else {
                break;
            };
            if boundary {
                let inner = out[body..body + len].to_string();
                out.replace_range(body..body + len, &mask_positional(&inner));
            }
            from = body + len + 1;
        }
    }
    out
}

fn mask_positional(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        let positional = c == '$' && matches!(chars.peek(), Some('1'..='9' | '@' | '*'));
        out.push(if positional { '_' } else { c });
    }
    out
}
