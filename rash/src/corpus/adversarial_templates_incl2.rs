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


include!("adversarial_templates_incl2_incl2.rs");
