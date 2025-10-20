//! Lint rules for shell script analysis

// ShellCheck-equivalent rules
pub mod sc2001;
pub mod sc2002;
pub mod sc2006;
pub mod sc2027;
pub mod sc2028;
pub mod sc2034;
pub mod sc2043;
pub mod sc2044;
pub mod sc2045;
pub mod sc2046;
pub mod sc2048;
pub mod sc2050;
pub mod sc2066;
pub mod sc2068;
pub mod sc2070;
pub mod sc2071;
pub mod sc2072;
pub mod sc2076;
pub mod sc2081;
pub mod sc2086;
pub mod sc2103;
pub mod sc2104;
pub mod sc2105;
pub mod sc2107;
pub mod sc2116;
pub mod sc2128;
pub mod sc2145;
pub mod sc2153;
pub mod sc2154;
pub mod sc2155;
pub mod sc2157;
pub mod sc2158;
pub mod sc2160;
pub mod sc2162;
pub mod sc2163;
pub mod sc2164;
pub mod sc2166;
pub mod sc2168;
pub mod sc2169;
pub mod sc2170;
pub mod sc2172;
pub mod sc2178;
pub mod sc2181;
pub mod sc2190;
pub mod sc2191;
pub mod sc2196;

// Determinism rules (bashrs-specific)
pub mod det001;
pub mod det002;
pub mod det003;

// Idempotency rules (bashrs-specific)
pub mod idem001;
pub mod idem002;
pub mod idem003;

// Security rules (bashrs-specific)
pub mod sec001;
pub mod sec002;
pub mod sec003;
pub mod sec004;
pub mod sec005;
pub mod sec006;
pub mod sec007;
pub mod sec008;

// Makefile-specific rules (bashrs-specific)
pub mod make001;
pub mod make002;
pub mod make003;
pub mod make004;
pub mod make005;
pub mod make006;
pub mod make007;
pub mod make008;
pub mod make009;
pub mod make010;
pub mod make011;
pub mod make012;
pub mod make013;
pub mod make014;
pub mod make015;
pub mod make016;
pub mod make017;
pub mod make018;
pub mod make019;
pub mod make020;

use crate::linter::LintResult;

/// Lint a shell script and return all diagnostics
pub fn lint_shell(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Parse the shell script
    // For now, we'll use a simple token-based approach
    // In production, this would use the bash_parser AST

    // Run ShellCheck-equivalent rules
    result.merge(sc2001::check(source));
    result.merge(sc2002::check(source));
    result.merge(sc2006::check(source));
    result.merge(sc2027::check(source));
    result.merge(sc2028::check(source));
    result.merge(sc2034::check(source));
    result.merge(sc2043::check(source));
    result.merge(sc2044::check(source));
    result.merge(sc2045::check(source));
    result.merge(sc2046::check(source));
    result.merge(sc2048::check(source));
    result.merge(sc2050::check(source));
    result.merge(sc2066::check(source));
    result.merge(sc2068::check(source));
    result.merge(sc2070::check(source));
    result.merge(sc2071::check(source));
    result.merge(sc2072::check(source));
    result.merge(sc2076::check(source));
    result.merge(sc2081::check(source));
    result.merge(sc2086::check(source));
    result.merge(sc2103::check(source));
    result.merge(sc2104::check(source));
    result.merge(sc2105::check(source));
    result.merge(sc2107::check(source));
    result.merge(sc2116::check(source));
    result.merge(sc2128::check(source));
    result.merge(sc2145::check(source));
    result.merge(sc2153::check(source));
    result.merge(sc2154::check(source));
    result.merge(sc2155::check(source));
    result.merge(sc2157::check(source));
    result.merge(sc2158::check(source));
    result.merge(sc2160::check(source));
    result.merge(sc2162::check(source));
    result.merge(sc2163::check(source));
    result.merge(sc2164::check(source));
    result.merge(sc2166::check(source));
    result.merge(sc2168::check(source));
    result.merge(sc2169::check(source));
    result.merge(sc2170::check(source));
    result.merge(sc2172::check(source));
    result.merge(sc2178::check(source));
    result.merge(sc2181::check(source));
    result.merge(sc2190::check(source));
    result.merge(sc2191::check(source));
    result.merge(sc2196::check(source));

    // Run determinism rules
    result.merge(det001::check(source));
    result.merge(det002::check(source));
    result.merge(det003::check(source));

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

    result
}

/// Lint a Makefile and return all diagnostics
pub fn lint_makefile(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Run Makefile-specific rules
    result.merge(make001::check(source));
    result.merge(make002::check(source));
    result.merge(make003::check(source));
    result.merge(make004::check(source));
    result.merge(make005::check(source));
    result.merge(make006::check(source));
    result.merge(make007::check(source));
    result.merge(make008::check(source)); // CRITICAL: Tab vs spaces
    result.merge(make009::check(source));
    result.merge(make010::check(source));
    result.merge(make011::check(source));
    result.merge(make012::check(source));
    result.merge(make013::check(source));
    result.merge(make014::check(source));
    result.merge(make015::check(source));
    result.merge(make016::check(source));
    result.merge(make017::check(source));
    result.merge(make018::check(source));
    result.merge(make019::check(source));
    result.merge(make020::check(source));

    result
}
