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
/// that directly follows `sh -c ` or `bash -c ` with `_`. The marker counts only
/// when its `'` opens a quote (even `'` and `"` counts before it) and `sh`/`bash`
/// is the command word of its segment (see `command_word`).
pub(super) fn mask_child_shell(line: &str) -> String {
    let mut out = line.to_string();
    for marker in ["sh -c '", "bash -c '"] {
        let mut from = 0;
        while let Some(rel) = out[from..].find(marker) {
            let start = from + rel;
            let body = start + marker.len();
            from = start + 1;
            if !marker_is_live(&out[..start]) {
                continue;
            }
            let Some(len) = out[body..].find('\'') else {
                break;
            };
            let inner = out[body..body + len].to_string();
            out.replace_range(body..body + len, &mask_positional(&inner));
            from = body + len + 1;
        }
    }
    out
}

/// `before` is the line text ahead of the marker.
fn marker_is_live(before: &str) -> bool {
    let even = |c: char| before.matches(c).count().is_multiple_of(2);
    even('\'') && even('"') && command_word(before)
}

/// True when the marker's `sh`/`bash` starts its command: only blanks follow the
/// last `;`, `|`, `&`, `(` or `{`, or the segment is `runuser ... -- `. Any other
/// prefix (`eval`, `exec`, `sudo`, `ssh host`, `then`, ...) is not masked.
fn command_word(before: &str) -> bool {
    let seg = before.rsplit([';', '|', '&', '(', '{']).next().unwrap_or("");
    let t = seg.trim();
    if t.is_empty() {
        return seg.is_empty() || seg.ends_with([' ', '\t']) || before.len() == seg.len();
    }
    t.starts_with("runuser ") && t.ends_with(" --") && !t.contains('$') && seg.ends_with(' ')
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
