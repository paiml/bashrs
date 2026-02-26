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

pub fn non_idempotent_templates() -> Vec<AdversarialTemplate> {
    vec![
        // mkdir without -p (8)
        AdversarialTemplate {
            family: "NONIDEM-MKDIR-SIMPLE",
            target_class: 3,
            template: "mkdir \"{DIR}\"",
            params: &[ParamSlot {
                name: "DIR",
                pool: SAFE_DIRNAMES,
            }],
        },
        AdversarialTemplate {
            family: "NONIDEM-MKDIR-NESTED",
            target_class: 3,
            template: "mkdir \"{DIR}/sub\"\necho \"Created directory\"",
            params: &[ParamSlot {
                name: "DIR",
                pool: SAFE_DIRNAMES,
            }],
        },
        AdversarialTemplate {
            family: "NONIDEM-MKDIR-VAR",
            target_class: 3,
            template: "echo \"Setting up {DIR}\"\nmkdir \"{DIR}\"",
            params: &[ParamSlot {
                name: "DIR",
                pool: SAFE_DIRNAMES,
            }],
        },
        AdversarialTemplate {
            family: "NONIDEM-MKDIR-MULTI",
            target_class: 3,
            template: "mkdir \"{DIR1}\"\nmkdir \"{DIR2}\"",
            params: &[
                ParamSlot {
                    name: "DIR1",
                    pool: SAFE_DIRNAMES,
                },
                ParamSlot {
                    name: "DIR2",
                    pool: &[
                        "/opt/data",
                        "/srv/cache",
                        "/opt/work",
                        "/var/lib/run",
                        "/srv/output",
                    ],
                },
            ],
        },
        AdversarialTemplate {
            family: "NONIDEM-MKDIR-LOOP",
            target_class: 3,
            template: "mkdir \"/opt/{A}\"\nmkdir \"/opt/{B}\"\nmkdir \"/opt/{C}\"",
            params: &[
                ParamSlot {
                    name: "A",
                    pool: &["logs", "data", "cache", "tmp", "build"],
                },
                ParamSlot {
                    name: "B",
                    pool: &["output", "state", "run", "work", "stage"],
                },
                ParamSlot {
                    name: "C",
                    pool: &["backup", "archive", "export", "dist", "pkg"],
                },
            ],
        },
        AdversarialTemplate {
            family: "NONIDEM-MKDIR-INSTALL",
            target_class: 3,
            template:
                "echo \"Installing to {DIR}\"\nmkdir \"{DIR}\"\nmkdir \"{DIR}/bin\"\necho \"Done.\"",
            params: &[ParamSlot {
                name: "DIR",
                pool: &[
                    "/opt/myapp",
                    "/usr/local/app",
                    "/home/user/tool",
                    "/opt/service",
                    "/srv/deploy",
                ],
            }],
        },
        AdversarialTemplate {
            family: "NONIDEM-MKDIR-DEPLOY",
            target_class: 3,
            template: "mkdir \"{DIR}\"\nmkdir \"{DIR}/config\"\necho \"Deployed.\"",
            params: &[ParamSlot {
                name: "DIR",
                pool: SAFE_DIRNAMES,
            }],
        },
        AdversarialTemplate {
            family: "NONIDEM-MKDIR-CONDITIONAL",
            target_class: 3,
            template: "if [ \"{MODE}\" = \"init\" ]; then\n  mkdir \"{DIR}\"\nfi",
            params: &[
                ParamSlot {
                    name: "MODE",
                    pool: &["init", "setup", "install", "create", "bootstrap"],
                },
                ParamSlot {
                    name: "DIR",
                    pool: SAFE_DIRNAMES,
                },
            ],
        },
        // rm without -f (9)
        AdversarialTemplate {
            family: "NONIDEM-RM-FILE",
            target_class: 3,
            template: "rm \"{DIR}/{FILE}\"",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &[
                        "/opt/data",
                        "/srv/logs",
                        "/home/user",
                        "/var/lib/app",
                        "/opt/cache",
                    ],
                },
                ParamSlot {
                    name: "FILE",
                    pool: FILENAMES,
                },
            ],
        },
        AdversarialTemplate {
            family: "NONIDEM-RM-DIR",
            target_class: 3,
            template: "rm -r \"{DIR}\"",
            params: &[ParamSlot {
                name: "DIR",
                pool: SAFE_DIRNAMES,
            }],
        },
        AdversarialTemplate {
            family: "NONIDEM-RM-MULTI",
            target_class: 3,
            template: "rm \"{DIR}/{FILE1}\"\nrm \"{DIR}/{FILE2}\"",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/data", "/srv/logs", "/var/lib/app", "/home/user/work"],
                },
                ParamSlot {
                    name: "FILE1",
                    pool: FILENAMES,
                },
                ParamSlot {
                    name: "FILE2",
                    pool: &["lock.pid", "socket.sock", "temp.dat", "old.bak"],
                },
            ],
        },
        AdversarialTemplate {
            family: "NONIDEM-RM-VAR",
            target_class: 3,
            template: "DEST=\"{DIR}/{FILE}\"\nrm \"${DEST}\"",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/data", "/srv/logs", "/var/lib/app", "/home/user/work"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: FILENAMES,
                },
            ],
        },
        AdversarialTemplate {
            family: "NONIDEM-RM-GLOB",
            target_class: 3,
            template: "rm \"{DIR}\"/*.{EXT}",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/data", "/srv/logs", "/var/lib/app", "/home/user/work"],
                },
                ParamSlot {
                    name: "EXT",
                    pool: &["tmp", "bak", "log", "old", "cache"],
                },
            ],
        },
        AdversarialTemplate {
            family: "NONIDEM-RM-CLEANUP",
            target_class: 3,
            template: "echo \"Cleaning up\"\nrm \"{DIR}/{FILE}\"\necho \"Clean.\"",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/data", "/srv/logs", "/var/lib/app", "/home/user/work"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: FILENAMES,
                },
            ],
        },
        AdversarialTemplate {
            family: "NONIDEM-RM-CACHE",
            target_class: 3,
            template: "rm -r \"{DIR}/cache\"\necho \"Cache cleared.\"",
            params: &[ParamSlot {
                name: "DIR",
                pool: &["/var/lib", "/opt/app", "/home/user/.local", "/srv", "/opt"],
            }],
        },
        AdversarialTemplate {
            family: "NONIDEM-RM-CONDITIONAL",
            target_class: 3,
            template: "if [ \"{ACTION}\" = \"clean\" ]; then\n  rm \"{DIR}/{FILE}\"\nfi",
            params: &[
                ParamSlot {
                    name: "ACTION",
                    pool: &["clean", "reset", "purge", "clear", "wipe"],
                },
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/data", "/srv/logs", "/var/lib/app", "/home/user/work"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: FILENAMES,
                },
            ],
        },
        AdversarialTemplate {
            family: "NONIDEM-RM-LOOP",
            target_class: 3,
            template: "for f in {A} {B} {C}; do\n  rm \"$f\"\ndone",
            params: &[
                ParamSlot {
                    name: "A",
                    pool: &["old.log", "cache.db", "temp.txt", "lock.pid", "run.sock"],
                },
                ParamSlot {
                    name: "B",
                    pool: &[
                        "state.dat",
                        "session.tmp",
                        "token.key",
                        "pid.lock",
                        "flag.set",
                    ],
                },
                ParamSlot {
                    name: "C",
                    pool: &["dump.sql", "trace.out", "core.err", "heap.prof", "gc.log"],
                },
            ],
        },
        // ln -s without -f (8)
        AdversarialTemplate {
            family: "NONIDEM-LN-SIMPLE",
            target_class: 3,
            template: "ln -s \"{TARGET}\" \"{LINK}\"",
            params: &[
                ParamSlot {
                    name: "TARGET",
                    pool: LINK_TARGETS,
                },
                ParamSlot {
                    name: "LINK",
                    pool: LINK_NAMES,
                },
            ],
        },
        AdversarialTemplate {
            family: "NONIDEM-LN-VERSION",
            target_class: 3,
            template: "ln -s \"/opt/{APP}/v{VER}\" \"/opt/{APP}/current\"",
            params: &[
                ParamSlot {
                    name: "APP",
                    pool: &["myapp", "service", "tool", "daemon", "agent"],
                },
                ParamSlot {
                    name: "VER",
                    pool: &["1.0", "2.0", "3.1", "4.0", "1.5"],
                },
            ],
        },
        AdversarialTemplate {
            family: "NONIDEM-LN-DOTFILE",
            target_class: 3,
            template: "ln -s \"{TARGET}\" \"/home/user/.{NAME}\"",
            params: &[
                ParamSlot {
                    name: "TARGET",
                    pool: &[
                        "/etc/app/config",
                        "/opt/tool/rc",
                        "/usr/share/defaults/conf",
                        "/var/lib/settings",
                        "/srv/config/main",
                    ],
                },
                ParamSlot {
                    name: "NAME",
                    pool: &["apprc", "toolrc", "config", "profile", "settings"],
                },
            ],
        },
        AdversarialTemplate {
            family: "NONIDEM-LN-BIN",
            target_class: 3,
            template: "ln -s \"/opt/{APP}/bin/{APP}\" \"/usr/local/bin/{APP}\"",
            params: &[ParamSlot {
                name: "APP",
                pool: &["myapp", "tool", "cli", "daemon", "agent"],
            }],
        },
        AdversarialTemplate {
            family: "NONIDEM-LN-RELATIVE",
            target_class: 3,
            template: "ln -s \"/opt/{SRC}\" \"{DIR}/{LINK}\"",
            params: &[
                ParamSlot {
                    name: "SRC",
                    pool: &[
                        "shared/lib",
                        "common/config",
                        "base/data",
                        "core/assets",
                        "vendor/deps",
                    ],
                },
                ParamSlot {
                    name: "DIR",
                    pool: &[
                        "/opt/app",
                        "/srv/deploy",
                        "/var/lib/svc",
                        "/home/user/proj",
                        "/opt/build",
                    ],
                },
                ParamSlot {
                    name: "LINK",
                    pool: &["lib", "config", "data", "assets", "deps"],
                },
            ],
        },
        AdversarialTemplate {
            family: "NONIDEM-LN-CONFIG",
            target_class: 3,
            template: "ln -s \"/etc/{APP}/{FILE}\" \"/opt/{APP}/config/{FILE}\"",
            params: &[
                ParamSlot {
                    name: "APP",
                    pool: &["myapp", "service", "tool", "daemon", "agent"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: &[
                        "app.conf",
                        "settings.yaml",
                        "config.toml",
                        "env.sh",
                        "params.json",
                    ],
                },
            ],
        },
        AdversarialTemplate {
            family: "NONIDEM-LN-MULTI",
            target_class: 3,
            template: "ln -s \"{TARGET1}\" \"{LINK1}\"\nln -s \"{TARGET2}\" \"{LINK2}\"",
            params: &[
                ParamSlot {
                    name: "TARGET1",
                    pool: LINK_TARGETS,
                },
                ParamSlot {
                    name: "LINK1",
                    pool: LINK_NAMES,
                },
                ParamSlot {
                    name: "TARGET2",
                    pool: &[
                        "/opt/lib/libapp.so",
                        "/usr/share/app/data",
                        "/var/lib/app/state",
                        "/etc/app/defaults",
                        "/srv/app/assets",
                    ],
                },
                ParamSlot {
                    name: "LINK2",
                    pool: &[
                        "/usr/lib/libapp.so",
                        "/opt/app/data",
                        "/var/app/state",
                        "/etc/defaults",
                        "/srv/assets",
                    ],
                },
            ],
        },
        AdversarialTemplate {
            family: "NONIDEM-LN-COMBO",
            target_class: 3,
            template: "mkdir \"{DIR}\"\nln -s \"{TARGET}\" \"{DIR}/link\"",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: SAFE_DIRNAMES,
                },
                ParamSlot {
                    name: "TARGET",
                    pool: LINK_TARGETS,
                },
            ],
        },
    ]
}

// ── Class 4: Unsafe templates (25) ─────────────────────────────────────

pub fn unsafe_templates() -> Vec<AdversarialTemplate> {
    vec![
        // SEC001 eval injection (3)
        AdversarialTemplate {
            family: "UNSAFE-EVAL-VAR",
            target_class: 4,
            template: "{VAR}=\"echo hello\"\neval \"${VAR}\"",
            params: &[ParamSlot { name: "VAR", pool: &["cmd", "command", "action", "instruction", "op"] }],
        },
        AdversarialTemplate {
            family: "UNSAFE-EVAL-ARGS",
            target_class: 4,
            template: "eval \"$@\"",
            params: &[],
        },
        AdversarialTemplate {
            family: "UNSAFE-EVAL-READ",
            target_class: 4,
            template: "read -r {VAR}\neval \"${VAR}\"",
            params: &[ParamSlot { name: "VAR", pool: &["input", "line", "cmd", "data", "expr"] }],
        },
        // SEC002 unquoted dangerous commands (4)
        AdversarialTemplate {
            family: "UNSAFE-CURL-UNQUOTED",
            target_class: 4,
            template: "{VAR}=\"{URL}\"\ncurl ${VAR}",
            params: &[
                ParamSlot { name: "VAR", pool: &["url", "endpoint", "target", "api_url", "download_url"] },
                ParamSlot { name: "URL", pool: URLS },
            ],
        },
        AdversarialTemplate {
            family: "UNSAFE-WGET-UNQUOTED",
            target_class: 4,
            template: "{VAR}=\"{URL}\"\nwget ${VAR}",
            params: &[
                ParamSlot { name: "VAR", pool: &["url", "src", "download", "pkg_url", "mirror"] },
                ParamSlot { name: "URL", pool: URLS },
            ],
        },
        AdversarialTemplate {
            family: "UNSAFE-SSH-UNQUOTED",
            target_class: 4,
            template: "{VAR}=\"{HOST}\"\nssh ${VAR} \"echo connected\"",
            params: &[
                ParamSlot { name: "VAR", pool: &["host", "server", "target", "remote", "node"] },
                ParamSlot { name: "HOST", pool: HOSTS },
            ],
        },
        AdversarialTemplate {
            family: "UNSAFE-GIT-UNQUOTED",
            target_class: 4,
            template: "{VAR}=\"https://github.com/example/repo.git\"\ngit clone ${VAR}",
            params: &[ParamSlot { name: "VAR", pool: &["repo", "repo_url", "source", "upstream", "origin"] }],
        },
        // SEC003 find -exec injection (2)
        AdversarialTemplate {
            family: "UNSAFE-FIND-EXEC",
            target_class: 4,
            template: "find \"{DIR}\" -name \"*.{EXT}\" -exec sh -c 'echo \"{}\"' \\;",
            params: &[
                ParamSlot { name: "DIR", pool: &["/tmp", "/var/log", "/opt/data", "/srv/files", "/home/user"] },
                ParamSlot { name: "EXT", pool: EXTENSIONS },
            ],
        },
        AdversarialTemplate {
            family: "UNSAFE-FIND-EXEC-RM",
            target_class: 4,
            template: "find \"{DIR}\" -name \"*.{EXT}\" -exec sh -c 'rm \"{}\"' \\;",
            params: &[
                ParamSlot { name: "DIR", pool: &["/tmp", "/var/cache", "/opt/data", "/srv/logs"] },
                ParamSlot { name: "EXT", pool: &["tmp", "bak", "old", "cache", "swp"] },
            ],
        },
        // SEC004 TLS bypass (2)
        AdversarialTemplate {
            family: "UNSAFE-WGET-NOCHECK",
            target_class: 4,
            template: "wget --no-check-certificate \"{URL}\"",
            params: &[ParamSlot { name: "URL", pool: URLS }],
        },
        AdversarialTemplate {
            family: "UNSAFE-CURL-INSECURE",
            target_class: 4,
            template: "curl -k \"{URL}\"",
            params: &[ParamSlot { name: "URL", pool: URLS }],
        },
        // SEC005 hardcoded secrets (3)
        AdversarialTemplate {
            family: "UNSAFE-SECRET-APIKEY",
            target_class: 4,
            template: "API_KEY=\"{SECRET}\"\ncurl -H \"Authorization: Bearer ${API_KEY}\" \"{URL}\"",
            params: &[
                ParamSlot { name: "SECRET", pool: SECRETS },
                ParamSlot { name: "URL", pool: URLS },
            ],
        },
        AdversarialTemplate {
            family: "UNSAFE-SECRET-PASSWORD",
            target_class: 4,
            template: "DB_PASSWORD=\"{SECRET}\"\nmysql -u root -p\"${DB_PASSWORD}\" \"{DB}\"",
            params: &[
                ParamSlot { name: "SECRET", pool: SECRETS },
                ParamSlot { name: "DB", pool: &["mydb", "production", "app_data", "users", "analytics"] },
            ],
        },
        AdversarialTemplate {
            family: "UNSAFE-SECRET-TOKEN",
            target_class: 4,
            template: "TOKEN=\"{SECRET}\"\necho \"Using token: ${TOKEN}\"",
            params: &[ParamSlot { name: "SECRET", pool: SECRETS }],
        },
        // SEC006 insecure tempfiles (2)
        AdversarialTemplate {
            family: "UNSAFE-TEMPFILE-PREDICT",
            target_class: 4,
            template: "echo \"{DATA}\" > /tmp/{FILE}\n{CMD} /tmp/{FILE}",
            params: &[
                ParamSlot { name: "DATA", pool: &["config data", "secret value", "temp output", "processing", "staging"] },
                ParamSlot { name: "FILE", pool: &["predictable.txt", "myapp.tmp", "data.cache", "output.log", "temp.dat"] },
                ParamSlot { name: "CMD", pool: &["cat", "wc -l", "head -1", "sort", "uniq"] },
            ],
        },
        AdversarialTemplate {
            family: "UNSAFE-TEMPFILE-RACE",
            target_class: 4,
            template: "TMPFILE=\"/tmp/{PREFIX}_output\"\necho \"{DATA}\" > \"${TMPFILE}\"\n{CMD} \"${TMPFILE}\"",
            params: &[
                ParamSlot { name: "PREFIX", pool: &["myapp", "build", "deploy", "stage", "proc"] },
                ParamSlot { name: "DATA", pool: &["result", "status", "output", "report", "digest"] },
                ParamSlot { name: "CMD", pool: &["cat", "wc -l", "head -1", "sort", "chmod 644"] },
            ],
        },
        // SEC001/SEC008 source untrusted (2)
        AdversarialTemplate {
            family: "UNSAFE-SOURCE-UNTRUSTED",
            target_class: 4,
            template: "eval \"$(cat /tmp/{FILE})\"",
            params: &[ParamSlot { name: "FILE", pool: &["env.sh", "config.sh", "setup.sh", "vars.sh", "init.sh"] }],
        },
        AdversarialTemplate {
            family: "UNSAFE-SOURCE-DOT",
            target_class: 4,
            template: "curl -sS \"{URL}\" | sh",
            params: &[ParamSlot { name: "URL", pool: URLS }],
        },
        // Additional unsafe patterns (7)
        AdversarialTemplate {
            family: "UNSAFE-CURL-PIPE-SH",
            target_class: 4,
            template: "curl -sSL \"{URL}\" | sh",
            params: &[ParamSlot { name: "URL", pool: URLS }],
        },
        AdversarialTemplate {
            family: "UNSAFE-CHMOD-777",
            target_class: 4,
            template: "chmod 777 \"{DIR}\"",
            params: &[ParamSlot { name: "DIR", pool: DIRNAMES }],
        },
        AdversarialTemplate {
            family: "UNSAFE-SUDO-RM-VAR",
            target_class: 4,
            template: "TARGET=\"{DIR}\"\nsudo rm -rf $TARGET",
            params: &[ParamSlot { name: "DIR", pool: &["/var/lib/app", "/opt/service", "/srv/data", "/etc/app", "/usr/local/share"] }],
        },
        AdversarialTemplate {
            family: "UNSAFE-SQL-INJECT",
            target_class: 4,
            template: "mysql -e \"SELECT * FROM {TABLE} WHERE id=$USER_INPUT\"",
            params: &[
                ParamSlot { name: "TABLE", pool: &["users", "orders", "sessions", "accounts", "logs"] },
            ],
        },
        AdversarialTemplate {
            family: "UNSAFE-XARGS-UNQUOTED",
            target_class: 4,
            template: "find \"{DIR}\" -name \"*.{EXT}\" -exec sh -c 'rm \"{}\"' \\;",
            params: &[
                ParamSlot { name: "DIR", pool: &["/opt/data", "/var/lib/cache", "/srv/data"] },
                ParamSlot { name: "EXT", pool: &["tmp", "bak", "old"] },
            ],
        },
        AdversarialTemplate {
            family: "UNSAFE-BACKTICK-EVAL",
            target_class: 4,
            template: "{VAR}=\"echo injected\"\neval \"${VAR}\"",
            params: &[ParamSlot { name: "VAR", pool: &["cmd", "op", "action", "expr", "run"] }],
        },
        AdversarialTemplate {
            family: "UNSAFE-EXPORT-SECRET",
            target_class: 4,
            template: "export {NAME}=\"{SECRET}\"",
            params: &[
                ParamSlot { name: "NAME", pool: &["API_KEY", "SECRET_KEY", "DB_PASSWORD", "AUTH_TOKEN", "PRIVATE_KEY"] },
                ParamSlot { name: "SECRET", pool: SECRETS },
            ],
        },
    ]
}

// ── Class 1: Needs-quoting templates (25) ──────────────────────────────

pub fn needs_quoting_templates() -> Vec<AdversarialTemplate> {
    vec![
        AdversarialTemplate {
            family: "QUOTE-ECHO-VAR",
            target_class: 1,
            template: "{VAR}=\"hello world\"\necho ${VAR}",
            params: &[ParamSlot {
                name: "VAR",
                pool: VARNAMES,
            }],
        },
        AdversarialTemplate {
            family: "QUOTE-CD-VAR",
            target_class: 1,
            template: "DEST=\"{DIR}\"\ncd $DEST",
            params: &[ParamSlot {
                name: "DIR",
                pool: SAFE_DIRNAMES,
            }],
        },
        AdversarialTemplate {
            family: "QUOTE-CP-SRC",
            target_class: 1,
            template: "SRC=\"{FILE}\"\ncp $SRC backup/",
            params: &[ParamSlot {
                name: "FILE",
                pool: FILENAMES,
            }],
        },
        AdversarialTemplate {
            family: "QUOTE-TEST-FILE",
            target_class: 1,
            template: "FILE=\"{DIR}/{FILE}\"\nif test -f $FILE; then\n  echo \"exists\"\nfi",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/data", "/var/lib", "/opt", "/srv", "/home/user"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: FILENAMES,
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-CAT-FILE",
            target_class: 1,
            template: "CONFIG=\"{DIR}/{FILE}\"\ncat $CONFIG",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &[
                        "/etc",
                        "/opt/app/config",
                        "/var/lib/svc",
                        "/srv/settings",
                        "/home/user/.config",
                    ],
                },
                ParamSlot {
                    name: "FILE",
                    pool: &[
                        "app.conf",
                        "settings.ini",
                        "config.yaml",
                        "env.sh",
                        "params.json",
                    ],
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-CHMOD-FILE",
            target_class: 1,
            template: "SCRIPT=\"{DIR}/{FILE}\"\nchmod +x $SCRIPT",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &[
                        "/usr/local/bin",
                        "/opt/app/bin",
                        "/home/user/bin",
                        "/srv/scripts",
                        "/var/lib/cron",
                    ],
                },
                ParamSlot {
                    name: "FILE",
                    pool: &["run.sh", "start.sh", "deploy.sh", "backup.sh", "cleanup.sh"],
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-MV-FILE",
            target_class: 1,
            template: "OLD=\"{DIR}/{FILE}\"\nNEW=\"{DIR}/{FILE}.bak\"\nmv $OLD \"${NEW}\"",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/work", "/var/lib/app", "/opt/data", "/srv/files"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: FILENAMES,
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-GREP-FILE",
            target_class: 1,
            template: "LOGFILE=\"{DIR}/{FILE}\"\ngrep \"{PATTERN}\" $LOGFILE",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/var/log", "/tmp", "/opt/logs", "/srv/logs"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: &[
                        "app.log",
                        "error.log",
                        "access.log",
                        "system.log",
                        "debug.log",
                    ],
                },
                ParamSlot {
                    name: "PATTERN",
                    pool: &["ERROR", "WARN", "FAIL", "timeout", "refused"],
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-WC-FILE",
            target_class: 1,
            template: "INPUT=\"{DIR}/{FILE}\"\nwc -l $INPUT",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/data", "/var/log", "/srv/data", "/srv/files"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: FILENAMES,
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-HEAD-FILE",
            target_class: 1,
            template: "DATA=\"{DIR}/{FILE}\"\nhead -n {N} $DATA",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/data", "/var/log", "/srv/data", "/srv/files"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: FILENAMES,
                },
                ParamSlot {
                    name: "N",
                    pool: &["1", "5", "10", "20", "50"],
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-TAIL-FILE",
            target_class: 1,
            template: "LOG=\"{DIR}/{FILE}\"\ntail -n {N} $LOG",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/var/log", "/tmp", "/opt/logs", "/srv/logs"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: &[
                        "app.log",
                        "error.log",
                        "access.log",
                        "system.log",
                        "debug.log",
                    ],
                },
                ParamSlot {
                    name: "N",
                    pool: &["10", "20", "50", "100", "200"],
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-SORT-FILE",
            target_class: 1,
            template: "INPUT=\"{DIR}/{FILE}\"\nsort $INPUT",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/work", "/var/data", "/opt/output", "/srv/files"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: &[
                        "data.csv",
                        "names.txt",
                        "scores.dat",
                        "results.tsv",
                        "entries.log",
                    ],
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-DIFF-FILES",
            target_class: 1,
            template: "FILE_A=\"{DIR}/{FILE1}\"\nFILE_B=\"{DIR}/{FILE2}\"\ndiff $FILE_A $FILE_B",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/work", "/opt/data", "/var/lib", "/srv/files"],
                },
                ParamSlot {
                    name: "FILE1",
                    pool: &["old.txt", "before.log", "expected.dat", "baseline.csv"],
                },
                ParamSlot {
                    name: "FILE2",
                    pool: &["new.txt", "after.log", "actual.dat", "current.csv"],
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-BASENAME",
            target_class: 1,
            template: "FILEPATH=\"{DIR}/{FILE}\"\nNAME=$(basename $FILEPATH)",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/work", "/opt/data", "/var/lib", "/srv/files"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: FILENAMES,
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-DIRNAME",
            target_class: 1,
            template: "FILEPATH=\"{DIR}/{FILE}\"\nPARENT=$(dirname $FILEPATH)",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &[
                        "/opt/data/sub",
                        "/opt/data/v1",
                        "/var/lib/app",
                        "/srv/files/latest",
                    ],
                },
                ParamSlot {
                    name: "FILE",
                    pool: FILENAMES,
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-LS-DIR",
            target_class: 1,
            template: "WORKDIR=\"{DIR}\"\nls $WORKDIR",
            params: &[ParamSlot {
                name: "DIR",
                pool: SAFE_DIRNAMES,
            }],
        },
        AdversarialTemplate {
            family: "QUOTE-READLINK",
            target_class: 1,
            template: "LINK=\"{LINK}\"\nreadlink $LINK",
            params: &[ParamSlot {
                name: "LINK",
                pool: LINK_NAMES,
            }],
        },
        AdversarialTemplate {
            family: "QUOTE-STAT-FILE",
            target_class: 1,
            template: "TARGET=\"{DIR}/{FILE}\"\nstat $TARGET",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/data", "/var/log", "/srv/data", "/srv/files"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: FILENAMES,
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-TOUCH-FILE",
            target_class: 1,
            template: "DEST=\"{DIR}/{FILE}\"\nmkdir -p \"{DIR}\"\ntouch $DEST",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/out", "/var/lib/app", "/opt/data/new", "/srv/staging"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: FILENAMES,
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-CHOWN-FILE",
            target_class: 1,
            template: "DEST=\"{DIR}/{FILE}\"\nchown \"{OWNER}\" $DEST",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/app", "/var/lib/svc", "/srv/data", "/home/user"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: FILENAMES,
                },
                ParamSlot {
                    name: "OWNER",
                    pool: &[
                        "root:root",
                        "www-data:www-data",
                        "nobody:nogroup",
                        "app:app",
                        "deploy:deploy",
                    ],
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-INSTALL-FILE",
            target_class: 1,
            template: "BIN=\"{DIR}/{APP}\"\ninstall -m 755 $BIN /usr/local/bin/",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/build", "/opt/stage", "/var/dist", "/srv/release"],
                },
                ParamSlot {
                    name: "APP",
                    pool: &["myapp", "tool", "cli", "daemon", "agent"],
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-PRINTF-VAR",
            target_class: 1,
            template: "{VAR}=\"{VAL}\"\nprintf \"%s\\n\" ${{VAR}}",
            params: &[
                ParamSlot {
                    name: "VAR",
                    pool: VARNAMES,
                },
                ParamSlot {
                    name: "VAL",
                    pool: &[
                        "hello world",
                        "test data",
                        "output value",
                        "some text",
                        "status ok",
                    ],
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-IF-VAR",
            target_class: 1,
            template: "{VAR}=\"{VAL}\"\nif [ ${{VAR}} = \"expected\" ]; then\n  echo \"match\"\nfi",
            params: &[
                ParamSlot {
                    name: "VAR",
                    pool: VARNAMES,
                },
                ParamSlot {
                    name: "VAL",
                    pool: &["expected", "test", "check", "verify", "confirm"],
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-WHILE-READ",
            target_class: 1,
            template:
                "FILE=\"{DIR}/{FILE}\"\nwhile read -r line; do\n  echo $line\ndone < \"${FILE}\"",
            params: &[
                ParamSlot {
                    name: "DIR",
                    pool: &["/opt/data", "/var/data", "/opt/input", "/srv/files"],
                },
                ParamSlot {
                    name: "FILE",
                    pool: FILENAMES,
                },
            ],
        },
        AdversarialTemplate {
            family: "QUOTE-EXPORT-VAR",
            target_class: 1,
            template: "{VAR}=\"{VAL}\"\nexport PATH=${{VAR}}:\"${PATH}\"",
            params: &[
                ParamSlot {
                    name: "VAR",
                    pool: &["BIN_DIR", "LIB_DIR", "APP_HOME", "TOOL_PATH", "EXTRA_PATH"],
                },
                ParamSlot {
                    name: "VAL",
                    pool: &[
                        "/opt/bin",
                        "/usr/local/lib",
                        "/home/user/bin",
                        "/srv/tools",
                        "/var/lib/app/bin",
                    ],
                },
            ],
        },
    ]
}

/// Return all 100 templates grouped by class.
pub fn all_templates() -> Vec<AdversarialTemplate> {
    let mut all = Vec::with_capacity(100);
    all.extend(needs_quoting_templates());
    all.extend(non_deterministic_templates());
    all.extend(non_idempotent_templates());
    all.extend(unsafe_templates());
    all
}
