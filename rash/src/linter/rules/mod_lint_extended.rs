/// Apply extended lint rules: determinism, security, performance, portability, reliability.
/// Also applies inline suppression and embedded program filtering.
fn apply_extended_lint_rules(source: &str, result: &mut LintResult) {
    use crate::linter::suppression::SuppressionManager;

    // Run determinism rules
    result.merge(det001::check(source));
    result.merge(det002::check(source));
    result.merge(det003::check(source));
    result.merge(det004::check(source));

    // Run idempotency rules
    result.merge(idem001::check(source));
    result.merge(idem002::check(source));
    result.merge(idem003::check(source));

    // Run security rules
    result.merge(sec001::check(source));
    result.merge(sec002::check(source));
    result.merge(sec003::check(source));
    result.merge(sec004::check(source));
    result.merge(sec005::check(source));
    result.merge(sec006::check(source));
    result.merge(sec007::check(source));
    result.merge(sec008::check(source));
    result.merge(sec009::check(source));
    result.merge(sec010::check(source));
    result.merge(sec011::check(source));
    result.merge(sec012::check(source));
    result.merge(sec013::check(source));
    result.merge(sec014::check(source));
    result.merge(sec015::check(source));
    result.merge(sec016::check(source));
    result.merge(sec017::check(source));
    result.merge(sec018::check(source));
    // SEC019 not dispatched: see note in lint_shell_filtered
    result.merge(sec020::check(source));
    result.merge(sec021::check(source));
    result.merge(sec022::check(source));
    result.merge(sec023::check(source));
    result.merge(sec024::check(source));

    // Performance rules
    result.merge(perf001::check(source));
    result.merge(perf002::check(source));
    result.merge(perf003::check(source));
    result.merge(perf004::check(source));
    result.merge(perf005::check(source));

    // Portability rules
    result.merge(port001::check(source));
    result.merge(port002::check(source));
    result.merge(port003::check(source));
    result.merge(port004::check(source));
    result.merge(port005::check(source));

    // Reliability rules
    result.merge(rel001::check(source));
    result.merge(rel002::check(source));
    result.merge(rel003::check(source));
    result.merge(rel004::check(source));
    result.merge(rel005::check(source));

    // Apply inline suppression filtering
    let suppression_manager = SuppressionManager::from_source(source);
    result
        .diagnostics
        .retain(|diag| !suppression_manager.is_suppressed(&diag.code, diag.span.start_line));

    // Filter out diagnostics inside embedded programs (awk, sed, perl, etc.)
    // See: https://github.com/paiml/bashrs/issues/137
    // Security (SEC*) and determinism (DET*) rules are exempt — they detect
    // genuine threats/issues at the shell command level, not inside awk/sed code
    let embedded_lines = crate::linter::embedded::embedded_program_lines(source);
    if !embedded_lines.is_empty() {
        result.diagnostics.retain(|diag| {
            diag.code.starts_with("SEC")
                || diag.code.starts_with("DET")
                || !embedded_lines.contains(&diag.span.start_line)
        });
    }
}
