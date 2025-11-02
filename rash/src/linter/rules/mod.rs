//! Lint rules for shell script analysis

// ShellCheck-equivalent rules
pub mod sc2001;
pub mod sc2002;
pub mod sc2003;
pub mod sc2004;
pub mod sc2005;
pub mod sc2006;
pub mod sc2007;
pub mod sc2015;
pub mod sc2016;
pub mod sc2017;
pub mod sc2018;
pub mod sc2019;
pub mod sc2020;
pub mod sc2021;
pub mod sc2022;
pub mod sc2023;
pub mod sc2024;
pub mod sc2025;
pub mod sc2026;
pub mod sc2027;
pub mod sc2028;
pub mod sc2029;
pub mod sc2030;
pub mod sc2031;
pub mod sc2032;
pub mod sc2033;
pub mod sc2034;
pub mod sc2035;
pub mod sc2036;
pub mod sc2037;
pub mod sc2038;
pub mod sc2039;
pub mod sc2040;
pub mod sc2041;
pub mod sc2042;
pub mod sc2043;
pub mod sc2044;
pub mod sc2045;
pub mod sc2046;
pub mod sc2047;
pub mod sc2048;
pub mod sc2049;
pub mod sc2050;
pub mod sc2051;
pub mod sc2052;
pub mod sc2053;
pub mod sc2054;
pub mod sc2055;
pub mod sc2056;
pub mod sc2057;
pub mod sc2059;
pub mod sc2060;
pub mod sc2061;
pub mod sc2062;
pub mod sc2063;
pub mod sc2064;
pub mod sc2065;
pub mod sc2066;
pub mod sc2067;
pub mod sc2068;
pub mod sc2069;
pub mod sc2070;
pub mod sc2071;
pub mod sc2072;
pub mod sc2073;
pub mod sc2074;
pub mod sc2075;
pub mod sc2076;
pub mod sc2077;
pub mod sc2078;
pub mod sc2079;
pub mod sc2080;
pub mod sc2081;
pub mod sc2082;
pub mod sc2083;
pub mod sc2084;
pub mod sc2085;
pub mod sc2086;
pub mod sc2087;
pub mod sc2088;
pub mod sc2089;
pub mod sc2090;
pub mod sc2091;
pub mod sc2092;
pub mod sc2093;
pub mod sc2094;
pub mod sc2095;
pub mod sc2096;
pub mod sc2097;
pub mod sc2098;
pub mod sc2099;
pub mod sc2100;
pub mod sc2101;
pub mod sc2102;
pub mod sc2103;
pub mod sc2104;
pub mod sc2105;
pub mod sc2106;
pub mod sc2107;
pub mod sc2108;
pub mod sc2109;
pub mod sc2110;
pub mod sc2111;
pub mod sc2112;
pub mod sc2113;
pub mod sc2114;
pub mod sc2115;
pub mod sc2116;
pub mod sc2117;
pub mod sc2118;
// pub mod sc2119;  // TODO: Requires AST parsing for proper function analysis (has false positives)
// pub mod sc2120;  // TODO: Requires AST parsing for proper function analysis (has false positives)
pub mod sc2121;
pub mod sc2122;
pub mod sc2123;
pub mod sc2124;
pub mod sc2125;
pub mod sc2126;
pub mod sc2127;
pub mod sc2128;
pub mod sc2129;
pub mod sc2130;
pub mod sc2131;
pub mod sc2132;
pub mod sc2133;
pub mod sc2134;
pub mod sc2135;
pub mod sc2136;
pub mod sc2137;
pub mod sc2138;
pub mod sc2139;
pub mod sc2140;
pub mod sc2141;
pub mod sc2142;
pub mod sc2143;
pub mod sc2144;
pub mod sc2145;
pub mod sc2146;
pub mod sc2147;
pub mod sc2148;
pub mod sc2149;
pub mod sc2150;
pub mod sc2151;
pub mod sc2152;
pub mod sc2153;
pub mod sc2154;
pub mod sc2155;
pub mod sc2156;
pub mod sc2157;
pub mod sc2158;
pub mod sc2159;
pub mod sc2160;
pub mod sc2161;
pub mod sc2162;
pub mod sc2163;
pub mod sc2164;
pub mod sc2165;
pub mod sc2166;
pub mod sc2167;
pub mod sc2168;
pub mod sc2169;
pub mod sc2170;
pub mod sc2171;
pub mod sc2172;
pub mod sc2173;
pub mod sc2174;
pub mod sc2175;
pub mod sc2176;
pub mod sc2177;
pub mod sc2178;
pub mod sc2179;
pub mod sc2180;
pub mod sc2181;
pub mod sc2182;
pub mod sc2183;
pub mod sc2184;
pub mod sc2185;
pub mod sc2186;
pub mod sc2187;
pub mod sc2188;
pub mod sc2189;
pub mod sc2190;
pub mod sc2191;
pub mod sc2192;
pub mod sc2193;
pub mod sc2194;
pub mod sc2195;
pub mod sc2196;
pub mod sc2197;
pub mod sc2198;
pub mod sc2199;
pub mod sc2200;
pub mod sc2201;
pub mod sc2202;
pub mod sc2203;
pub mod sc2204;
pub mod sc2205;
pub mod sc2206;
pub mod sc2207;
pub mod sc2208;
pub mod sc2209;
pub mod sc2210;
pub mod sc2211;
pub mod sc2212;
pub mod sc2213;
pub mod sc2214;
pub mod sc2215;
pub mod sc2216;
pub mod sc2217;
pub mod sc2218;
pub mod sc2219;
pub mod sc2220;
pub mod sc2221;
pub mod sc2222;
pub mod sc2223;
pub mod sc2224;
pub mod sc2225;
pub mod sc2226;
pub mod sc2227;
pub mod sc2228;
pub mod sc2229;
pub mod sc2230;
pub mod sc2231;
pub mod sc2232;
pub mod sc2233;
pub mod sc2234;
pub mod sc2235;
pub mod sc2236;
pub mod sc2237;
pub mod sc2238;
pub mod sc2239;
pub mod sc2240;
pub mod sc2241;
pub mod sc2242;
pub mod sc2243;
pub mod sc2244;
pub mod sc2245;
pub mod sc2246;
pub mod sc2247;
pub mod sc2248;
pub mod sc2249;
pub mod sc2250;
pub mod sc2251;
pub mod sc2252;
pub mod sc2253;
pub mod sc2254;
pub mod sc2255;
pub mod sc2256;
pub mod sc2257;
pub mod sc2258;
pub mod sc2259;
pub mod sc2260;
pub mod sc2261;
pub mod sc2262;
pub mod sc2263;
pub mod sc2264;
pub mod sc2265;
pub mod sc2266;
pub mod sc2267;
pub mod sc2268;
pub mod sc2269;
pub mod sc2270;
pub mod sc2271;
pub mod sc2272;
pub mod sc2273;
pub mod sc2274;
pub mod sc2275;
pub mod sc2276;
pub mod sc2277;
pub mod sc2278;
pub mod sc2279;
pub mod sc2280;
pub mod sc2281;
pub mod sc2282;
pub mod sc2283;
pub mod sc2284;
pub mod sc2285;
pub mod sc2286;
pub mod sc2287;
pub mod sc2288;
pub mod sc2289;
pub mod sc2290;
pub mod sc2291;
pub mod sc2292;
pub mod sc2293;
pub mod sc2294;
pub mod sc2295;
pub mod sc2296;
pub mod sc2297;
pub mod sc2298;
pub mod sc2299;
pub mod sc2300;
pub mod sc2301;
pub mod sc2302;
pub mod sc2303;
pub mod sc2304;
pub mod sc2305;
pub mod sc2306;
pub mod sc2307;
pub mod sc2308;
pub mod sc2309;
pub mod sc2310;
pub mod sc2311;
pub mod sc2312;
pub mod sc2313;
pub mod sc2314;
pub mod sc2315;
pub mod sc2316;
pub mod sc2317;
pub mod sc2318;
pub mod sc2319;
pub mod sc2320;
pub mod sc2321;
pub mod sc2322;
pub mod sc2323;
pub mod sc2324;
pub mod sc2325;

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

/// Lint a shell script with path-based shell type detection
///
/// Detects shell type from file path and content, then lints accordingly.
/// Shell-specific rules are filtered based on compatibility.
///
/// # Arguments
/// * `path` - File path for shell type detection (.zshrc, .bashrc, etc.)
/// * `source` - Shell script source code
///
/// # Returns
/// LintResult with diagnostics appropriate for the detected shell type
pub fn lint_shell_with_path(path: &std::path::Path, source: &str) -> LintResult {
    use crate::linter::shell_type::detect_shell_type;

    // Detect shell type from path and content
    let shell_type = detect_shell_type(path, source);

    // Run rules with shell-specific filtering
    lint_shell_filtered(source, shell_type)
}

/// Lint a shell script with shell-specific rule filtering
///
/// Conditionally applies rules based on shell type compatibility.
/// For example, bash-only array rules are skipped for POSIX sh files.
///
/// # Arguments
/// * `source` - Shell script source code
/// * `shell_type` - Detected or explicit shell type
///
/// # Returns
/// LintResult with filtered diagnostics
fn lint_shell_filtered(
    source: &str,
    shell_type: crate::linter::shell_type::ShellType,
) -> LintResult {
    use crate::linter::rule_registry::should_apply_rule;

    let mut result = LintResult::new();

    // Helper macro to conditionally apply rules
    macro_rules! apply_rule {
        ($rule_id:expr, $check_fn:expr) => {
            if should_apply_rule($rule_id, shell_type) {
                result.merge($check_fn(source));
            }
        };
    }

    // Run ShellCheck-equivalent rules with filtering
    apply_rule!("SC2001", sc2001::check);
    apply_rule!("SC2002", sc2002::check);
    apply_rule!("SC2003", sc2003::check);
    apply_rule!("SC2004", sc2004::check);
    apply_rule!("SC2005", sc2005::check);
    apply_rule!("SC2006", sc2006::check);
    apply_rule!("SC2007", sc2007::check);
    apply_rule!("SC2015", sc2015::check);
    apply_rule!("SC2016", sc2016::check);
    apply_rule!("SC2017", sc2017::check);
    apply_rule!("SC2018", sc2018::check);
    apply_rule!("SC2019", sc2019::check);
    apply_rule!("SC2020", sc2020::check);
    apply_rule!("SC2021", sc2021::check);
    apply_rule!("SC2022", sc2022::check);
    apply_rule!("SC2023", sc2023::check);
    apply_rule!("SC2024", sc2024::check);
    apply_rule!("SC2025", sc2025::check);
    apply_rule!("SC2026", sc2026::check);
    apply_rule!("SC2027", sc2027::check);
    apply_rule!("SC2029", sc2029::check);
    apply_rule!("SC2030", sc2030::check);

    // Add classified rules (SC2039 and SC2198-2201)
    apply_rule!("SC2039", sc2039::check); // NotSh - bash/zsh features
    apply_rule!("SC2198", sc2198::check); // NotSh - arrays
    apply_rule!("SC2199", sc2199::check); // NotSh - arrays
    apply_rule!("SC2200", sc2200::check); // NotSh - arrays
    apply_rule!("SC2201", sc2201::check); // NotSh - arrays

    // TODO: Add remaining SC2xxx rules (317 rules)
    // For now, fall back to lint_shell() for unclassified rules
    // This ensures backward compatibility while we incrementally classify

    // Determinism rules (Universal - always apply)
    result.merge(det001::check(source));
    result.merge(det002::check(source));
    result.merge(det003::check(source));

    // Idempotency rules (Universal - always apply)
    result.merge(idem001::check(source));
    result.merge(idem002::check(source));
    result.merge(idem003::check(source));

    // Security rules (Universal - always apply)
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

/// Lint a shell script and return all diagnostics
pub fn lint_shell(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Parse the shell script
    // For now, we'll use a simple token-based approach
    // In production, this would use the bash_parser AST

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

    // Preprocess Makefile to convert $$ → $ in recipes for shell linting
    use crate::linter::make_preprocess::preprocess_for_linting;
    let preprocessed = preprocess_for_linting(source);

    // Run Makefile-specific rules on original source
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

    // Run shell linting rules on preprocessed source
    // This prevents false positives from Make's $$ escaping
    result.merge(sc2133::check(&preprocessed));
    result.merge(sc2168::check(&preprocessed));
    result.merge(sc2299::check(&preprocessed));

    // For DET002, we want to allow timestamps in Makefiles
    // (they're used for build tracking), so we don't run it

    result
}
