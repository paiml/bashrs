#!/bin/bash
# Simplified small bash script for purification benchmarks
# Focuses on purification opportunities without unsupported syntax
# Target: ~50 lines

# Non-deterministic: $RANDOM
ID=$RANDOM
echo "Random ID: $ID"

# Non-deterministic: $$
PID=$$
TEMP_DIR="/tmp/build-$PID"

# Non-idempotent: mkdir without -p
mkdir logs
mkdir data
mkdir cache

# Unquoted variables (safety issue)
SOURCE="/tmp/input"
DEST="/tmp/output"
cp $SOURCE $DEST

# Non-idempotent: rm without -f
rm temp.txt
rm data.tmp

# Non-deterministic: generate random values
function gen_id() {
    echo $RANDOM
}

function gen_temp() {
    echo "/tmp/file-$$"
}

# Non-idempotent: ln without -sf
ln config.txt current.txt

# Unquoted command substitution
FILES=$(ls /tmp)
echo $FILES

# Non-idempotent: touch
touch app.log
touch debug.log

# More non-deterministic values
SESSION=$RANDOM
REQUEST=$$

echo "Done"
