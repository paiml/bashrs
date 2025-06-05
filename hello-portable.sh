#!/bin/sh
# Self-extracting RASH script
set -euf

# Embedded compressed script
PAYLOAD='KLUv/QBgJREAZiJpJvB0qwOoJKJM/uVrePHyB/lmypoPuB2aZVDEBPrg/REDNpHxaKoCYABeAF0ABAPLr4Ysd1DU8aJyvo5knb7xtaxMWesB0mgcOh40b57VyK5nAfyBNCQya6rb3kvX6upxRA/hUvQt3YNVnFHjyGhcYJyyDCoHFYPNKeoyTuJxadvxyp04ZVHUIp+HJYASK1kYDrlG37RL0sU4I3bgMChWChItFX2BYRwGNZW+wC4OgyBXad/0eDcdkptQikxpnaw3Xd5vBCPAcFbgXEFa+Nz0lhszrqBx2HCQYJ9UtF9yqf4Ca3DYAz69/18lD/J9rMflcxWnrHVC0g++rp8JCJg7R9GomfxOrDQBAeMoKrn2siTrnHhKXqrfWiy9CGm9HyJwm0oO+2xnkKvDjqlcNhR2a39o1U1Bz2dPrlq7RoNO7SLAgDmEVVo2AYs1OOzyubjkc/HbNdqRiFPW2shwRPxhjlOhqGPLSRXb0hQyMhAYFRslDvvE7ZGfV457yfmqSrfwgfHSKkGeIhAvrWw9AXHy9AnhUjzQh3y9yx8ngXJKblk+FxcNLyAgAiIgDDM7DT4Fbe6iRQYNcxhAntIL28WRVRFuUkpauuy0KWET0xinxcZeVdqwdmkw/uDYIKFht47WBiKIBwBrcxmooAu4bC11cjxcD8ZKhdEcWxkoyzGhBRoSIzHQIhi2YWC6n0gg6X15ypLNaPHVaJvBcyjkDGo1nDM='

# Extract and decompress
if command -v base64 >/dev/null && command -v zstd >/dev/null; then
    echo "$PAYLOAD" | base64 -d | zstd -d | sh -euf
elif command -v base64 >/dev/null && command -v gzip >/dev/null; then
    # Fallback to gzip if zstd not available
    echo "Warning: zstd not found, using gzip (larger size)" >&2
    GZIP_PAYLOAD='H4sIAAAAAAAC/6VTTW/aQBA9d3/F4FhxW8kQiNoDlVERTVok0kTAITdk7DFe1d519oMPpfz3jm0gUJG2Un3aHc+89+Zp30WjNeeipVN2AV9RoAoNxjDfwDjUKSyvmtfNK/r1cD8ZPvqRzIuMh8KATjHLQEeKF4YxjQZ8tAkb3k4CD94wj+G6kMrAaDDrj0bBgBFGhaisMDxHSKyIDJdCM0XlmcInyxW+fQfPDOjjCTTAcT87n8CkKKpa+WGUSnBu+9P+qAvjeihHEpSEPMO4C+57B3qXnZeBNTfQrq4JZ1tW08VyJTIZxrMlKp5wjA/EVmWB47aJN9aGTh06RSlGP7TN6XrtVF17jWRIHooY/CVENAm9VozLlrDkTad32f5NfNXiJ3oyAt8vlDQSvCA1ptAeFUyml+1mh7amNgd82tMlDTUhZqdsqwVZ/me2qsV/ut/B7HB3aBrPW/odOYGoWqqQqkZJpBXxqa8KjVXixdlXXNFp2Pnwkbz7i9hahLt3GqDW/PMIwI+OMM6aQr3/x1TRhECEZ9leNU0enghYwzNuNv9i2bYMxV3IxS5HMMcFp0DkVDq8R4VhLEW2gYVCNFwsAu8bRU9ComReRarhsaO99m3O6bTVqIKVVFl83FxWnVrHIMNQ2AKkqCLDjAoL8BT5oRLqfJ7ePXwZjrt+y+TFtlWGqOm6jgc3j8NpOX+zxsgahFL8IdzVKlWM2S8HxSrtaAQAAA=='
    echo "$GZIP_PAYLOAD" | base64 -d | gzip -d | sh -euf
else
    echo "Error: base64 and compression tools required" >&2
    exit 1
fi
