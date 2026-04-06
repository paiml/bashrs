#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::bash_parser::parser_arith::ArithToken;
    #[test]
    fn test_V_TEST_001_variable_set() {
        let input = "if [[ -v MYVAR ]]; then\n    echo set\nfi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "-v test operator should parse: {:?}",
            ast.err()
        );
    }

    #[test]
    fn test_ENV_PREFIX_001_while_ifs() {
        // IFS='=' before read — env prefix, not assignment condition
        let input =
            "while IFS='=' read -r key value; do\n    echo \"$key=$value\"\ndone < input.txt";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "IFS= env prefix in while should parse: {:?}",
            ast.err()
        );
    }

    #[test]
    fn test_REGEX_POSIX_CLASS_001_bracket_depth() {
        // =~ with POSIX char class [[:space:]] should not break on ]] inside
        let input = "if [[ \"$key\" =~ ^[[:space:]]*# ]]; then\n    echo comment\nfi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "=~ with [[:space:]] should parse: {:?}",
            ast.err()
        );
    }

    #[test]
    fn test_COMBINED_REDIR_001_if_condition() {
        // &>/dev/null in if command condition
        let input = "if command -v git &>/dev/null; then\n    echo found\nfi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "&>/dev/null in if condition should parse: {:?}",
            ast.err()
        );
    }

    #[test]
    fn test_COMBINED_REDIR_002_negated_condition() {
        // ! command -v ... &>/dev/null
        let input = "if ! command -v git &>/dev/null; then\n    echo missing\nfi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "negated &>/dev/null in condition should parse: {:?}",
            ast.err()
        );
    }

    #[test]
    fn test_COMBINED_REDIR_003_in_command() {
        // &> in regular command (already tested but verify no regression)
        let input = "echo hello &> output.log";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(ast.is_ok(), "&> in command should parse: {:?}", ast.err());
        if let BashStmt::Command { redirects, .. } = &ast.expect("ok").statements[0] {
            assert_eq!(redirects.len(), 1, "Should have one Combined redirect");
            assert!(matches!(&redirects[0], Redirect::Combined { .. }));
        }
    }

    #[test]
    fn test_DOGFOOD_022_assoc_arrays_and_arithmetic() {
        // Full dogfood_22 constructs
        let input = r#"declare -A config
config[host]="localhost"
config[port]="8080"
for key in "${!config[@]}"; do
    printf "%s = %s\n" "$key" "${config[$key]}"
done
arr=(zero one two three four five)
echo "Elements 2-4: ${arr[@]:2:3}"
echo "Last element: ${arr[-1]}"
a=10; b=3
echo "Add: $((a + b))"
echo "Mul: $((a * b))"
max=$((a > b ? a : b))
echo "Max: $max"
"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "dogfood_22 constructs should parse: {:?}",
            ast.err()
        );
    }

    #[test]
    fn test_DOGFOOD_023_deployment_script() {
        // Key constructs from dogfood_23
        let input = r#"set -euo pipefail
readonly LOG_FILE="/var/log/deploy.log"
readonly TIMESTAMP_FMT="+%Y-%m-%d %H:%M:%S"

log() {
    local level="$1"
    shift
    local msg="$*"
    echo "[$level] $msg" >&2
}

info()  { log "INFO"  "$@"; }

health_check() {
    local url="$1"
    local max_retries="${2:-10}"
    local attempt=0
    while (( attempt < max_retries )); do
        if curl -sf -o /dev/null "$url" 2>/dev/null; then
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 5
    done
    return 1
}

deploy_service() {
    local service_name="$1"
    for cmd in docker curl jq; do
        if ! command -v "$cmd" &>/dev/null; then
            return 1
        fi
    done
    if ! docker pull "$service_name" 2>/dev/null; then
        return 1
    fi
}

main() {
    info "Starting deployment"
    deploy_service "${SERVICE_NAME:-myapp}"
    health_check "${HEALTH_URL:-http://localhost:8080/health}"
}

main "$@"
"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "dogfood_23 key constructs should parse: {:?}",
            ast.err()
        );
    }

    // --- Batch 3: $'...' ANSI-C quoting, heredoc on done, -L test op ---

    #[test]
    fn test_ANSI_C_QUOTE_001_tab() {
        let input = "IFS=$'\\t' read -r a b";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "$'\\t' ANSI-C quoting should parse: {:?}",
            ast.err()
        );
    }

    #[test]
    fn test_ANSI_C_QUOTE_002_newline() {
        let input = "echo $'hello\\nworld'";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "$'\\n' ANSI-C quoting should parse: {:?}",
            ast.err()
        );
    }

    #[test]
    fn test_HEREDOC_COMPOUND_001_done_heredoc() {
        let input = "while read -r line; do\n    echo \"$line\"\ndone <<EOF\napple\nbanana\nEOF";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(ast.is_ok(), "done <<EOF should parse: {:?}", ast.err());
    }

    #[test]
    fn test_FILE_TEST_001_symlink() {
        let input = "if [ -L /tmp/link ]; then echo symlink; fi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(ast.is_ok(), "-L test should parse: {:?}", ast.err());
    }

    #[test]
    fn test_FILE_TEST_002_all_operators() {
        // Test all file test operators
        for op in [
            "-f", "-e", "-s", "-d", "-r", "-w", "-x", "-L", "-h", "-p", "-b", "-c", "-g", "-k",
            "-u", "-t", "-O", "-G", "-N", "-v", "-n", "-z",
        ] {
            let input = format!("[ {} /tmp/test ]", op);
            let mut parser = BashParser::new(&input).expect("parser");
            let ast = parser.parse();
            assert!(ast.is_ok(), "{} test should parse: {:?}", op, ast.err());
        }
    }

    #[test]
    fn test_TRIPLE_ELIF_001_with_else() {
        let input = "if [ -f x ]; then\n    echo a\nelif [ -d x ]; then\n    echo b\nelif [ -L x ]; then\n    echo c\nelse\n    echo d\nfi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(ast.is_ok(), "triple elif should parse: {:?}", ast.err());
        if let BashStmt::If {
            elif_blocks,
            else_block,
            ..
        } = &ast.expect("ok").statements[0]
        {
            assert_eq!(elif_blocks.len(), 2, "Should have 2 elif blocks");
            assert!(else_block.is_some(), "Should have else block");
        }
    }

    #[test]
    fn test_DOGFOOD_024_traps_and_ansi_c() {
        let input = r#"set -euo pipefail
TMPDIR=$(mktemp -d)
cleanup() {
    local exit_code=$?
    rm -rf "$TMPDIR"
    exit "$exit_code"
}
trap cleanup EXIT
trap 'echo "Caught SIGINT" >&2; cleanup' INT
exec 200>"$LOCKFILE"
flock -n 200 || { echo "Already running" >&2; exit 1; }
"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "dogfood_24 traps should parse: {:?}",
            ast.err()
        );
    }

    #[test]
    fn test_DOGFOOD_026_git_and_find() {
        let input = r#"current_branch=$(git branch --show-current)
default_branch=$(git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null | sed 's|origin/||' || echo "main")
if [[ "$current_branch" != "$default_branch" ]]; then
    echo "Not on $default_branch branch"
fi
find /var/log -type f -name "*.log" -exec gzip {} \;
find . -name "*.txt" -print0 | xargs -0 grep -l "pattern" 2>/dev/null || true
"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "dogfood_26 git/find should parse: {:?}",
            ast.err()
        );
    }

    #[test]
    fn test_DOGFOOD_027_detect_os_and_install() {
        let input = r#"detect_os() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        echo "$ID"
    elif [[ -f /etc/redhat-release ]]; then
        echo "rhel"
    elif command -v sw_vers &>/dev/null; then
        echo "macos"
    else
        echo "unknown"
    fi
}

install_package() {
    local pkg="$1"
    case "$(detect_os)" in
        ubuntu|debian)
            sudo apt-get install -y "$pkg"
            ;;
        centos|rhel|fedora)
            sudo yum install -y "$pkg"
            ;;
        *)
            echo "Unknown OS" >&2
            return 1
            ;;
    esac
}
"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "dogfood_27 detect_os should parse: {:?}",
            ast.err()
        );
    }

    // --- Batch 4: && || inside [[ ]], -a -o inside [ ] ---

    #[test]
    fn test_TEST_AND_001_double_bracket() {
        let input = r#"if [[ "$a" == "1" && "$b" == "2" ]]; then echo ok; fi"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(ast.is_ok(), "&& inside [[ ]] should parse: {:?}", ast.err());
    }

    #[test]
    fn test_TEST_OR_001_double_bracket() {
        let input = r#"if [[ "$a" == "1" || "$b" == "2" ]]; then echo ok; fi"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(ast.is_ok(), "|| inside [[ ]] should parse: {:?}", ast.err());
    }

    #[test]
    fn test_TEST_AND_002_single_bracket() {
        let input = "if [ -f /etc/passwd -a -r /etc/passwd ]; then echo ok; fi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(ast.is_ok(), "-a inside [ ] should parse: {:?}", ast.err());
    }

    #[test]
    fn test_TEST_OR_002_single_bracket() {
        let input = "if [ -f /tmp/a -o -f /tmp/b ]; then echo ok; fi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(ast.is_ok(), "-o inside [ ] should parse: {:?}", ast.err());
    }

    #[test]
    fn test_TEST_COMPOUND_001_triple_and() {
        let input = r#"[[ "$a" == "1" && "$b" == "2" && "$c" == "3" ]]"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "triple && inside [[ ]] should parse: {:?}",
            ast.err()
        );
    }

    #[test]
    fn test_DOGFOOD_029_edge_cases() {
        let input = r#"result=$(echo "$(basename "$(dirname "$(pwd)")")")
echo "Grandparent: $result"
echo "${UNDEFINED:-default value with spaces}"
outer="hello"
echo "${outer:-${inner:-deep_default}}"
x=10
(( x += 5 ))
echo "x=$x"
for i in 1 2 3; do
    for j in a b c; do
        if [[ "$j" == "b" ]]; then
            continue
        fi
        if [[ "$i" == "2" && "$j" == "c" ]]; then
            break 2
        fi
        echo "$i-$j"
    done
done
n=5
until [[ $n -le 0 ]]; do
    echo "Countdown: $n"
    n=$((n - 1))
done
if (( age >= 18 && age < 65 )); then
    echo "Working age"
fi
if [ -f /etc/passwd -a -r /etc/passwd ]; then
    echo "readable"
fi
"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "dogfood_29 edge cases should parse: {:?}",
            ast.err()
        );
    }
}
