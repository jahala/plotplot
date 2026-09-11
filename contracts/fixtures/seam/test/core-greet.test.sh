#!/usr/bin/env bash
# The core module greets the name it is given, and only that name.
. src/core.sh
[ "$(core_greet ada)" = "hello, ada" ] && [ "$(core_greet grace)" = "hello, grace" ]
