"use strict";

// The release workflow turns the release's own SHA256SUMS into the checksums.json the
// package carries, so the digests install.js trusts are the published ones.

const { test } = require("node:test");
const assert = require("node:assert/strict");

const { fromSha256sums } = require("../checksums.js");

const A = "a".repeat(64);
const B = "b".repeat(64);

test("each line of sha256sum's output becomes one asset and its digest", () => {
  assert.deepEqual(
    fromSha256sums(`${A}  plotplot-aarch64-apple-darwin.tar.gz\n${B}  plotplot-x86_64-pc-windows-msvc.tar.gz\n`),
    {
      "plotplot-aarch64-apple-darwin.tar.gz": A,
      "plotplot-x86_64-pc-windows-msvc.tar.gz": B,
    },
  );
});

test("binary mode's asterisk is not part of the name", () => {
  assert.deepEqual(fromSha256sums(`${A} *plotplot-x86_64-apple-darwin.tar.gz\n`), {
    "plotplot-x86_64-apple-darwin.tar.gz": A,
  });
});

test("a line that is not a digest and a name is refused, naming the line", () => {
  assert.throws(() => fromSha256sums(`${A}  ok.tar.gz\nnot a digest line\n`), /line 2/);
  assert.throws(() => fromSha256sums(`${"A".repeat(64)}  upper.tar.gz\n`), /line 1/);
});

test("an empty file, or one naming an asset twice, is refused", () => {
  assert.throws(() => fromSha256sums("\n"), /no digests/);
  assert.throws(() => fromSha256sums(`${A}  same.tar.gz\n${B}  same.tar.gz\n`), /twice/);
});
