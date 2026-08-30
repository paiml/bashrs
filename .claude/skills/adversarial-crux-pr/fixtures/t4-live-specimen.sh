#!/usr/bin/env bash
# T4 LIVE SPECIMEN — bashrs 6.67.0 reports SC1078 on the line below and is
# WRONG: the string is closed. shellcheck and `bash -n` both accept it.
#
# This fixture must go CLEAN when T4 lands. If SC1078 reappears here, T4 has
# regressed. It is deliberately the only place this construct lives, so the
# gate itself can stay lint-clean while the specimen is still measurable.
set -euo pipefail
printf %s "not this PR%s problem" "'s"
