#!/bin/bash
# comply:disable=COMPLY-001
# Generate bash test scripts of specified line counts for performance testing

set -euo pipefail

LINES=${1:-1000}
OUTPUT=${2:-/tmp/test_script.sh}

cat > "$OUTPUT" << 'SCRIPT_START'
#!/bin/bash
# Generated test script for performance benchmarking
# Contains a mix of deterministic and idempotent code

SCRIPT_START

# Generate script content
for ((i=1; i<=LINES; i++)); do
    case $((i % 10)) in
        0) echo "echo \"Line $i: Hello world\"" >> "$OUTPUT" ;;
        1) echo "if [ -f /tmp/test ]; then echo \"found\"; fi" >> "$OUTPUT" ;;
        2) echo "for x in 1 2 3; do echo \$x; done" >> "$OUTPUT" ;;
        3) echo "mkdir -p /tmp/dir$i" >> "$OUTPUT" ;;
        4) echo "rm -f /tmp/file$i" >> "$OUTPUT" ;;
        5) echo "VAR$i=\"value$i\"" >> "$OUTPUT" ;;
        6) echo "# Comment line $i" >> "$OUTPUT" ;;
        7) echo "echo \$HOME" >> "$OUTPUT" ;;
        8) echo "cd /tmp && ls" >> "$OUTPUT" ;;
        9) echo "cat /dev/null > /tmp/out$i" >> "$OUTPUT" ;;
    esac
done

echo "Generated $LINES-line script at: $OUTPUT"
wc -l "$OUTPUT"
