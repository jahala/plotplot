#!/usr/bin/env node
"use strict";

// The release workflow's step from the release's own SHA256SUMS to the checksums.json the
// package carries:
//
//   node npm/checksums.js <SHA256SUMS> > npm/checksums.json
//
// Not shipped in the package: install.js reads the JSON this writes, never this script.

const fs = require("node:fs");

/** `sha256sum`'s output as { asset: digest }, refusing anything else. */
function fromSha256sums(text) {
  const digests = {};
  const lines = text.split("\n");
  lines.forEach((line, index) => {
    if (line.trim() === "") return;
    const match = /^([0-9a-f]{64}) [ *](\S+)$/.exec(line);
    if (match === null) {
      throw new Error(`line ${index + 1} is not a sha256 digest and a file name: ${line}`);
    }
    const [, digest, name] = match;
    if (name in digests) {
      throw new Error(`${name} is named twice`);
    }
    digests[name] = digest;
  });
  if (Object.keys(digests).length === 0) {
    throw new Error("no digests");
  }
  return digests;
}

if (require.main === module) {
  const [file] = process.argv.slice(2);
  if (!file) {
    console.error("usage: node checksums.js <SHA256SUMS>");
    process.exit(2);
  }
  try {
    process.stdout.write(`${JSON.stringify(fromSha256sums(fs.readFileSync(file, "utf8")), null, 2)}\n`);
  } catch (error) {
    console.error(`checksums.js: ${file}: ${error.message}`);
    process.exit(1);
  }
}

module.exports = { fromSha256sums };
