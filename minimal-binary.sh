#!/bin/sh
# Self-extracting RASH script
set -euf

# Embedded compressed script
PAYLOAD='KLUv/QBgjRAAZmFmJdDWVgfYLFbZzjk1Mkg/wf87zxBOCREGccWpJV/GyC/wxYNUPEFeAFsAWgDwjnD9kKyryk4KG5o3OkvY89aKUg+PRuPw8aB5VzqVsHIUwJ1nQ6JCRZfbm61SPcEK2cfIkn1PB+MkKdQ8KjYwMl5RGMuBxWBzyTqNm4D8cbvKkydeUZK9TAhyCagE0DftbzopKbQO2HrktSIQS9AXVoatp+37wsKw9cR6H8MIVt58PEfjJKFPpSgYYd6wJCPA8FXArvA4Qhe9J0uNK2gcOBwk1qkE8dVkqb+wGth6wCP4/+rkBANcA0qlmDSMsO9pAkLmzkkWaig/k9cmIGScZGe9zfXNS6NKZ2WpIV+wxRhpzSEkkds+tk7bV6yXrWcsmI2N3dodO3VLj3PatE6tXbNho4YRYMg8xjsvG34Xhsd6A2o1sPUJnVQTOnnuC3FJxCtKcVR4RNxZjFkk6+olZZJ9aQoaFRAZFhwnbD2q7bI6n1xtNemnOx3Hh6WcFyvtOI2RKZyHQ5y1LG8SJ32O64ROKjthCIN8QmfxSiwgIAIiIAqzOw07BQ1+cfs4Ug5X3bhJKWnpstMGhE1MYxyLjXtVt2EtpMHwwZFBYsNuHq0NRBAPAKbmMqigy3DZInVCPFwMxmqF0RxbGSjLMaoFGhIvMdBBMLVhoLufSiDp+/LKkuFo8dWoNoPVUMgZ1GowZw=='

# Extract and decompress
if command -v base64 >/dev/null && command -v zstd >/dev/null; then
    echo "$PAYLOAD" | base64 -d | zstd -d | sh -euf
elif command -v base64 >/dev/null && command -v gzip >/dev/null; then
    # Fallback to gzip if zstd not available
    echo "Warning: zstd not found, using gzip (larger size)" >&2
    GZIP_PAYLOAD='H4sIAAAAAAAC/6VTXW/aQBB87v2KjWPFbSVDIWofqEBFNGmRSBMBUvOGDnuNTz3fOfdBQCn/vYtNCKlIW6l+ulvvzsyObk5PmnOhmjZnp/AFFRruMIX5Gsbc5rB81zhvtOjXzfVkeBsnuiil4MqBzVFKsIkRpWPMooMYfcaGl5NuBK9YxHBVauNgNJj1R6PugBFGhWi8cqJAyLxKnNDKMkPlmcE7Lwy+fgMPDOgTGZxAEH4KPoLLUVW17YdJriG47E/7ow6M66ECSVDGhcS0A+HbAHpn7aeBlXDQqq6ZYBtW06X6XknN09kSjcgEpntib2Q3CFvEm1pHpzadkhyTH9YXdD0Pqq5HjWRIwVUK8RISmoReM8VlU3nypt07a/0mvmqJMzsZQRyXRjsNUTd3rrQRFZy0y1ajTVtTWwAx7RmShpoQ5XO2+wVZ/me2qiW+u97B7HB3aBaPW/oNBYGYWqrSpkbJtFfpc18NOm/Uk7MvuGJz3n7/gbz7i9haRPjoNECt+ecBQJwcYBw1hXr/j6mi4UCER9leNE3vnwh4J6Rw63+xbLMNxRUXapcjmONCUCAKKu3fo0GeaiXXUKC1fIHd6CslT8N3bWQasYONdg1BjTuQyJUvQasqAswZXkJkaD+TUfPD9Orm83DciZuuKDfNbSgaYRhEcHE7nG7nL1aYeIewFbMPayWtiiX7BbtQ+sM4BAAA'
    echo "$GZIP_PAYLOAD" | base64 -d | gzip -d | sh -euf
else
    echo "Error: base64 and compression tools required" >&2
    exit 1
fi
