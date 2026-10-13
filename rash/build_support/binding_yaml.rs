//! Strict reader for provable-contracts `binding.yaml`, shared by `build.rs`
//! and its tests (each includes this file with `#[path]`).
//!
//! `build.rs` used to parse the file with `serde_yaml_ng`, a build-dependency
//! that pulls `unsafe-libyaml` (RUSTSEC-2024-0449) into every crate that
//! depends on bashrs. The build script reads four scalar keys of one shape —
//! a top-level `bindings:` block sequence of flat mappings — so it reads that
//! shape and nothing else.
//!
//! The reader is strict, not lenient: any construct it does not fully
//! understand in a key it uses (block or flow collections, block scalars,
//! anchors, tags, escapes, multi-line scalars, duplicate keys) is an `Err`,
//! never a guess. Keys it does not use are skipped with their nested lines.

/// One entry of `bindings:`; the fields `build.rs` reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Binding {
    pub(crate) contract: String,
    pub(crate) equation: String,
    pub(crate) status: String,
    pub(crate) function: Option<String>,
}

/// The `cargo:rustc-env` variable name `build.rs` emits for a binding.
pub(crate) fn env_var_name(b: &Binding) -> String {
    format!(
        "CONTRACT_{}_{}",
        b.contract
            .trim_end_matches(".yaml")
            .to_uppercase()
            .replace('-', "_"),
        b.equation.to_uppercase().replace('-', "_")
    )
}

/// Parse `binding.yaml` source into its bindings, or say why it cannot.
pub(crate) fn parse_bindings(src: &str) -> Result<Vec<Binding>, String> {
    let mut doc = Doc::default();
    for (idx, raw) in src.lines().enumerate() {
        doc.line(idx + 1, raw)?;
    }
    doc.finish()
}

#[derive(Default)]
struct Item {
    contract: Option<String>,
    equation: Option<String>,
    status: Option<String>,
    function: Option<Option<String>>,
}

#[derive(Default)]
struct Doc {
    /// Top-level key whose nested lines are being read.
    section: Option<String>,
    has_version: bool,
    /// `None` until `bindings:` is seen.
    bindings: Option<Vec<Binding>>,
    /// Column of the `-` of the first item.
    dash_col: Option<usize>,
    item: Option<Item>,
    /// The item key whose value may continue on deeper lines.
    last_key: Option<String>,
}

impl Doc {
    fn line(&mut self, n: usize, raw: &str) -> Result<(), String> {
        if raw.contains('\t') {
            return Err(format!("line {n}: tab character"));
        }
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(());
        }
        let indent = raw.len() - raw.trim_start().len();
        if indent == 0 && n == 1 && trimmed == "---" {
            return Ok(());
        }
        if indent == 0 && !trimmed.starts_with('-') {
            return self.top_level(n, trimmed);
        }
        if self.section.as_deref() == Some("bindings") {
            return self.binding_line(n, indent, trimmed);
        }
        if self.section.is_some() {
            return Ok(());
        }
        Err(format!("line {n}: content outside any top-level key"))
    }

    fn top_level(&mut self, n: usize, text: &str) -> Result<(), String> {
        self.close_item()?;
        let (key, value) = split_key(n, text)?;
        match key {
            "version" => {
                if self.has_version {
                    return Err(format!("line {n}: duplicate key `version`"));
                }
                self.has_version = true;
                scalar(n, value)?;
            }
            "bindings" => {
                if self.bindings.is_some() {
                    return Err(format!("line {n}: duplicate key `bindings`"));
                }
                match value.trim() {
                    "" => {}
                    "[]" => {
                        self.bindings = Some(Vec::new());
                        self.section = None;
                        return Ok(());
                    }
                    other => return Err(format!("line {n}: unsupported `bindings: {other}`")),
                }
                self.bindings = Some(Vec::new());
            }
            _ => {}
        }
        self.section = Some(key.to_string());
        Ok(())
    }

    fn binding_line(&mut self, n: usize, indent: usize, text: &str) -> Result<(), String> {
        let dash_col = *self.dash_col.get_or_insert(indent);
        let key_col = dash_col + 2;
        if indent == dash_col && (text == "-" || text.starts_with("- ")) {
            self.close_item()?;
            self.item = Some(Item::default());
            self.last_key = None;
            let rest = text[1..].trim_start();
            if rest.is_empty() {
                return Ok(());
            }
            return self.item_key(n, rest);
        }
        if indent == key_col && self.item.is_some() {
            return self.item_key(n, text);
        }
        if indent > key_col && self.item.is_some() {
            return match self.last_key.as_deref() {
                Some(k) if is_used(k) => Err(format!("line {n}: multi-line value for `{k}`")),
                Some(_) => Ok(()),
                None => Err(format!("line {n}: unexpected indentation")),
            };
        }
        Err(format!("line {n}: unexpected indentation in `bindings`"))
    }

    fn item_key(&mut self, n: usize, text: &str) -> Result<(), String> {
        let (key, value) = split_key(n, text)?;
        let item = self
            .item
            .as_mut()
            .ok_or(format!("line {n}: key outside an item"))?;
        self.last_key = Some(key.to_string());
        let slot = match key {
            "contract" => &mut item.contract,
            "equation" => &mut item.equation,
            "status" => &mut item.status,
            "function" => {
                if item.function.is_some() {
                    return Err(format!("line {n}: duplicate key `function`"));
                }
                item.function = Some(scalar(n, value)?);
                return Ok(());
            }
            _ => return Ok(()),
        };
        if slot.is_some() {
            return Err(format!("line {n}: duplicate key `{key}`"));
        }
        let v = scalar(n, value)?.ok_or(format!("line {n}: `{key}` is null"))?;
        *slot = Some(v);
        Ok(())
    }

    fn close_item(&mut self) -> Result<(), String> {
        let Some(item) = self.item.take() else {
            return Ok(());
        };
        let missing = |k: &str| format!("binding missing `{k}`");
        let b = Binding {
            contract: item.contract.ok_or_else(|| missing("contract"))?,
            equation: item.equation.ok_or_else(|| missing("equation"))?,
            status: item.status.ok_or_else(|| missing("status"))?,
            function: item.function.flatten(),
        };
        if let Some(list) = self.bindings.as_mut() {
            list.push(b);
        }
        Ok(())
    }

    fn finish(mut self) -> Result<Vec<Binding>, String> {
        self.close_item()?;
        if !self.has_version {
            return Err("missing top-level `version`".into());
        }
        self.bindings
            .ok_or_else(|| "missing top-level `bindings`".into())
    }
}

fn is_used(key: &str) -> bool {
    matches!(key, "contract" | "equation" | "status" | "function")
}

/// Split `key: value` (or `key:`); quoted or complex keys are refused.
fn split_key(n: usize, text: &str) -> Result<(&str, &str), String> {
    // The first `: ` ends the key; a value may itself contain or end in `:`.
    let (key, value) = match (text.split_once(": "), text.strip_suffix(':')) {
        (Some(kv), _) => kv,
        (None, Some(k)) => (k, ""),
        (None, None) => return Err(format!("line {n}: expected `key: value`")),
    };
    let ok = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !ok {
        return Err(format!("line {n}: unsupported key `{key}`"));
    }
    Ok((key, value))
}

/// A single-line scalar: `Ok(None)` is YAML null.
fn scalar(n: usize, value: &str) -> Result<Option<String>, String> {
    let v = value.trim();
    if v.is_empty() || matches!(v, "~" | "null" | "Null" | "NULL") {
        return Ok(None);
    }
    if let Some(rest) = v.strip_prefix('\'') {
        return single_quoted(n, rest).map(Some);
    }
    if let Some(rest) = v.strip_prefix('"') {
        return double_quoted(n, rest).map(Some);
    }
    let first = v.chars().next().unwrap_or(' ');
    if "|>{[&*!%@`?".contains(first) || v == "-" || v.starts_with("- ") {
        return Err(format!("line {n}: unsupported scalar `{v}`"));
    }
    let plain = match v.find(" #") {
        Some(i) => v[..i].trim_end(),
        None => v,
    };
    if plain.contains(": ") || plain.ends_with(':') {
        return Err(format!("line {n}: unsupported scalar `{v}`"));
    }
    Ok(Some(plain.to_string()))
}

fn double_quoted(n: usize, rest: &str) -> Result<String, String> {
    let (body, tail) = rest
        .split_once('"')
        .ok_or(format!("line {n}: unterminated double quote"))?;
    if body.contains('\\') {
        return Err(format!("line {n}: escape sequences are not supported"));
    }
    trailing_comment_only(n, tail)?;
    Ok(body.to_string())
}

fn single_quoted(n: usize, rest: &str) -> Result<String, String> {
    let mut out = String::new();
    let mut chars = rest.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c != '\'' {
            out.push(c);
            continue;
        }
        if chars.peek().map(|&(_, c2)| c2) == Some('\'') {
            out.push('\'');
            chars.next();
            continue;
        }
        trailing_comment_only(n, &rest[i + 1..])?;
        return Ok(out);
    }
    Err(format!("line {n}: unterminated single quote"))
}

fn trailing_comment_only(n: usize, tail: &str) -> Result<(), String> {
    let t = tail.trim_start();
    if t.is_empty() || (t.starts_with('#') && tail.len() > t.len()) {
        Ok(())
    } else {
        Err(format!("line {n}: text after a quoted scalar"))
    }
}
