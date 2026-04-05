/// Lint a shell script and return all diagnostics.
///
/// Runs all ShellCheck-equivalent rules on the provided shell script source code
/// and returns a collection of lint diagnostics (errors, warnings, info).
///
/// # Arguments
///
/// * `source` - The shell script source code to lint
///
/// # Returns
///
/// A [`LintResult`] containing all detected issues with their locations and severity.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use bashrs::linter::lint_shell;
///
/// let script = "#!/bin/sh\nprintf '%s\\n' 'hello'";
/// let result = lint_shell(script);
/// // Linting completes successfully
/// // (diagnostics may or may not be empty depending on rules)
/// ```
///
/// ## Detecting issues
///
/// ```
/// use bashrs::linter::lint_shell;
///
/// // Useless use of cat (SC2002)
/// let script = "cat file.txt | grep pattern";
/// let result = lint_shell(script);
/// // Should detect at least one issue
/// assert!(!result.diagnostics.is_empty());
/// ```
///
/// ## Multiple diagnostics
///
/// ```
/// use bashrs::linter::lint_shell;
///
/// let script = r#"
/// cat file | grep x
/// echo `date`
/// "#;
/// let result = lint_shell(script);
/// // Should find multiple issues (useless cat, backticks)
/// assert!(result.diagnostics.len() >= 2);
/// ```
pub fn lint_shell(source: &str) -> LintResult {
    // Contract: safety-classifier-v1.yaml precondition (pv codegen)
    contract_pre_lint_shell!(source);
    use crate::linter::suppression::SuppressionManager;

    let mut result = LintResult::new();

    // Parse the shell script
    // For now, we'll use a simple token-based approach
    // In production, this would use the bash_parser AST

    // Run SC1xxx rules (source code issues)
    result.merge(sc1014::check(source));
    result.merge(sc1017::check(source));
    result.merge(sc1018::check(source));
    result.merge(sc1026::check(source));
    result.merge(sc1028::check(source));
    result.merge(sc1036::check(source));
    result.merge(sc1038::check(source));
    result.merge(sc1040::check(source));
    result.merge(sc1041::check(source));
    result.merge(sc1044::check(source));
    result.merge(sc1045::check(source));
    result.merge(sc1065::check(source));
    result.merge(sc1066::check(source));
    result.merge(sc1075::check(source));
    result.merge(sc1082::check(source));
    result.merge(sc1083::check(source));
    result.merge(sc1086::check(source));
    result.merge(sc1090::check(source));
    result.merge(sc1091::check(source));
    result.merge(sc1094::check(source));
    result.merge(sc1097::check(source));
    result.merge(sc1100::check(source));
    result.merge(sc1109::check(source));
    result.merge(sc1008::check(source));
    result.merge(sc1084::check(source));
    result.merge(sc1104::check(source));
    result.merge(sc1113::check(source));
    result.merge(sc1114::check(source));
    result.merge(sc1115::check(source));
    result.merge(sc1120::check(source));
    result.merge(sc1127::check(source));
    result.merge(sc1128::check(source));
    result.merge(sc1129::check(source));

    // New SC1xxx rules
    result.merge(sc1007::check(source));
    result.merge(sc1009::check(source));
    result.merge(sc1020::check(source));
    result.merge(sc1035::check(source));
    result.merge(sc1068::check(source));
    result.merge(sc1069::check(source));
    result.merge(sc1095::check(source));
    result.merge(sc1099::check(source));
    result.merge(sc1101::check(source));
    result.merge(sc1037::check(source));
    result.merge(sc1076::check(source));
    result.merge(sc1087::check(source));
    result.merge(sc1105::check(source));
    result.merge(sc1106::check(source));
    result.merge(sc1131::check(source));
    result.merge(sc1139::check(source));
    result.merge(sc1140::check(source));
    result.merge(sc1003::check(source));
    result.merge(sc1004::check(source));
    result.merge(sc1012::check(source));
    result.merge(sc1078::check(source));
    result.merge(sc1079::check(source));
    result.merge(sc1098::check(source));
    result.merge(sc1110::check(source));
    result.merge(sc1111::check(source));
    result.merge(sc1117::check(source));
    result.merge(sc1135::check(source));

    // Run ShellCheck-equivalent rules
    result.merge(sc2001::check(source));
    result.merge(sc2002::check(source));
    result.merge(sc2003::check(source));
    result.merge(sc2004::check(source));
    result.merge(sc2005::check(source));
    result.merge(sc2006::check(source));
    result.merge(sc2007::check(source));
    result.merge(sc2015::check(source));
    result.merge(sc2016::check(source));
    result.merge(sc2017::check(source));
    result.merge(sc2018::check(source));
    result.merge(sc2019::check(source));
    result.merge(sc2020::check(source));
    result.merge(sc2021::check(source));
    result.merge(sc2022::check(source));
    result.merge(sc2023::check(source));
    result.merge(sc2024::check(source));
    result.merge(sc2025::check(source));
    result.merge(sc2026::check(source));
    result.merge(sc2027::check(source));
    result.merge(sc2029::check(source));
    result.merge(sc2030::check(source));
    result.merge(sc2031::check(source));
    result.merge(sc2032::check(source));
    result.merge(sc2028::check(source));
    result.merge(sc2033::check(source));
    result.merge(sc2036::check(source));
    result.merge(sc2037::check(source));
    result.merge(sc2038::check(source));
    result.merge(sc2039::check(source));
    result.merge(sc2040::check(source));
    result.merge(sc2041::check(source));
    result.merge(sc2042::check(source));
    result.merge(sc2034::check(source));
    result.merge(sc2035::check(source));
    result.merge(sc2043::check(source));
    result.merge(sc2044::check(source));
    result.merge(sc2045::check(source));
    result.merge(sc2046::check(source));
    result.merge(sc2047::check(source));
    result.merge(sc2048::check(source));
    result.merge(sc2049::check(source));
    result.merge(sc2050::check(source));
    result.merge(sc2051::check(source));
    result.merge(sc2052::check(source));
    result.merge(sc2053::check(source));
    result.merge(sc2054::check(source));
    result.merge(sc2055::check(source));
    result.merge(sc2056::check(source));
    result.merge(sc2057::check(source));
    result.merge(sc2058::check(source));
    result.merge(sc2059::check(source));
    result.merge(sc2060::check(source));
    result.merge(sc2062::check(source));
    result.merge(sc2063::check(source));
    result.merge(sc2064::check(source));
    result.merge(sc2065::check(source));
    result.merge(sc2061::check(source));
    result.merge(sc2066::check(source));
    result.merge(sc2067::check(source));
    result.merge(sc2068::check(source));
    result.merge(sc2069::check(source));
    result.merge(sc2070::check(source));
    result.merge(sc2071::check(source));
    result.merge(sc2072::check(source));
    result.merge(sc2073::check(source));
    result.merge(sc2074::check(source));
    result.merge(sc2075::check(source));
    result.merge(sc2076::check(source));
    result.merge(sc2077::check(source));
    result.merge(sc2078::check(source));
    result.merge(sc2079::check(source));
    result.merge(sc2080::check(source));
    result.merge(sc2081::check(source));
    result.merge(sc2082::check(source));
    result.merge(sc2083::check(source));
    result.merge(sc2084::check(source));
    result.merge(sc2085::check(source));
    result.merge(sc2086::check(source));
    result.merge(sc2087::check(source));
    result.merge(sc2088::check(source));
    result.merge(sc2089::check(source));
    result.merge(sc2090::check(source));
    result.merge(sc2091::check(source));
    result.merge(sc2092::check(source));
    result.merge(sc2093::check(source));
    result.merge(sc2094::check(source));
    result.merge(sc2095::check(source));
    result.merge(sc2096::check(source));
    result.merge(sc2097::check(source));
    result.merge(sc2098::check(source));
    result.merge(sc2099::check(source));
    result.merge(sc2100::check(source));
    result.merge(sc2101::check(source));
    result.merge(sc2102::check(source));
    result.merge(sc2103::check(source));
    result.merge(sc2104::check(source));
    result.merge(sc2105::check(source));
    result.merge(sc2106::check(source));
    result.merge(sc2107::check(source));
    result.merge(sc2108::check(source));
    result.merge(sc2109::check(source));
    result.merge(sc2110::check(source));
    result.merge(sc2111::check(source));
    result.merge(sc2112::check(source));
    result.merge(sc2113::check(source));
    result.merge(sc2114::check(source));
    result.merge(sc2115::check(source));
    result.merge(sc2116::check(source));
    result.merge(sc2117::check(source));
    result.merge(sc2118::check(source));
    // result.merge(sc2119::check(source));  // Deferred: False positives without AST
    // result.merge(sc2120::check(source));  // Deferred: False positives without AST
    result.merge(sc2121::check(source));
    result.merge(sc2122::check(source));
    result.merge(sc2123::check(source));
    result.merge(sc2124::check(source));
    result.merge(sc2125::check(source));
    result.merge(sc2126::check(source));
    result.merge(sc2127::check(source));
    result.merge(sc2128::check(source));
    result.merge(sc2129::check(source));
    result.merge(sc2130::check(source));
    result.merge(sc2131::check(source));
    result.merge(sc2132::check(source));
    result.merge(sc2133::check(source));
    result.merge(sc2134::check(source));
    result.merge(sc2135::check(source));
    result.merge(sc2136::check(source));
    result.merge(sc2137::check(source));
    result.merge(sc2138::check(source));
    result.merge(sc2139::check(source));
    result.merge(sc2140::check(source));
    result.merge(sc2141::check(source));
    result.merge(sc2142::check(source));
    result.merge(sc2143::check(source));
    result.merge(sc2144::check(source));
    result.merge(sc2145::check(source));
    result.merge(sc2146::check(source));
    result.merge(sc2147::check(source));
    result.merge(sc2148::check(source));
    result.merge(sc2149::check(source));
    result.merge(sc2150::check(source));
    result.merge(sc2151::check(source));
    result.merge(sc2152::check(source));
    result.merge(sc2153::check(source));
    result.merge(sc2154::check(source));
    result.merge(sc2155::check(source));
    result.merge(sc2156::check(source));
    result.merge(sc2157::check(source));
    result.merge(sc2158::check(source));
    result.merge(sc2159::check(source));
    result.merge(sc2160::check(source));
    result.merge(sc2161::check(source));
    result.merge(sc2162::check(source));
    result.merge(sc2163::check(source));
    result.merge(sc2164::check(source));
    result.merge(sc2165::check(source));
    result.merge(sc2166::check(source));
    result.merge(sc2167::check(source));
    result.merge(sc2168::check(source));
    result.merge(sc2169::check(source));
    result.merge(sc2170::check(source));
    result.merge(sc2171::check(source));
    result.merge(sc2172::check(source));
    result.merge(sc2173::check(source));
    result.merge(sc2174::check(source));
    result.merge(sc2175::check(source));
    result.merge(sc2176::check(source));
    result.merge(sc2177::check(source));
    result.merge(sc2178::check(source));
    result.merge(sc2179::check(source));
    result.merge(sc2180::check(source));
    result.merge(sc2181::check(source));
    result.merge(sc2182::check(source));
    result.merge(sc2183::check(source));
    result.merge(sc2184::check(source));
    result.merge(sc2185::check(source));
    result.merge(sc2186::check(source));
    result.merge(sc2187::check(source));
    result.merge(sc2188::check(source));
    result.merge(sc2189::check(source));
    result.merge(sc2190::check(source));
    result.merge(sc2191::check(source));
    result.merge(sc2192::check(source));
    result.merge(sc2193::check(source));
    result.merge(sc2194::check(source));
    result.merge(sc2195::check(source));
    result.merge(sc2196::check(source));
    result.merge(sc2197::check(source));
    result.merge(sc2198::check(source));
    result.merge(sc2199::check(source));
    result.merge(sc2200::check(source));
    result.merge(sc2201::check(source));
    result.merge(sc2202::check(source));
    result.merge(sc2203::check(source));
    result.merge(sc2204::check(source));
    result.merge(sc2205::check(source));
    result.merge(sc2206::check(source));
    result.merge(sc2207::check(source));
    result.merge(sc2208::check(source));
    result.merge(sc2209::check(source));
    result.merge(sc2210::check(source));
    result.merge(sc2211::check(source));
    result.merge(sc2212::check(source));
    result.merge(sc2213::check(source));
    result.merge(sc2214::check(source));
    result.merge(sc2215::check(source));
    result.merge(sc2216::check(source));
    result.merge(sc2217::check(source));
    result.merge(sc2218::check(source));
    result.merge(sc2219::check(source));
    result.merge(sc2220::check(source));
    result.merge(sc2221::check(source));
    result.merge(sc2222::check(source));
    result.merge(sc2223::check(source));
    result.merge(sc2224::check(source));
    result.merge(sc2225::check(source));
    result.merge(sc2226::check(source));
    result.merge(sc2227::check(source));
    result.merge(sc2228::check(source));
    result.merge(sc2229::check(source));
    result.merge(sc2230::check(source));
    result.merge(sc2231::check(source));
    result.merge(sc2232::check(source));
    result.merge(sc2233::check(source));
    result.merge(sc2234::check(source));
    result.merge(sc2235::check(source));
    result.merge(sc2236::check(source));
    result.merge(sc2237::check(source));
    result.merge(sc2238::check(source));
    result.merge(sc2239::check(source));
    result.merge(sc2240::check(source));
    result.merge(sc2241::check(source));
    result.merge(sc2242::check(source));
    result.merge(sc2243::check(source));
    result.merge(sc2244::check(source));
    result.merge(sc2245::check(source));
    result.merge(sc2246::check(source));
    result.merge(sc2247::check(source));
    result.merge(sc2248::check(source));
    result.merge(sc2249::check(source));
    result.merge(sc2250::check(source));
    result.merge(sc2251::check(source));
    result.merge(sc2252::check(source));
    result.merge(sc2253::check(source));
    result.merge(sc2254::check(source));
    result.merge(sc2255::check(source));
    result.merge(sc2256::check(source));
    result.merge(sc2257::check(source));
    result.merge(sc2258::check(source));
    result.merge(sc2259::check(source));
    result.merge(sc2260::check(source));
    result.merge(sc2261::check(source));
    result.merge(sc2262::check(source));
    result.merge(sc2263::check(source));
    result.merge(sc2264::check(source));
    result.merge(sc2265::check(source));
    result.merge(sc2266::check(source));
    result.merge(sc2267::check(source));
    result.merge(sc2268::check(source));
    result.merge(sc2269::check(source));
    result.merge(sc2270::check(source));
    result.merge(sc2271::check(source));
    result.merge(sc2272::check(source));
    result.merge(sc2273::check(source));
    result.merge(sc2274::check(source));
    result.merge(sc2275::check(source));
    result.merge(sc2276::check(source));
    result.merge(sc2277::check(source));
    result.merge(sc2278::check(source));
    result.merge(sc2279::check(source));
    result.merge(sc2280::check(source));
    result.merge(sc2281::check(source));
    result.merge(sc2282::check(source));
    result.merge(sc2283::check(source));
    result.merge(sc2284::check(source));
    result.merge(sc2285::check(source));
    result.merge(sc2286::check(source));
    result.merge(sc2287::check(source));
    result.merge(sc2288::check(source));
    result.merge(sc2289::check(source));
    result.merge(sc2290::check(source));
    result.merge(sc2291::check(source));
    result.merge(sc2292::check(source));
    result.merge(sc2293::check(source));
    result.merge(sc2294::check(source));
    result.merge(sc2295::check(source));
    result.merge(sc2296::check(source));
    result.merge(sc2297::check(source));
    result.merge(sc2298::check(source));
    result.merge(sc2299::check(source));
    result.merge(sc2300::check(source));
    result.merge(sc2301::check(source));
    result.merge(sc2302::check(source));
    result.merge(sc2303::check(source));
    result.merge(sc2304::check(source));
    result.merge(sc2305::check(source));
    result.merge(sc2306::check(source));
    result.merge(sc2307::check(source));
    result.merge(sc2308::check(source));
    result.merge(sc2309::check(source));
    result.merge(sc2310::check(source));
    result.merge(sc2311::check(source));
    result.merge(sc2312::check(source));
    result.merge(sc2313::check(source));
    result.merge(sc2314::check(source));
    result.merge(sc2315::check(source));
    result.merge(sc2316::check(source));
    result.merge(sc2317::check(source));
    result.merge(sc2318::check(source));
    result.merge(sc2319::check(source));
    result.merge(sc2320::check(source));
    result.merge(sc2321::check(source));
    result.merge(sc2322::check(source));
    result.merge(sc2323::check(source));
    result.merge(sc2324::check(source));
    result.merge(sc2325::check(source));

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

    result
}











include!("mod_std.rs");
