#!/usr/bin/env bash
# The core module greets the name it is given, and only that name. Prints one TAP line per case.
. src/core.sh
fail=0
if [ "$(core_greet ada)" = "hello, ada" ]; then echo "ok - core greets ada"; else echo "not ok - core greets ada (got '$(core_greet ada)')"; fail=1; fi
if [ "$(core_greet grace)" = "hello, grace" ]; then echo "ok - core greets grace"; else echo "not ok - core greets grace (got '$(core_greet grace)')"; fail=1; fi
exit "$fail"
