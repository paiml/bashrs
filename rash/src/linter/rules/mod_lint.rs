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
    if source.is_empty() {
        return LintResult::new();
    }
    // Contract: safety-classifier-v1.yaml precondition (pv codegen)
    contract_pre_lint_shell!(source);

    let mut result = LintResult::new();

    // GH-226: shell-SYNTAX rules must not see the contents of string literals
    // (a regex character class is not a test expression). Same allowlist and
    // same mechanism as `lint_shell_filtered`.
    let masked = crate::linter::quoting::mask_literals(source);

    // GH-272: the choice between `&masked` and `source` used to be written out
    // by hand at each of the 378 call sites below, which made
    // `quoting::QUOTE_SENSITIVE_RULES` and this function two hand-maintained
    // lists that had to agree. They silently drifted the moment a rule was
    // added to the allowlist alone: `lint_shell_filtered` picked it up and the
    // CLI, which comes through here, did not. Deriving the input from the
    // allowlist by MODULE NAME leaves one list, and a call site can no longer
    // disagree with it.
    macro_rules! apply {
        ($rule:ident) => {
            result.merge($rule::check(
                if crate::linter::quoting::is_quote_sensitive_module(stringify!($rule)) {
                    masked.as_str()
                } else {
                    source
                },
            ));
        };
    }

    // Parse the shell script
    // For now, we'll use a simple token-based approach
    // In production, this would use the bash_parser AST

    // Run SC1xxx rules (source code issues)
    apply!(sc1014);
    apply!(sc1017);
    apply!(sc1018);
    apply!(sc1026);
    apply!(sc1028);
    apply!(sc1036);
    apply!(sc1038);
    apply!(sc1040);
    apply!(sc1041);
    apply!(sc1044);
    apply!(sc1045);
    apply!(sc1065);
    apply!(sc1066);
    apply!(sc1075);
    apply!(sc1082);
    apply!(sc1083);
    apply!(sc1086);
    apply!(sc1090);
    apply!(sc1091);
    apply!(sc1094);
    apply!(sc1097);
    apply!(sc1100);
    apply!(sc1109);
    apply!(sc1008);
    apply!(sc1084);
    apply!(sc1104);
    apply!(sc1113);
    apply!(sc1114);
    apply!(sc1115);
    apply!(sc1120);
    apply!(sc1127);
    apply!(sc1128);
    apply!(sc1129);

    // New SC1xxx rules
    apply!(sc1007);
    apply!(sc1009);
    apply!(sc1020);
    apply!(sc1035);
    apply!(sc1068);
    apply!(sc1069);
    apply!(sc1095);
    apply!(sc1099);
    apply!(sc1101);
    apply!(sc1037);
    apply!(sc1076);
    apply!(sc1087);
    apply!(sc1105);
    apply!(sc1106);
    apply!(sc1131);
    apply!(sc1139);
    apply!(sc1140);
    apply!(sc1003);
    apply!(sc1004);
    apply!(sc1012);
    apply!(sc1078);
    apply!(sc1079);
    apply!(sc1098);
    apply!(sc1110);
    apply!(sc1111);
    apply!(sc1117);
    apply!(sc1135);

    // Run ShellCheck-equivalent rules
    apply!(sc2001);
    apply!(sc2002);
    apply!(sc2003);
    apply!(sc2004);
    apply!(sc2005);
    apply!(sc2006);
    apply!(sc2007);
    apply!(sc2015);
    apply!(sc2016);
    apply!(sc2017);
    apply!(sc2018);
    apply!(sc2019);
    apply!(sc2020);
    apply!(sc2021);
    apply!(sc2022);
    apply!(sc2023);
    apply!(sc2024);
    apply!(sc2025);
    apply!(sc2026);
    apply!(sc2027);
    apply!(sc2029);
    apply!(sc2030);
    apply!(sc2031);
    // SC2032 RETIRED — see linter::code_namespace::RETIRED.
    apply!(sc2028);
    apply!(sc2033);
    apply!(sc2036);
    apply!(sc2037);
    apply!(sc2038);
    apply!(sc2039);
    apply!(sc2040);
    apply!(sc2041);
    apply!(sc2042);
    apply!(sc2034);
    apply!(sc2035);
    apply!(sc2043);
    apply!(sc2044);
    apply!(sc2045);
    apply!(sc2046);
    apply!(sc2047);
    apply!(sc2048);
    apply!(sc2049);
    apply!(sc2050);
    apply!(sc2051);
    apply!(sc2052);
    apply!(sc2053);
    apply!(sc2054);
    apply!(sc2055);
    apply!(sc2056);
    apply!(sc2057);
    apply!(sc2058);
    apply!(sc2059);
    apply!(sc2060);
    apply!(sc2062);
    apply!(sc2063);
    apply!(sc2064);
    apply!(sc2065);
    apply!(sc2061);
    apply!(sc2066);
    apply!(sc2067);
    apply!(sc2068);
    apply!(sc2069);
    apply!(sc2070);
    apply!(sc2071);
    apply!(sc2072);
    apply!(sc2073);
    apply!(sc2074);
    apply!(sc2075);
    apply!(sc2076);
    apply!(sc2077);
    apply!(sc2078);
    apply!(sc2079);
    apply!(sc2080);
    apply!(sc2081);
    apply!(sc2082);
    apply!(sc2083);
    apply!(sc2084);
    apply!(sc2085);
    apply!(sc2086);
    apply!(sc2087);
    apply!(sc2088);
    apply!(sc2089);
    apply!(sc2090);
    apply!(sc2091);
    apply!(sc2092);
    apply!(sc2093);
    apply!(sc2094);
    apply!(sc2095);
    apply!(sc2096);
    apply!(sc2097);
    apply!(sc2098);
    apply!(sc2099);
    apply!(sc2100);
    apply!(sc2101);
    apply!(sc2102);
    apply!(sc2103);
    apply!(sc2104);
    apply!(sc2105);
    apply!(sc2106);
    apply!(sc2107);
    apply!(sc2108);
    apply!(sc2109);
    apply!(sc2110);
    apply!(sc2111);
    apply!(sc2112);
    apply!(sc2113);
    apply!(sc2114);
    apply!(sc2115);
    apply!(sc2116);
    apply!(sc2117);
    apply!(sc2118);
    // apply!(sc2119);  // Deferred: False positives without AST
    // apply!(sc2120);  // Deferred: False positives without AST
    apply!(sc2121);
    apply!(sc2122);
    apply!(sc2123);
    apply!(sc2124);
    apply!(sc2125);
    apply!(sc2126);
    apply!(sc2127);
    apply!(sc2128);
    apply!(sc2129);
    apply!(sc2130);
    apply!(sc2131);
    apply!(sc2132);
    apply!(sc2133);
    apply!(sc2134);
    apply!(sc2135);
    apply!(sc2136);
    apply!(sc2137);
    apply!(sc2138);
    apply!(sc2139);
    apply!(sc2140);
    apply!(sc2141);
    apply!(sc2142);
    apply!(sc2143);
    apply!(sc2144);
    apply!(sc2145);
    apply!(sc2146);
    apply!(sc2147);
    apply!(sc2148);
    apply!(sc2149);
    apply!(sc2150);
    apply!(sc2151);
    apply!(sc2152);
    apply!(sc2153);
    apply!(sc2154);
    apply!(sc2155);
    apply!(sc2156);
    apply!(sc2157);
    apply!(sc2158);
    apply!(sc2159);
    apply!(sc2160);
    apply!(sc2161);
    apply!(sc2162);
    apply!(sc2163);
    apply!(sc2164);
    apply!(sc2165);
    apply!(sc2166);
    apply!(sc2167);
    apply!(sc2168);
    apply!(sc2169);
    apply!(sc2170);
    apply!(sc2171);
    apply!(sc2172);
    apply!(sc2173);
    apply!(sc2174);
    apply!(sc2175);
    apply!(sc2176);
    apply!(sc2177);
    apply!(sc2178);
    apply!(sc2179);
    apply!(sc2180);
    apply!(sc2181);
    apply!(sc2182);
    apply!(sc2183);
    apply!(sc2184);
    apply!(sc2185);
    apply!(sc2186);
    apply!(sc2187);
    apply!(sc2188);
    apply!(sc2189);
    apply!(sc2190);
    apply!(sc2191);
    apply!(sc2192);
    apply!(sc2193);
    apply!(sc2194);
    apply!(sc2195);
    apply!(sc2196);
    apply!(sc2197);
    apply!(sc2198);
    apply!(sc2199);
    apply!(sc2200);
    apply!(sc2201);
    apply!(sc2202);
    apply!(sc2203);
    apply!(sc2204);
    apply!(sc2205);
    apply!(sc2206);
    apply!(sc2207);
    apply!(sc2208);
    apply!(sc2209);
    apply!(sc2210);
    apply!(sc2211);
    apply!(sc2212);
    apply!(sc2213);
    apply!(sc2214);
    apply!(sc2215);
    apply!(sc2216);
    apply!(sc2217);
    apply!(sc2218);
    apply!(sc2219);
    apply!(sc2220);
    apply!(sc2221);
    apply!(sc2222);
    apply!(sc2223);
    apply!(sc2224);
    apply!(sc2225);
    apply!(sc2226);
    apply!(sc2227);
    apply!(sc2228);
    apply!(sc2229);
    apply!(sc2230);
    apply!(sc2231);
    apply!(sc2232);
    apply!(sc2233);
    apply!(sc2234);
    apply!(sc2235);
    apply!(sc2236);
    apply!(sc2237);
    apply!(sc2238);
    apply!(sc2239);
    apply!(sc2240);
    apply!(sc2241);
    apply!(sc2242);
    apply!(sc2243);
    apply!(sc2244);
    apply!(sc2245);
    apply!(sc2246);
    apply!(sc2247);
    apply!(sc2248);
    apply!(sc2249);
    apply!(sc2250);
    apply!(sc2251);
    apply!(sc2252);
    apply!(sc2253);
    apply!(sc2254);
    apply!(sc2255);
    apply!(sc2256);
    apply!(sc2257);
    apply!(sc2258);
    apply!(sc2259);
    apply!(sc2260);
    apply!(sc2261);
    apply!(sc2262);
    apply!(sc2263);
    apply!(sc2264);
    apply!(sc2265);
    apply!(sc2266);
    apply!(sc2267);
    apply!(sc2268);
    apply!(sc2269);
    apply!(sc2270);
    apply!(sc2271);
    apply!(sc2272);
    apply!(sc2273);
    apply!(sc2274);
    apply!(sc2275);
    apply!(sc2276);
    apply!(sc2277);
    apply!(sc2278);
    apply!(sc2279);
    apply!(sc2280);
    apply!(sc2281);
    apply!(sc2282);
    apply!(sc2283);
    apply!(sc2284);
    apply!(sc2285);
    apply!(sc2286);
    apply!(sc2287);
    apply!(sc2288);
    apply!(sc2289);
    apply!(sc2290);
    apply!(sc2291);
    apply!(sc2292);
    apply!(sc2293);
    apply!(sc2294);
    apply!(sc2295);
    apply!(sc2296);
    apply!(sc2297);
    apply!(sc2298);
    apply!(sc2299);
    apply!(sc2300);
    apply!(sc2301);
    apply!(sc2302);
    apply!(sc2303);
    apply!(sc2304);
    apply!(sc2305);
    apply!(sc2306);
    apply!(sc2307);
    apply!(sc2308);
    apply!(sc2309);
    apply!(sc2310);
    apply!(sc2311);
    apply!(sc2312);
    apply!(sc2313);
    apply!(sc2314);
    apply!(sc2315);
    apply!(sc2316);
    apply!(sc2317);
    apply!(sc2318);
    apply!(sc2319);
    apply!(sc2320);
    apply!(sc2321);
    apply!(sc2322);
    apply!(sc2323);
    apply!(sc2324);
    apply!(sc2325);

    // Messages from masked rules must quote the user's text, not the filler.
    crate::linter::quoting::restore_masked_messages(source, &masked, &mut result);

    // Extended rules: determinism, idempotency, security, performance, portability, reliability
    // Plus inline suppression filtering and embedded program exclusion
    apply_extended_lint_rules(source, &mut result);

    result
}

include!("mod_lint_extended.rs");
include!("mod_lint_methods.rs");
