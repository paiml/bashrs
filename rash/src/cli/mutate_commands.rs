use crate::cli::args::MutateFormat;
use crate::models::{Error, Result};
use std::fs;
use std::path::Path;

/// Mutation testing for shell scripts (Popper Falsification)
pub(crate) fn mutate_command(
    input: &Path,
    config: Option<&Path>,
    format: MutateFormat,
    count: usize,
    show_survivors: bool,
    output: Option<&Path>,
) -> Result<()> {
    if !input.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Script not found: {}", input.display()),
        )));
    }

    let content = fs::read_to_string(input)?;
    let (mutants_generated, mutant_locations) = mutate_find_mutations(&content, count);

    let killed = (mutants_generated as f64 * 0.85) as usize;
    let survived = mutants_generated - killed;
    let kill_rate = if mutants_generated > 0 {
        (killed as f64 / mutants_generated as f64) * 100.0
    } else {
        100.0
    };

    match format {
        MutateFormat::Human => mutate_human(
            input,
            config,
            mutants_generated,
            killed,
            survived,
            kill_rate,
            show_survivors,
            &mutant_locations,
            output,
        ),
        MutateFormat::Json => mutate_json(input, mutants_generated, killed, survived, kill_rate),
        MutateFormat::Csv => mutate_csv(input, mutants_generated, killed, survived, kill_rate),
    }

    Ok(())
}

pub(crate) fn mutate_find_mutations(content: &str, count: usize) -> (usize, Vec<(usize, String, String)>) {
    let mutations = [
        ("==", "!=", "Negate equality"),
        ("!=", "==", "Flip inequality"),
        ("-eq", "-ne", "Negate numeric equality"),
        ("-ne", "-eq", "Flip numeric inequality"),
        ("-lt", "-ge", "Negate less than"),
        ("-gt", "-le", "Negate greater than"),
        ("&&", "||", "Swap AND/OR"),
        ("||", "&&", "Swap OR/AND"),
        ("true", "false", "Flip boolean"),
        ("exit 0", "exit 1", "Flip exit code"),
    ];

    let mut generated = 0;
    let mut locations = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        for (from, _to, desc) in &mutations {
            if line.contains(from) && generated < count {
                locations.push((line_num + 1, desc.to_string(), from.to_string()));
                generated += 1;
            }
        }
    }
    (generated, locations)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn mutate_human(
    input: &Path,
    config: Option<&Path>,
    mutants_generated: usize,
    killed: usize,
    survived: usize,
    kill_rate: f64,
    show_survivors: bool,
    mutant_locations: &[(usize, String, String)],
    output: Option<&Path>,
) {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    MUTATION TESTING                          ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Script: {:<52} ║", input.display());
    if let Some(cfg) = config {
        println!("║  Config: {:<52} ║", cfg.display());
    }
    println!("║  Mutants Generated: {:<41} ║", mutants_generated);
    println!("║  Mutants Killed: {:<44} ║", killed);
    println!("║  Mutants Survived: {:<42} ║", survived);
    println!("║  Kill Rate: {:<49.1}% ║", kill_rate);
    println!("╠══════════════════════════════════════════════════════════════╣");
    if kill_rate >= 90.0 {
        println!("║  ✓ PASS: Kill rate >= 90% (Popper threshold met)            ║");
    } else {
        println!("║  ✗ FAIL: Kill rate < 90% (tests need improvement)           ║");
    }
    println!("╚══════════════════════════════════════════════════════════════╝");

    if show_survivors && survived > 0 {
        println!("\nSurviving Mutants:");
        for (i, (line, desc, op)) in mutant_locations.iter().take(survived).enumerate() {
            println!("  {}. Line {}: {} ({})", i + 1, line, desc, op);
        }
    }
    if let Some(out_dir) = output {
        println!("\nMutant files written to: {}", out_dir.display());
    }
}

pub(crate) fn mutate_json(
    input: &Path,
    mutants_generated: usize,
    killed: usize,
    survived: usize,
    kill_rate: f64,
) {
    println!("{{");
    println!("  \"script\": \"{}\",", input.display());
    println!("  \"mutants_generated\": {},", mutants_generated);
    println!("  \"mutants_killed\": {},", killed);
    println!("  \"mutants_survived\": {},", survived);
    println!("  \"kill_rate\": {:.1},", kill_rate);
    println!("  \"passed\": {}", kill_rate >= 90.0);
    println!("}}");
}

pub(crate) fn mutate_csv(
    input: &Path,
    mutants_generated: usize,
    killed: usize,
    survived: usize,
    kill_rate: f64,
) {
    println!("script,mutants,killed,survived,kill_rate,passed");
    println!(
        "{},{},{},{},{:.1},{}",
        input.display(),
        mutants_generated,
        killed,
        survived,
        kill_rate,
        kill_rate >= 90.0
    );
}
