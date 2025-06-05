#!/bin/sh
# Self-extracting RASH script
set -euf

# Embedded compressed script
PAYLOAD='KLUv/QBg5REAViRuJvCSVge0IXSntJRQ19DLL193yQxZEIHnqd0q4FEO7IVc4BlIRVMVZABlAGEA8P9XyAkGmMLrwRRD1UdX7Qlt1Hc6S2j7UslaD9Dl4oB5uNzSWW14OQvgDpQhYVnTZXfLdS1bQRrVx8hUve0HYyVn5DYsGRYXlyzK5GBioDlVH8ZBOC6xfZSbJy5ZVH0acjwBkpeMOjmKAEPlMd5+NCw9DI/0Rte4S/2knBE7aAySt3ZETEVfYJXGIFulLzBKY1CkV2I4QcoajPRm7I8prZEFJ8ot3FQEGM4K2hUkjvBNt80S4woXBw0HCfZJRfzVTNkvsIbGHvAp94E2cGJaHST2lB7Thk+u2jDDHG/4KC5ZawTEwgjt/UxAqPw9qkZO5G/y1gSEyqNq02uZSn3nRGdTpizkCq4WHs5y5iOyrVJjn+4s0tvYKxOKpkbr3J1aWUtBzqedtHLuAYs1NPaGT66GT57tjXgTcclaGpaNiDustQlVH31GKmkfp4Bhgbig0Ahp7BN1l9R59ahdPV/dfhwfWGOnUOqQrLHzjU/jZBrG+Gxti3POvFpZYpbBxGBCgXrCSlkTXzTjQ2sxICACIiDMMvN9QAp23UVZ4/JrGFzWzjG1BTfCmEgoReSYJJouJZNtnkQTaILzAmJe1W1YW2kw+uBokKhht47WViKIBwBTcxmpoKtw2ULqRDxcGoxVCqM5tjJQkGNKCzQkRmKgh2Bqw0B1P5VA0vvylSW/0eKr0TaD51DIGdRqWGc='

# Extract and decompress
if command -v base64 >/dev/null && command -v zstd >/dev/null; then
    echo "$PAYLOAD" | base64 -d | zstd -d | sh -euf
elif command -v base64 >/dev/null && command -v gzip >/dev/null; then
    # Fallback to gzip if zstd not available
    echo "Warning: zstd not found, using gzip (larger size)" >&2
    GZIP_PAYLOAD='H4sIAAAAAAAC/6VUTU/bQBA9d3/FYCzcVnJMgtoDVVARhSpSKCjJgVu0scfxivWu2Y+QiOa/d2Lng1ShrVSfdmdn3nvzNOPjo2QiVGILdgzfUaHhDjOYLGDAbQGz09ZZq01P93fD3kOc6rKSgisHtkApwaZGVI4xiw5i9Dnr3Qy7EbxjEcN5pY2D/tX4st/vXjHCqBGNV06UCLlXqRNaWWYoPDb45IXB9x/ghQF9IocjCMKvwRdwBao6tvowLTQEN5ejy/45DJqiEklQzoXE7BzCjwFcnHR2BXPhoF1fc8GWrKHL9LOSmmfjGRqRC8y2xN7IbhC2iTezjk4dOqUFpo/Wl3Q9C+qsjUYypOQqg3gGKVXCRZLhLFGevOlcnLR/E1+nxLkd9iGOK6OdhqhbOFfZiAJO2lm71aGuKS2AmPoMSUNDiHKf7XlKlv+ZrU6Jn+7WMGvcNZrFw5b+QEEgppGqtGlQcu1Vtu+rQeeN2jn7hiu24J1Pn8m7v4htRIQbpwEazT9fAcTpK4yDplDu/zHVNByI8CDbm6bp7YiAd0IKt/gXy5arpbjlQq33CCY4FbQQJYW282iQZ1rJBVQGczHvJt6aROqUy/1nGmRL69Rtt05bp2wjcVyitXyKEPWUdVxKoaab1GiXNePSI5myfmmGpHzMhKFgQxw0cq8kcuUr0KreLOYMryAyZJvJKfdldHv/rTc4jxNXVstktWutMAwiuH7ojVb113NMvUNY9bj9B9Qd19vOfgEOpSaFjwQAAA=='
    echo "$GZIP_PAYLOAD" | base64 -d | gzip -d | sh -euf
else
    echo "Error: base64 and compression tools required" >&2
    exit 1
fi
