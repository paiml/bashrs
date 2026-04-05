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


include!("adversarial_templates_unsafe_needs_quoting_templates.rs");
