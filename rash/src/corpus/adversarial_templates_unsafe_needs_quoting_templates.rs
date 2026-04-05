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
