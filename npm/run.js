#!/usr/bin/env node
"use strict";

// The `plotplot` command npm puts on PATH: the binary install.js placed, with the same
// arguments, stdio and exit status.

const path = require("node:path");
const { spawnSync } = require("node:child_process");
const { binaryName } = require("./install.js");

const binary = path.join(__dirname, "bin", binaryName(process.platform));
const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });

if (result.error) {
  console.error(`plotplot: ${binary} could not be run: ${result.error.message}`);
  process.exit(1);
}
if (result.signal) {
  process.kill(process.pid, result.signal);
}
process.exit(result.status ?? 1);
