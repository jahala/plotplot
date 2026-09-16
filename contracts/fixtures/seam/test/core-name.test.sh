#!/usr/bin/env bash
# The core module names itself. Prints one TAP line, so a verifier can count what ran.
. src/core.sh
if [ "$(core_name)" = "core" ]; then echo "ok - the core module names itself"; else echo "not ok - the core module names itself (got '$(core_name)')"; exit 1; fi
