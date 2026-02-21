#!/usr/bin/env python3
"""Fix Makefile and Dockerfile corpus entries to use Rust input format."""

import re
import sys

def format_rust_string(s):
    if '"#' in s:
        return 'r##"' + s + '"##'
    return 'r#"' + s + '"#'

# Map old M/D entries to proper Rust-input entries
# Format: (id, name, desc, format, rust_input, expected_contains)
REPLACEMENTS = {
    # Round 1 Makefiles
    "M-15900": ("M-15900", "make-pathological-quote", "Makefile: shell quoting in build recipe",
        "Makefile",
        r'''fn build_flags(debug: i64) -> i64 { if debug == 1 { return 1; } return 0; }
fn main() { let cc: &str = "gcc"; let cflags: &str = "-Wall -Wextra -O2"; let src: &str = "main.c"; println!("build: {} {} {}", cc, cflags, src); }''',
        "build:"),

    "M-15901": ("M-15901", "make-nested-var-expand", "Makefile: nested variable expansion",
        "Makefile",
        r'''fn resolve_prefix(custom: i64) -> i64 { if custom != 0 { return custom; } return 1; }
fn main() { let prefix: &str = "/usr/local"; let bindir: &str = "bin"; let libdir: &str = "lib"; println!("prefix={} bin={} lib={}", prefix, bindir, libdir); }''',
        "prefix="),

    "M-15902": ("M-15902", "make-conditional-complex", "Makefile: conditional compilation flags",
        "Makefile",
        r'''fn select_flags(debug: i64, platform: i64) -> i64 {
    if debug == 1 { return 0; }
    if platform == 1 { return 10; }
    return 20;
}
fn main() { let flags: i64 = select_flags(1, 0); println!("flags={}", flags); }''',
        "select_flags()"),

    # Round 1 Dockerfiles
    "D-15903": ("D-15903", "docker-multistage-build", "Dockerfile: multi-stage build simulation",
        "Dockerfile",
        r'''fn build_stage(src: i64) -> i64 { return src * 2; }
fn runtime_stage(artifact: i64) -> i64 { return artifact + 1; }
fn main() { let built: i64 = build_stage(42); let app: i64 = runtime_stage(built); println!("app={}", app); }''',
        "build_stage()"),

    "D-15904": ("D-15904", "docker-arg-env-layers", "Dockerfile: ARG/ENV layer simulation",
        "Dockerfile",
        r'''fn resolve_arg(val: i64, default: i64) -> i64 { if val != 0 { return val; } return default; }
fn main() { let version: i64 = resolve_arg(0, 100); let port: i64 = resolve_arg(3000, 8080); println!("v={} port={}", version, port); }''',
        "resolve_arg()"),

    "D-15905": ("D-15905", "docker-test-stage", "Dockerfile: test stage between build and runtime",
        "Dockerfile",
        r'''fn run_tests(code: i64) -> i64 { if code > 0 { return 0; } return 1; }
fn build_prod(tested: i64) -> i64 { if tested == 0 { return 1; } return 0; }
fn main() { let t: i64 = run_tests(42); let p: i64 = build_prod(t); println!("test={} prod={}", t, p); }''',
        "run_tests()"),

    # Round 2 Makefiles
    "M-15929": ("M-15929", "make-shell-fn-quote", "Makefile: shell function with quoting patterns",
        "Makefile",
        r'''fn compile(src: i64, flags: i64) -> i64 { return src + flags; }
fn link(objs: [i64; 4], count: i64) -> i64 { let mut total: i64 = 0; let mut i: i64 = 0; while i < count { total = total + objs[i]; i = i + 1; } return total; }
fn main() { let o1: i64 = compile(1, 10); let o2: i64 = compile(2, 10); let objs: [i64; 4] = [o1, o2, 0, 0]; let binary: i64 = link(objs, 2); println!("binary={}", binary); }''',
        "compile()"),

    "M-15930": ("M-15930", "make-recursive-submake", "Makefile: recursive sub-directory build",
        "Makefile",
        r'''fn build_subdir(dir_id: i64) -> i64 { println!("build dir {}", dir_id); return dir_id * 10; }
fn main() { let r1: i64 = build_subdir(1); let r2: i64 = build_subdir(2); let r3: i64 = build_subdir(3); println!("all: {} {} {}", r1, r2, r3); }''',
        "build_subdir()"),

    "M-15931": ("M-15931", "make-auto-deps", "Makefile: automatic dependency generation",
        "Makefile",
        r'''fn gen_deps(src_id: i64) -> i64 { return src_id * 100 + 1; }
fn compile_with_deps(src: i64, dep: i64) -> i64 { return src + dep; }
fn main() { let d1: i64 = gen_deps(1); let d2: i64 = gen_deps(2); let o1: i64 = compile_with_deps(1, d1); let o2: i64 = compile_with_deps(2, d2); println!("o1={} o2={}", o1, o2); }''',
        "gen_deps()"),

    # Round 2 Dockerfiles
    "D-15932": ("D-15932", "docker-chained-run", "Dockerfile: chained RUN layer optimization",
        "Dockerfile",
        r'''fn install_deps(count: i64) -> i64 { return count * 5; }
fn cleanup(installed: i64) -> i64 { return installed - 2; }
fn main() { let deps: i64 = install_deps(10); let cleaned: i64 = cleanup(deps); println!("deps={} after_cleanup={}", deps, cleaned); }''',
        "install_deps()"),

    "D-15933": ("D-15933", "docker-cache-optimize", "Dockerfile: build cache optimization",
        "Dockerfile",
        r'''fn cache_layer(layer_id: i64, changed: i64) -> i64 { if changed == 0 { return layer_id; } return layer_id + 1000; }
fn main() { let l1: i64 = cache_layer(1, 0); let l2: i64 = cache_layer(2, 1); let l3: i64 = cache_layer(3, 0); println!("l1={} l2={} l3={}", l1, l2, l3); }''',
        "cache_layer()"),

    "D-15934": ("D-15934", "docker-onbuild-env", "Dockerfile: ONBUILD triggers and ENV layering",
        "Dockerfile",
        r'''fn onbuild_trigger(base: i64, trigger: i64) -> i64 { return base + trigger; }
fn set_env(key: i64, val: i64) -> i64 { return key * 100 + val; }
fn main() { let base: i64 = onbuild_trigger(10, 5); let env1: i64 = set_env(1, 42); let env2: i64 = set_env(2, 99); println!("base={} e1={} e2={}", base, env1, env2); }''',
        "onbuild_trigger()"),

    # Round 3 Makefiles
    "M-15957": ("M-15957", "make-multiline-recipe", "Makefile: multiline recipe with version embedding",
        "Makefile",
        r'''fn embed_version(major: i64, minor: i64, patch: i64) -> i64 { return major * 10000 + minor * 100 + patch; }
fn build_with_version(src: i64, ver: i64) -> i64 { return src * 100000 + ver; }
fn main() { let ver: i64 = embed_version(1, 2, 3); let result: i64 = build_with_version(42, ver); println!("ver={} result={}", ver, result); }''',
        "embed_version()"),

    "M-15958": ("M-15958", "make-phony-pattern", "Makefile: pattern rule generation",
        "Makefile",
        r'''fn gen_target(src_id: i64, suffix: i64) -> i64 { return src_id * 10 + suffix; }
fn main() { let t1: i64 = gen_target(1, 1); let t2: i64 = gen_target(2, 2); let t3: i64 = gen_target(3, 1); println!("targets: {} {} {}", t1, t2, t3); }''',
        "gen_target()"),

    "M-15959": ("M-15959", "make-target-specific-var", "Makefile: target-specific variable assignments",
        "Makefile",
        r'''fn debug_flags() -> i64 { return 1; }
fn release_flags() -> i64 { return 2; }
fn build(flags: i64, src: i64) -> i64 { return flags * 1000 + src; }
fn main() { let d: i64 = build(debug_flags(), 42); let r: i64 = build(release_flags(), 42); println!("debug={} release={}", d, r); }''',
        "debug_flags()"),

    # Round 3 Dockerfiles
    "D-15960": ("D-15960", "docker-4stage-pipeline", "Dockerfile: 4-stage build pipeline",
        "Dockerfile",
        r'''fn plan(src: i64) -> i64 { return src; }
fn cache_deps(planned: i64) -> i64 { return planned + 10; }
fn build_app(cached: i64) -> i64 { return cached * 2; }
fn package(built: i64) -> i64 { return built + 1; }
fn main() { let s1: i64 = plan(5); let s2: i64 = cache_deps(s1); let s3: i64 = build_app(s2); let s4: i64 = package(s3); println!("plan={} cache={} build={} pkg={}", s1, s2, s3, s4); }''',
        "plan()"),

    "D-15961": ("D-15961", "docker-conditional-build", "Dockerfile: conditional build based on environment",
        "Dockerfile",
        r'''fn select_deps(env: i64) -> i64 { if env == 1 { return 100; } return 50; }
fn build_for_env(deps: i64, env: i64) -> i64 { return deps + env * 10; }
fn main() { let prod_deps: i64 = select_deps(1); let dev_deps: i64 = select_deps(0); let prod: i64 = build_for_env(prod_deps, 1); let dev: i64 = build_for_env(dev_deps, 0); println!("prod={} dev={}", prod, dev); }''',
        "select_deps()"),

    "D-15962": ("D-15962", "docker-security-harden", "Dockerfile: security hardening with non-root user",
        "Dockerfile",
        r'''fn strip_binary(size: i64) -> i64 { return size / 3; }
fn set_user(uid: i64) -> i64 { if uid == 0 { return 65534; } return uid; }
fn main() { let binary: i64 = strip_binary(9000); let user: i64 = set_user(0); let user2: i64 = set_user(1000); println!("binary={} user={} user2={}", binary, user, user2); }''',
        "strip_binary()"),

    # Round 4 Makefiles
    "M-15985": ("M-15985", "make-cross-compile", "Makefile: cross-compilation targets",
        "Makefile",
        r'''fn cross_build(os: i64, arch: i64, src: i64) -> i64 { return os * 10000 + arch * 100 + src; }
fn main() { let linux_amd64: i64 = cross_build(1, 1, 42); let linux_arm64: i64 = cross_build(1, 2, 42); let darwin_amd64: i64 = cross_build(2, 1, 42); println!("linux_amd64={} linux_arm64={} darwin_amd64={}", linux_amd64, linux_arm64, darwin_amd64); }''',
        "cross_build()"),

    "M-15986": ("M-15986", "make-docker-compose", "Makefile: Docker Compose orchestration",
        "Makefile",
        r'''fn compose_up(env: i64, project: i64) -> i64 { return env * 100 + project; }
fn compose_down(handle: i64) -> i64 { return handle; }
fn main() { let dev: i64 = compose_up(1, 10); let prod: i64 = compose_up(2, 20); compose_down(dev); println!("dev={} prod={}", dev, prod); }''',
        "compose_up()"),

    "M-15987": ("M-15987", "make-c-library", "Makefile: C library with static and shared targets",
        "Makefile",
        r'''fn build_static(objs: [i64; 4], count: i64) -> i64 { let mut total: i64 = 0; let mut i: i64 = 0; while i < count { total = total + objs[i]; i = i + 1; } return total; }
fn build_shared(objs: [i64; 4], count: i64) -> i64 { let mut total: i64 = 0; let mut i: i64 = 0; while i < count { total = total + objs[i] * 10; i = i + 1; } return total; }
fn main() { let objs: [i64; 4] = [1, 2, 3, 4]; let s: i64 = build_static(objs, 4); let d: i64 = build_shared(objs, 4); println!("static={} shared={}", s, d); }''',
        "build_static()"),

    # Round 4 Dockerfiles
    "D-15988": ("D-15988", "docker-monorepo-build", "Dockerfile: monorepo multi-service build",
        "Dockerfile",
        r'''fn build_shared() -> i64 { return 100; }
fn build_api(shared: i64) -> i64 { return shared + 10; }
fn build_web(shared: i64) -> i64 { return shared + 20; }
fn main() { let shared: i64 = build_shared(); let api: i64 = build_api(shared); let web: i64 = build_web(shared); println!("shared={} api={} web={}", shared, api, web); }''',
        "build_shared()"),

    "D-15989": ("D-15989", "docker-distroless", "Dockerfile: distroless minimal container",
        "Dockerfile",
        r'''fn static_build(src: i64) -> i64 { return src * 3; }
fn strip_debug(binary: i64) -> i64 { return binary / 2; }
fn main() { let built: i64 = static_build(100); let stripped: i64 = strip_debug(built); println!("built={} stripped={}", built, stripped); }''',
        "static_build()"),

    "D-15990": ("D-15990", "docker-init-system", "Dockerfile: init system with signal handling",
        "Dockerfile",
        r'''fn init_process(pid: i64) -> i64 { return 1; }
fn forward_signal(init: i64, sig: i64) -> i64 { return init * 100 + sig; }
fn healthcheck(port: i64) -> i64 { if port > 0 { return 0; } return 1; }
fn main() { let init: i64 = init_process(1); let fwd: i64 = forward_signal(init, 15); let health: i64 = healthcheck(8000); println!("init={} fwd={} health={}", init, fwd, health); }''',
        "init_process()"),
}

def main():
    with open("rash/src/corpus/registry.rs", "r") as f:
        content = f.read()

    for entry_id, (new_id, name, desc, fmt, rust_input, expected) in REPLACEMENTS.items():
        fmt_str = f"CorpusFormat::{fmt}"
        # Find the entry and replace it
        # Pattern: from the entry start to the next entry or closing
        pattern = re.compile(
            rf'self\.entries\.push\(CorpusEntry::new\("{re.escape(entry_id)}".*?\)\);',
            re.DOTALL
        )

        rust_code = format_rust_string(rust_input)
        expected_code = format_rust_string(expected)

        replacement = f'self.entries.push(CorpusEntry::new("{new_id}", "{name}", "{desc}",\n            {fmt_str}, CorpusTier::Adversarial,\n            {rust_code},\n            {expected_code}));'

        new_content, count = pattern.subn(replacement, content, count=1)
        if count == 0:
            print(f"WARNING: Could not find {entry_id}", file=sys.stderr)
        else:
            content = new_content
            print(f"Fixed {entry_id}")

    with open("rash/src/corpus/registry.rs", "w") as f:
        f.write(content)

    print(f"\nFixed {len(REPLACEMENTS)} entries")

if __name__ == "__main__":
    main()
