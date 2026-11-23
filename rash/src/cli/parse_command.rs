//! `bashrs parse --format json <file|->`: the script's syntax tree plus every
//! simple command in it. Fails closed: any part that is not fully modelled
//! prints `{"unparsed":[{line,reason}]}` and exits 2, with no `commands`.
use crate::bash_parser::shell_extract::document;
use crate::bash_parser::shell_parse::parse_script;
use crate::cli::args::ParseFormat;
use crate::models::{Error, Result};
use std::io::Read;

fn unparsed(line: usize, reason: &str) -> ! {
    let doc = serde_json::json!({ "unparsed": [{ "line": line, "reason": reason }] });
    println!("{doc}");
    std::process::exit(2);
}

pub(crate) fn parse_command(input: &str, _format: &ParseFormat) -> Result<()> {
    let mut bytes = Vec::new();
    if input == "-" {
        std::io::stdin()
            .read_to_end(&mut bytes)
            .map_err(|e| Error::Validation(format!("Cannot read stdin: {e}")))?;
    } else {
        bytes = std::fs::read(input)
            .map_err(|e| Error::Validation(format!("Cannot read {input}: {e}")))?;
    }
    let source = match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => unparsed(1, "input is not valid UTF-8"),
    };
    match parse_script(&source) {
        Ok(program) => {
            println!("{}", document(&program));
            Ok(())
        }
        Err(e) => unparsed(e.line, &e.reason),
    }
}
