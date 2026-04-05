//! Adversarial template families for shell safety classifier training data.
//!
//! 100 parametric templates (25 per class) that generate realistic shell scripts
//! targeting each underrepresented safety class. Templates use `{PARAM}` placeholders
//! that are substituted from randomized pools to create diverse training examples.

/// A parametric template that generates scripts for a specific safety class.
#[derive(Debug, Clone)]
pub struct AdversarialTemplate {
    /// Template family identifier (e.g. "NONDET-RANDOM-ASSIGN")
    pub family: &'static str,
    /// Target safety class (0-4)
    pub target_class: u8,
    /// Template body with `{PARAM}` placeholders
    pub template: &'static str,
    /// Parameter slots for substitution
    pub params: &'static [ParamSlot],
}

/// A named parameter slot with a pool of possible values.
#[derive(Debug, Clone)]
pub struct ParamSlot {
    /// Placeholder name (without braces)
    pub name: &'static str,
    /// Pool of candidate values
    pub pool: &'static [&'static str],
}

// ── Context pools for realistic script wrapping ────────────────────────

pub const SHEBANGS: &[&str] = &["#!/bin/sh", "#!/bin/bash", "#!/usr/bin/env bash"];

pub const COMMENTS: &[&str] = &[
    "# Deployment script",
    "# Build automation",
    "# Cleanup routine",
    "# Configuration setup",
    "# Service management",
    "# Package installation",
    "# Database migration",
    "# Log rotation",
];

pub const SETUP_LINES: &[&str] = &["set -e", "set -u", "set -eu", "set -euo pipefail", ""];

pub const TRAILING_LINES: &[&str] = &["echo \"Done.\"", "exit 0", "echo \"Complete.\"", ""];

// ── Variable name pools ────────────────────────────────────────────────

const VARNAMES: &[&str] = &[
    "result", "value", "output", "token", "count", "idx", "status", "retval", "data", "item",
    "name", "code", "flag", "tmp", "buf",
];

const DIRNAMES: &[&str] = &[
    "/tmp/build",
    "/var/cache/app",
    "/opt/deploy",
    "/home/user/data",
    "/srv/app/logs",
    "/tmp/staging",
    "/var/lib/service",
    "/opt/release",
];

/// Directories that do NOT trigger SEC013 (/tmp/, /var/tmp/) or SEC014.
/// Used by class 1 (needs-quoting) and class 3 (non-idempotent) templates
/// to avoid cross-contamination into class 4 (unsafe).
const SAFE_DIRNAMES: &[&str] = &[
    "/opt/deploy",
    "/srv/data",
    "/opt/release",
    "/home/user/data",
    "/var/lib/service",
    "/opt/app/data",
    "/srv/cache",
    "/home/deploy/work",
];

const FILENAMES: &[&str] = &[
    "config.txt",
    "output.log",
    "data.json",
    "report.csv",
    "cache.db",
    "session.tmp",
    "state.dat",
    "results.xml",
];

const EXTENSIONS: &[&str] = &["log", "txt", "tmp", "bak", "dat", "csv", "json", "xml"];

const URLS: &[&str] = &[
    "https://example.com/pkg.tar.gz",
    "https://releases.example.org/v1.0/app",
    "https://cdn.example.net/download/tool",
    "https://mirror.example.com/archive.zip",
];

const HOSTS: &[&str] = &[
    "server1.example.com",
    "db.internal.net",
    "cache.local",
    "api.example.org",
    "worker-1.cluster.local",
];

const SECRETS: &[&str] = &[
    "sk_live_abc123def456ghi789",
    "AKIAIOSFODNN7EXAMPLE",
    "ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    "password123!@#",
];

const LINK_TARGETS: &[&str] = &[
    "/usr/local/bin/app",
    "/opt/app/current",
    "/etc/app/config",
    "/home/user/.config/tool",
    "/usr/bin/tool-v2",
];

const LINK_NAMES: &[&str] = &[
    "/usr/local/bin/app-link",
    "/opt/app/latest",
    "/etc/app/active",
    "/home/user/.local/bin/tool",
    "/usr/bin/tool",
];

// ── Class 2: Non-deterministic templates (25) ──────────────────────────

pub fn non_deterministic_templates() -> Vec<AdversarialTemplate> {
    vec![
        // $RANDOM templates (12)
        AdversarialTemplate {
            family: "NONDET-RANDOM-ASSIGN",
            target_class: 2,
            template: "{VAR}=$RANDOM\necho \"Generated: ${VAR}\"",
            params: &[ParamSlot { name: "VAR", pool: VARNAMES }],
        },
        AdversarialTemplate {
            family: "NONDET-RANDOM-ARITH",
            target_class: 2,
            template: "{VAR}=$(( $RANDOM % {MOD} ))\necho \"Value: ${VAR}\"",
            params: &[
                ParamSlot { name: "VAR", pool: VARNAMES },
                ParamSlot { name: "MOD", pool: &["100", "256", "1000", "65536", "10"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-RANDOM-COND",
            target_class: 2,
            template: "if [ $RANDOM -gt {THRESHOLD} ]; then\n  echo \"{MSG}\"\nfi",
            params: &[
                ParamSlot { name: "THRESHOLD", pool: &["16384", "10000", "32000", "5000", "20000"] },
                ParamSlot { name: "MSG", pool: &["selected", "chosen", "activated", "triggered", "enabled"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-RANDOM-LOOP",
            target_class: 2,
            template: "for i in 1 2 3; do\n  {VAR}=$RANDOM\n  echo \"Iteration $i: ${VAR}\"\ndone",
            params: &[ParamSlot { name: "VAR", pool: VARNAMES }],
        },
        AdversarialTemplate {
            family: "NONDET-RANDOM-PRINTF",
            target_class: 2,
            template: "printf \"%s-%d\\n\" \"{PREFIX}\" $RANDOM",
            params: &[
                ParamSlot { name: "PREFIX", pool: &["session", "token", "key", "id", "ref"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-RANDOM-FILENAME",
            target_class: 2,
            template: "{VAR}=\"/tmp/{PREFIX}_${RANDOM}.{EXT}\"\necho \"Temp file: ${VAR}\"",
            params: &[
                ParamSlot { name: "VAR", pool: VARNAMES },
                ParamSlot { name: "PREFIX", pool: &["build", "cache", "tmp", "work", "stage"] },
                ParamSlot { name: "EXT", pool: EXTENSIONS },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-RANDOM-CASE",
            target_class: 2,
            template: "case $(( $RANDOM % {CASES} )) in\n  0) echo \"{OPT1}\";;\n  1) echo \"{OPT2}\";;\n  *) echo \"{OPT3}\";;\nesac",
            params: &[
                ParamSlot { name: "CASES", pool: &["3", "4", "5", "2", "6"] },
                ParamSlot { name: "OPT1", pool: &["alpha", "first", "primary", "a", "option1"] },
                ParamSlot { name: "OPT2", pool: &["beta", "second", "backup", "b", "option2"] },
                ParamSlot { name: "OPT3", pool: &["gamma", "default", "fallback", "c", "option3"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-RANDOM-PORT",
            target_class: 2,
            template: "{VAR}=$(( $RANDOM + {BASE} ))\necho \"Listening on port ${VAR}\"",
            params: &[
                ParamSlot { name: "VAR", pool: &["port", "listen_port", "svc_port", "p", "http_port"] },
                ParamSlot { name: "BASE", pool: &["1024", "8000", "3000", "5000", "10000"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-RANDOM-SLEEP",
            target_class: 2,
            template: "{VAR}=$(( $RANDOM % {MAX} + 1 ))\nsleep \"${VAR}\"\necho \"Waited ${VAR}s\"",
            params: &[
                ParamSlot { name: "VAR", pool: &["delay", "wait_time", "interval", "pause", "backoff"] },
                ParamSlot { name: "MAX", pool: &["5", "10", "30", "60", "15"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-RANDOM-HEX",
            target_class: 2,
            template: "{VAR}=$(printf \"%04x\" $RANDOM)\necho \"ID: ${VAR}\"",
            params: &[ParamSlot { name: "VAR", pool: VARNAMES }],
        },
        AdversarialTemplate {
            family: "NONDET-RANDOM-ARRAY",
            target_class: 2,
            template: "{ITEMS}=\"{A} {B} {C}\"\n{VAR}=$RANDOM\necho \"Selected: ${VAR}\"",
            params: &[
                ParamSlot { name: "ITEMS", pool: &["options", "choices", "servers", "nodes", "targets"] },
                ParamSlot { name: "VAR", pool: VARNAMES },
                ParamSlot { name: "A", pool: &["red", "east", "primary", "fast", "hot"] },
                ParamSlot { name: "B", pool: &["blue", "west", "secondary", "slow", "cold"] },
                ParamSlot { name: "C", pool: &["green", "north", "tertiary", "medium", "warm"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-RANDOM-RETRY",
            target_class: 2,
            template: "{VAR}=0\nwhile [ \"${VAR}\" -lt {MAX} ]; do\n  {VAR}=$(( {VAR} + 1 ))\n  sleep $(( $RANDOM % 3 + 1 ))\n  echo \"Retry ${VAR}\"\ndone",
            params: &[
                ParamSlot { name: "VAR", pool: &["attempt", "tries", "retries", "count", "n"] },
                ParamSlot { name: "MAX", pool: &["3", "5", "10", "7", "4"] },
            ],
        },
        // $(date) templates (8)
        AdversarialTemplate {
            family: "NONDET-DATE-ASSIGN",
            target_class: 2,
            template: "{VAR}=$(date +\"{FMT}\")\necho \"Timestamp: ${VAR}\"",
            params: &[
                ParamSlot { name: "VAR", pool: &["ts", "timestamp", "now", "current_time", "dt"] },
                ParamSlot { name: "FMT", pool: &["%Y%m%d", "%s", "%Y-%m-%d %H:%M:%S", "%H:%M", "%Y%m%d%H%M%S"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-DATE-LOG",
            target_class: 2,
            template: "echo \"[$(date +\"{FMT}\")] {MSG}\"",
            params: &[
                ParamSlot { name: "FMT", pool: &["%Y-%m-%d %H:%M:%S", "%s", "%T", "%c", "%F %T"] },
                ParamSlot { name: "MSG", pool: &["Starting deployment", "Build began", "Process started", "Service initialized", "Task running"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-DATE-BACKUP",
            target_class: 2,
            template: "{VAR}=\"{PREFIX}_$(date +%Y%m%d_%H%M%S).{EXT}\"\necho \"Backup: ${VAR}\"",
            params: &[
                ParamSlot { name: "VAR", pool: &["backup", "archive", "snapshot", "dump", "save"] },
                ParamSlot { name: "PREFIX", pool: &["db", "config", "data", "logs", "state"] },
                ParamSlot { name: "EXT", pool: &["tar.gz", "sql", "bak", "dump", "zip"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-DATE-HEADER",
            target_class: 2,
            template: "echo \"========================================\"\necho \"Report generated: $(date)\"\necho \"========================================\"",
            params: &[],
        },
        AdversarialTemplate {
            family: "NONDET-DATE-EPOCH",
            target_class: 2,
            template: "{VAR}=$(date +%s)\necho \"Epoch: ${VAR}\"",
            params: &[
                ParamSlot { name: "VAR", pool: &["epoch", "unix_ts", "start_time", "begin", "mark"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-DATE-ELAPSED",
            target_class: 2,
            template: "{START}=$(date +%s)\necho \"Working...\"\n{END}=$(date +%s)\necho \"Elapsed: $(( {END} - {START} ))s\"",
            params: &[
                ParamSlot { name: "START", pool: &["t0", "start", "begin", "s", "before"] },
                ParamSlot { name: "END", pool: &["t1", "finish", "end", "e", "after"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-DATE-LOGFILE",
            target_class: 2,
            template: "{VAR}=\"/var/log/{APP}_$(date +%Y%m%d).log\"\necho \"Logging to: ${VAR}\"",
            params: &[
                ParamSlot { name: "VAR", pool: &["logfile", "log_path", "outfile", "dest", "target"] },
                ParamSlot { name: "APP", pool: &["myapp", "service", "daemon", "worker", "agent"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-DATE-BACKTICK",
            target_class: 2,
            template: "{VAR}=`date +%s`\necho \"Time: ${VAR}\"",
            params: &[
                ParamSlot { name: "VAR", pool: &["ts", "now", "t", "time_val", "stamp"] },
            ],
        },
        // Wildcard glob templates (5)
        AdversarialTemplate {
            family: "NONDET-GLOB-LS",
            target_class: 2,
            template: "for f in *.{EXT}; do\n  echo \"Processing: $f\"\ndone",
            params: &[
                ParamSlot { name: "EXT", pool: EXTENSIONS },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-GLOB-CAT",
            target_class: 2,
            template: "cat /var/log/*.{EXT}",
            params: &[
                ParamSlot { name: "EXT", pool: &["log", "txt", "err", "out", "warn"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-GLOB-WC",
            target_class: 2,
            template: "wc -l /tmp/*.{EXT}",
            params: &[
                ParamSlot { name: "EXT", pool: EXTENSIONS },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-GLOB-HEAD",
            target_class: 2,
            template: "head -n {N} /var/log/*.{EXT}",
            params: &[
                ParamSlot { name: "N", pool: &["1", "5", "10", "20", "50"] },
                ParamSlot { name: "EXT", pool: &["log", "txt", "err", "out", "csv"] },
            ],
        },
        AdversarialTemplate {
            family: "NONDET-GLOB-FORLOOP",
            target_class: 2,
            template: "for f in /tmp/{PREFIX}_*.{EXT}; do\n  echo \"File: $f\"\ndone",
            params: &[
                ParamSlot { name: "PREFIX", pool: &["build", "cache", "data", "run", "out"] },
                ParamSlot { name: "EXT", pool: EXTENSIONS },
            ],
        },
    ]
}

// ── Class 3: Non-idempotent templates (25) ─────────────────────────────

include!("adversarial_templates_non.rs");
