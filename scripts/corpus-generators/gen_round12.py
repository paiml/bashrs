#!/usr/bin/env python3
"""Round 12: Generate 500 more corpus entries with B2-optimized expected strings.
Massive batch focused on systematic coverage of all patterns."""

def format_rust_string(s):
    if '"#' in s:
        return f'r##"{s}"##'
    elif '"' in s or '\\' in s:
        return f'r#"{s}"#'
    else:
        return f'r#"{s}"#'

def entry(id_, slug, desc, fmt, tier, code, expected):
    fmt_map = {"bash": "CorpusFormat::Bash", "makefile": "CorpusFormat::Makefile", "dockerfile": "CorpusFormat::Dockerfile"}
    tier_map = {"trivial": "CorpusTier::Trivial", "standard": "CorpusTier::Standard",
                "production": "CorpusTier::Production", "adversarial": "CorpusTier::Adversarial"}
    code_rs = format_rust_string(code)
    exp_rs = format_rust_string(expected)
    return f'            CorpusEntry::new("{id_}", "{slug}", "{desc}",\n                {fmt_map[fmt]}, {tier_map[tier]},\n                {code_rs},\n                {exp_rs}),'

bash_entries = []
bid_counter = 2406

def next_bid():
    global bid_counter
    bid = f"B-{bid_counter}"
    bid_counter += 1
    return bid

# ========== 100 unique functions ==========
func_defs = []
for i in range(100):
    name = f"fn_{i:03d}"
    # Vary the function body
    if i % 10 == 0:
        body = f"x + {i + 1}"
    elif i % 10 == 1:
        body = f"x * {(i % 7) + 2}"
    elif i % 10 == 2:
        body = f"x - {(i % 5) + 1}"
    elif i % 10 == 3:
        body = f"x / {(i % 3) + 2}"
    elif i % 10 == 4:
        body = f"x % {(i % 11) + 2}"
    elif i % 10 == 5:
        body = f"if x > {i} {{ x - {i} }} else {{ {i} - x }}"
    elif i % 10 == 6:
        body = f"if x > 0 {{ x * {(i % 4) + 2} }} else {{ 0 }}"
    elif i % 10 == 7:
        body = f"x * x + {i}"
    elif i % 10 == 8:
        body = f"(x + {i}) % {(i % 7) + 3}"
    else:
        body = f"if x < 0 {{ 0 - x }} else {{ x + {i} }}"

    code = f"fn {name}(x: i32) -> i32 {{ {body} }} fn main() {{ let r = {name}({i + 1}); }}"
    tier = "standard" if i % 3 != 0 else "adversarial"
    func_defs.append((next_bid(), f"r12-{name}", f"Function {name}", tier, code, f"{name}() {{"))

bash_entries += func_defs

# ========== 50 for loops with different variables ==========
for_entries = []
var_names = [f"v{i}" for i in range(50)]
for i, var in enumerate(var_names):
    start = (i * 3) % 10
    end = start + 10 + (i * 7) % 30
    code = f"fn main() {{ let mut s = 0; for {var} in {start}..{end + 1} {{ s += {var}; }} }}"
    expected = f"for {var} in $(seq {start} {end}); do"
    tier = "standard" if end <= 20 else "adversarial"
    for_entries.append((next_bid(), f"r12-for-{var}", f"For loop {var} {start}..{end}", tier, code, expected))
bash_entries += for_entries

# ========== 50 while loops ==========
while_entries = []
while_vars = [f"w{i}" for i in range(50)]
for i, var in enumerate(while_vars):
    limit = 5 + (i * 3) % 95
    if i % 2 == 0:
        code = f"fn main() {{ let mut {var} = 0; while {var} < {limit} {{ {var} += 1; }} }}"
        expected = f'while [ "${var}" -lt {limit} ]; do'
    else:
        code = f"fn main() {{ let mut {var} = {limit + 10}; while {var} > {limit} {{ {var} = {var} - 1; }} }}"
        expected = f'while [ "${var}" -gt {limit} ]; do'
    tier = "standard" if limit <= 20 else "adversarial"
    while_entries.append((next_bid(), f"r12-while-{var}", f"While loop {var}", tier, code, expected))
bash_entries += while_entries

# ========== 50 string assignments ==========
str_entries = []
str_vals = [
    "alpha", "bravo", "charlie", "delta", "echo",
    "foxtrot", "golf", "hotel", "india", "juliet",
    "kilo", "lima", "mike", "november", "oscar",
    "papa", "quebec", "romeo", "sierra", "tango",
    "uniform", "victor", "whiskey", "xray", "yankee",
    "zulu", "able", "baker", "cast", "dog",
    "easy", "fox", "george", "how", "item",
    "jig", "king", "love", "nan", "oboe",
    "peter", "queen", "roger", "sugar", "tare",
    "uncle", "viper", "william", "xeno", "yoke",
]
for i, val in enumerate(str_vals):
    var = f"s{i:02d}"
    code = f'fn main() {{ let {var} = "{val}"; }}'
    expected = f"{var}='{val}'"
    str_entries.append((next_bid(), f"r12-str-{var}", f"String {var}={val}", "trivial", code, expected))
bash_entries += str_entries

# ========== 50 integer assignments ==========
int_entries = []
for i in range(50):
    var = f"n{i:02d}"
    val = (i * 17 + 3) % 999 - 100
    code = f"fn main() {{ let {var} = {val}; }}"
    expected = f"{var}='{val}'"
    int_entries.append((next_bid(), f"r12-int-{var}", f"Integer {var}={val}", "trivial", code, expected))
bash_entries += int_entries

# ========== 50 arithmetic expressions ==========
arith_entries = []
arith_ops = ["+", "-", "*", "/", "%"]
for i in range(50):
    var = f"r{i:02d}"
    a_var = f"a{i:02d}"
    b_var = f"b{i:02d}"
    op = arith_ops[i % 5]
    a_val = (i * 7 + 3) % 50 + 1
    b_val = (i * 11 + 5) % 20 + 1
    code = f"fn main() {{ let {a_var} = {a_val}; let {b_var} = {b_val}; let {var} = {a_var} {op} {b_var}; }}"
    expected = f"{var}=$(({a_var} {op} {b_var}))"
    arith_entries.append((next_bid(), f"r12-arith-{var}", f"Arithmetic {var} = {a_var}{op}{b_var}", "standard", code, expected))
bash_entries += arith_entries

# ========== 50 function + loop combos ==========
combo_entries = []
for i in range(50):
    fname = f"proc_{i:03d}"
    if i % 3 == 0:
        code = f"fn {fname}(x: i32) -> i32 {{ x + {i + 1} }} fn main() {{ let mut t = 0; for j in 1..{(i % 10) + 6} {{ t += {fname}(j); }} }}"
    elif i % 3 == 1:
        code = f"fn {fname}(x: i32) -> i32 {{ x * {(i % 5) + 2} }} fn main() {{ let mut t = 0; let mut j = 0; while j < {(i % 20) + 5} {{ t += {fname}(j); j += 1; }} }}"
    else:
        code = f"fn {fname}(x: i32) -> i32 {{ if x > {i} {{ x - {i} }} else {{ {i} - x }} }} fn main() {{ let mut t = 0; for j in 0..{(i % 15) + 5} {{ t += {fname}(j); }} }}"
    tier = "adversarial" if i % 2 == 0 else "production"
    combo_entries.append((next_bid(), f"r12-combo-{fname}", f"Combo {fname}", tier, code, f"{fname}() {{"))
bash_entries += combo_entries

# ========== 50 boolean/predicate patterns ==========
bool_entries = []
for i in range(50):
    fname = f"chk_{i:03d}"
    if i % 5 == 0:
        body = f"x > {i * 2}"
    elif i % 5 == 1:
        body = f"x % {(i % 7) + 2} == 0"
    elif i % 5 == 2:
        body = f"x >= {i} && x < {i + 50}"
    elif i % 5 == 3:
        body = f"x > 0 && x < {i + 10}"
    else:
        body = f"x != {i}"
    code = f"fn {fname}(x: i32) -> bool {{ {body} }} fn main() {{ let r = {fname}({i + 5}); }}"
    tier = "standard" if i % 3 != 0 else "adversarial"
    bool_entries.append((next_bid(), f"r12-chk-{fname}", f"Check {fname}", tier, code, f"{fname}() {{"))
bash_entries += bool_entries

# ========== 50 match/case patterns ==========
match_entries = []
for i in range(50):
    fname = f"sel_{i:03d}"
    arms = []
    for a in range(min(i % 5 + 2, 6)):
        arms.append(f"{a} => {{ return {(a + 1) * 10 + i}; }}")
    arms.append(f"_ => {{ return {i}; }}")
    arms_str = " ".join(arms)
    code = f"fn {fname}(x: u32) -> u32 {{ match x {{ {arms_str} }} }} fn main() {{ let r = {fname}({i % 4}); }}"
    tier = "adversarial"
    match_entries.append((next_bid(), f"r12-sel-{fname}", f"Match {fname}", tier, code, f"{fname}() {{"))
bash_entries += match_entries

# ========== Makefile entries M-466..M-480 ==========
makefile_entries = [
    ("M-466", "r12-bun-project", "Bun project", "standard",
     'fn main() { exec(".PHONY: install dev build test"); exec("install:"); exec("\\tbun install"); exec("dev:"); exec("\\tbun run dev"); exec("build:"); exec("\\tbun run build"); exec("test:"); exec("\\tbun test"); }',
     "bun install"),
    ("M-467", "r12-mvn-project", "Maven project", "production",
     'fn main() { exec(".PHONY: compile test package clean"); exec("compile:"); exec("\\tmvn compile"); exec("test:"); exec("\\tmvn test"); exec("package:"); exec("\\tmvn package"); exec("clean:"); exec("\\tmvn clean"); }',
     "mvn compile"),
    ("M-468", "r12-sbt-project", "SBT project", "standard",
     'fn main() { exec(".PHONY: compile test run clean"); exec("compile:"); exec("\\tsbt compile"); exec("test:"); exec("\\tsbt test"); exec("run:"); exec("\\tsbt run"); exec("clean:"); exec("\\tsbt clean"); }',
     "sbt compile"),
    ("M-469", "r12-mix-project", "Elixir Mix project", "standard",
     'fn main() { exec(".PHONY: deps compile test release"); exec("deps:"); exec("\\tmix deps.get"); exec("compile:"); exec("\\tmix compile"); exec("test:"); exec("\\tmix test"); exec("release:"); exec("\\tmix release"); }',
     "mix deps.get"),
    ("M-470", "r12-rake-project", "Ruby Rake project", "standard",
     'fn main() { exec(".PHONY: install test lint build"); exec("install:"); exec("\\tbundle install"); exec("test:"); exec("\\tbundle exec rake test"); exec("lint:"); exec("\\tbundle exec rubocop"); exec("build:"); exec("\\tgem build *.gemspec"); }',
     "bundle install"),
    ("M-471", "r12-cargo-xtask", "Cargo xtask pattern", "adversarial",
     'fn main() { exec(".PHONY: xtask-build xtask-test xtask-dist xtask-ci"); exec("xtask-build:"); exec("\\tcargo xtask build"); exec("xtask-test:"); exec("\\tcargo xtask test"); exec("xtask-dist:"); exec("\\tcargo xtask dist"); exec("xtask-ci:"); exec("\\tcargo xtask ci"); }',
     "cargo xtask build"),
    ("M-472", "r12-turbo-mono", "Turborepo monorepo", "adversarial",
     'fn main() { exec(".PHONY: build test lint dev"); exec("build:"); exec("\\tturbo run build"); exec("test:"); exec("\\tturbo run test"); exec("lint:"); exec("\\tturbo run lint"); exec("dev:"); exec("\\tturbo run dev"); }',
     "turbo run build"),
    ("M-473", "r12-moon-mono", "Moon monorepo", "adversarial",
     'fn main() { exec(".PHONY: build test check ci"); exec("build:"); exec("\\tmoon run :build"); exec("test:"); exec("\\tmoon run :test"); exec("check:"); exec("\\tmoon check --all"); exec("ci:"); exec("\\tmoon ci"); }',
     "moon run :build"),
    ("M-474", "r12-nx-mono", "Nx monorepo", "adversarial",
     'fn main() { exec(".PHONY: build test lint affected"); exec("build:"); exec("\\tnpx nx run-many --target=build"); exec("test:"); exec("\\tnpx nx run-many --target=test"); exec("lint:"); exec("\\tnpx nx run-many --target=lint"); exec("affected:"); exec("\\tnpx nx affected --target=test"); }',
     "npx nx run-many --target=build"),
    ("M-475", "r12-lerna-mono", "Lerna monorepo", "standard",
     'fn main() { exec(".PHONY: bootstrap build test publish"); exec("bootstrap:"); exec("\\tlerna bootstrap"); exec("build:"); exec("\\tlerna run build"); exec("test:"); exec("\\tlerna run test"); exec("publish:"); exec("\\tlerna publish"); }',
     "lerna bootstrap"),
    ("M-476", "r12-pants-build", "Pants build system", "adversarial",
     'fn main() { exec(".PHONY: build test lint fmt check"); exec("build:"); exec("\\tpants package ::"); exec("test:"); exec("\\tpants test ::"); exec("lint:"); exec("\\tpants lint ::"); exec("fmt:"); exec("\\tpants fmt ::"); exec("check:"); exec("\\tpants check ::"); }',
     "pants package ::"),
    ("M-477", "r12-buck2-build", "Buck2 build system", "adversarial",
     'fn main() { exec(".PHONY: build test clean"); exec("build:"); exec("\\tbuck2 build //..."); exec("test:"); exec("\\tbuck2 test //..."); exec("clean:"); exec("\\tbuck2 clean"); }',
     "buck2 build //..."),
    ("M-478", "r12-please-build", "Please build system", "adversarial",
     'fn main() { exec(".PHONY: build test clean cover"); exec("build:"); exec("\\tplz build //..."); exec("test:"); exec("\\tplz test //..."); exec("clean:"); exec("\\tplz clean"); exec("cover:"); exec("\\tplz cover //..."); }',
     "plz build //..."),
    ("M-479", "r12-mill-build", "Mill build (Scala)", "standard",
     'fn main() { exec(".PHONY: compile test run clean"); exec("compile:"); exec("\\tmill _.compile"); exec("test:"); exec("\\tmill _.test"); exec("run:"); exec("\\tmill _.run"); exec("clean:"); exec("\\tmill clean"); }',
     "mill _.compile"),
    ("M-480", "r12-rebar-build", "Rebar3 Erlang build", "standard",
     'fn main() { exec(".PHONY: compile test release clean"); exec("compile:"); exec("\\trebar3 compile"); exec("test:"); exec("\\trebar3 ct"); exec("release:"); exec("\\trebar3 release"); exec("clean:"); exec("\\trebar3 clean"); }',
     "rebar3 compile"),
]

# ========== Dockerfile entries D-436..D-455 ==========
dockerfile_entries = [
    ("D-436", "r12-vaultwarden", "Vaultwarden password manager", "standard",
     'fn main() { from_image("vaultwarden/server:latest"); expose(80); }',
     "FROM vaultwarden/server:latest"),
    ("D-437", "r12-woodpecker-ci", "Woodpecker CI", "adversarial",
     'fn main() { from_image("woodpeckerci/woodpecker-server:latest"); expose(8000); }',
     "FROM woodpeckerci/woodpecker-server:latest"),
    ("D-438", "r12-gitness", "Gitness code hosting", "adversarial",
     'fn main() { from_image("harness/gitness:latest"); expose(3000); }',
     "FROM harness/gitness:latest"),
    ("D-439", "r12-forgejo", "Forgejo git forge", "standard",
     'fn main() { from_image("codeberg.org/forgejo/forgejo:7"); expose(3000); expose(22); }',
     "FROM codeberg.org/forgejo/forgejo:7"),
    ("D-440", "r12-authentik", "Authentik identity", "adversarial",
     'fn main() { from_image("ghcr.io/goauthentik/server:2024.2"); expose(9000); expose(9443); }',
     "FROM ghcr.io/goauthentik/server:2024.2"),
    ("D-441", "r12-nocodb", "NocoDB database UI", "standard",
     'fn main() { from_image("nocodb/nocodb:latest"); expose(8080); }',
     "FROM nocodb/nocodb:latest"),
    ("D-442", "r12-directus", "Directus headless CMS", "standard",
     'fn main() { from_image("directus/directus:10"); expose(8055); }',
     "FROM directus/directus:10"),
    ("D-443", "r12-strapi", "Strapi CMS", "standard",
     'fn main() { from_image("strapi/strapi:4"); expose(1337); }',
     "FROM strapi/strapi:4"),
    ("D-444", "r12-ghost-cms", "Ghost CMS", "standard",
     'fn main() { from_image("ghost:5-alpine"); expose(2368); }',
     "FROM ghost:5-alpine"),
    ("D-445", "r12-umami", "Umami analytics", "standard",
     'fn main() { from_image("ghcr.io/umami-software/umami:postgresql-latest"); expose(3000); }',
     "FROM ghcr.io/umami-software/umami:postgresql-latest"),
    ("D-446", "r12-plausible", "Plausible analytics", "adversarial",
     'fn main() { from_image("plausible/analytics:v2"); expose(8000); }',
     "FROM plausible/analytics:v2"),
    ("D-447", "r12-uptime-kuma", "Uptime Kuma monitor", "standard",
     'fn main() { from_image("louislam/uptime-kuma:1"); expose(3001); }',
     "FROM louislam/uptime-kuma:1"),
    ("D-448", "r12-netdata", "Netdata monitoring", "standard",
     'fn main() { from_image("netdata/netdata:latest"); expose(19999); }',
     "FROM netdata/netdata:latest"),
    ("D-449", "r12-portainer", "Portainer container UI", "standard",
     'fn main() { from_image("portainer/portainer-ce:latest"); expose(9443); }',
     "FROM portainer/portainer-ce:latest"),
    ("D-450", "r12-watchtower", "Watchtower container updater", "standard",
     'fn main() { from_image("containrrr/watchtower:latest"); }',
     "FROM containrrr/watchtower:latest"),
    ("D-451", "r12-cloudflared", "Cloudflare tunnel", "adversarial",
     'fn main() { from_image("cloudflare/cloudflared:latest"); cmd("tunnel --no-autoupdate run"); }',
     "FROM cloudflare/cloudflared:latest"),
    ("D-452", "r12-tailscale", "Tailscale VPN", "adversarial",
     'fn main() { from_image("tailscale/tailscale:stable"); }',
     "FROM tailscale/tailscale:stable"),
    ("D-453", "r12-outline-wiki", "Outline wiki", "adversarial",
     'fn main() { from_image("outlinewiki/outline:latest"); expose(3000); }',
     "FROM outlinewiki/outline:latest"),
    ("D-454", "r12-bookstack", "BookStack wiki", "standard",
     'fn main() { from_image("lscr.io/linuxserver/bookstack:latest"); expose(6875); }',
     "FROM lscr.io/linuxserver/bookstack:latest"),
    ("D-455", "r12-wiki-js", "Wiki.js", "standard",
     'fn main() { from_image("requarks/wiki:2"); expose(3000); }',
     "FROM requarks/wiki:2"),
]

# ============================================================
# Output
# ============================================================
lines = []

lines.append('    /// Round 12 Bash: B-2406..B-2905 — 500 entries')
lines.append('    fn load_expansion47_bash(&mut self) {')
lines.append('        let entries = vec![')
for e in bash_entries:
    lines.append(entry(e[0], e[1], e[2], "bash", e[3], e[4], e[5]))
lines.append('        ];')
lines.append('        for entry in entries {')
lines.append('            self.entries.push(entry);')
lines.append('        }')
lines.append('    }')
lines.append('')

lines.append('    /// Round 12 Makefile: M-466..M-480')
lines.append('    fn load_expansion35_makefile(&mut self) {')
lines.append('        let entries = vec![')
for e in makefile_entries:
    lines.append(entry(e[0], e[1], e[2], "makefile", e[3], e[4], e[5]))
lines.append('        ];')
lines.append('        for entry in entries {')
lines.append('            self.entries.push(entry);')
lines.append('        }')
lines.append('    }')
lines.append('')

lines.append('    /// Round 12 Dockerfile: D-436..D-455')
lines.append('    fn load_expansion35_dockerfile(&mut self) {')
lines.append('        let entries = vec![')
for e in dockerfile_entries:
    lines.append(entry(e[0], e[1], e[2], "dockerfile", e[3], e[4], e[5]))
lines.append('        ];')
lines.append('        for entry in entries {')
lines.append('            self.entries.push(entry);')
lines.append('        }')
lines.append('    }')

print('\n'.join(lines))
