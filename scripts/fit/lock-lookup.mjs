#!/usr/bin/env node
// scripts/fit/lock-lookup.mjs: the only TOML-aware part of the fit runner.
// Reads a garden.lock (TOML) and prints the resolved judge as one JSON line:
//   {"version":"...","npm":"..."?,"url":"..."?,"sha256":"..."?}
// Usage: node lock-lookup.mjs <lockfile> <judge> [platform]
// Exit 0 with the JSON line on stdout when the judge (and, if a platform was given, that
// platform's artifact) is found; exit 1 with a message on stderr otherwise. Nothing here
// fetches or verifies anything: that stays in lib.sh, which is the part every test can
// see and reason about without reading Node.

import { readFileSync } from "node:fs";
import { parse } from "smol-toml";

const [, , lockPath, judgeName, platform] = process.argv;

if (!lockPath || !judgeName) {
  console.error("usage: lock-lookup.mjs <lockfile> <judge> [platform]");
  process.exit(1);
}

let doc;
try {
  doc = parse(readFileSync(lockPath, "utf8"));
} catch (err) {
  console.error(`lock-lookup: cannot read or parse ${lockPath}: ${err.message}`);
  process.exit(1);
}

const judge = doc.judges && doc.judges[judgeName];
if (!judge) {
  console.error(`lock-lookup: no judge named "${judgeName}" in ${lockPath}`);
  process.exit(1);
}

const out = { version: judge.version, season: doc.season };
if (judge.npm) out.npm = judge.npm;

if (platform) {
  const entry = judge.platforms && judge.platforms[platform];
  if (!entry) {
    console.error(`lock-lookup: judge "${judgeName}" has no platform "${platform}" in ${lockPath}`);
    process.exit(1);
  }
  out.url = entry.url;
  out.sha256 = entry.sha256;
}

process.stdout.write(JSON.stringify(out) + "\n");
