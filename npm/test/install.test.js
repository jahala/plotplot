"use strict";

// The installer, driven with bytes a test supplies in place of the network: the same seam
// the stem's own `lock verify` tests use. The archive is a real .tar.gz built here, so what
// is under test is the deciding and the placing, never a recording of them.

const { test } = require("node:test");
const assert = require("node:assert/strict");
const crypto = require("node:crypto");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const installer = require("../install.js");

const VERSION = "0.1.0";
const UNIX = process.platform !== "win32";

function scratch(t) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "plotplot-npm-test-"));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  return dir;
}

// A release asset as the release workflow packs it: the executable alone, at the root.
function asset(t) {
  const dir = scratch(t);
  const binary = path.join(dir, "plotplot");
  fs.writeFileSync(binary, "#!/bin/sh\necho \"plotplot " + VERSION + "\"\n", { mode: 0o755 });
  const archive = path.join(dir, "asset.tar.gz");
  execFileSync("tar", ["-czf", archive, "-C", dir, "plotplot"]);
  return fs.readFileSync(archive);
}

function sha256(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

// A fetcher serving one url, recording what was asked for.
function serving(url, bytes) {
  const asked = [];
  const fetch = async (wanted) => {
    asked.push(wanted);
    if (wanted !== url) throw new Error(`nothing is served at ${wanted}`);
    return bytes;
  };
  return { fetch, asked };
}

test("every platform the release builds maps to its target, and no other does", () => {
  assert.equal(installer.target("darwin", "arm64"), "aarch64-apple-darwin");
  assert.equal(installer.target("darwin", "x64"), "x86_64-apple-darwin");
  assert.equal(installer.target("linux", "x64"), "x86_64-unknown-linux-musl");
  assert.equal(installer.target("linux", "arm64"), "aarch64-unknown-linux-musl");
  assert.equal(installer.target("win32", "x64"), "x86_64-pc-windows-msvc");
  assert.equal(installer.target("linux", "ia32"), null);
});

test("the asset is the release's own, at the package's version", () => {
  assert.equal(
    installer.assetUrl(VERSION, "aarch64-apple-darwin"),
    "https://github.com/jahala/plotplot/releases/download/v0.1.0/plotplot-aarch64-apple-darwin.tar.gz",
  );
});

test("bytes whose digest is the one the package carries are placed and run", { skip: !UNIX }, async (t) => {
  const bytes = asset(t);
  const target = installer.target("linux", "x64");
  const url = installer.assetUrl(VERSION, target);
  const { fetch, asked } = serving(url, bytes);
  const dir = scratch(t);

  const placed = await installer.install({
    platform: "linux",
    arch: "x64",
    version: VERSION,
    checksums: { [installer.assetName(target)]: sha256(bytes) },
    dir,
    fetch,
  });

  assert.deepEqual(asked, [url]);
  assert.equal(placed, path.join(dir, "bin", "plotplot"));
  assert.equal(execFileSync(placed).toString(), "plotplot 0.1.0\n");
  // Nothing but the executable is left behind.
  assert.deepEqual(fs.readdirSync(dir), ["bin"]);
  assert.deepEqual(fs.readdirSync(path.join(dir, "bin")), ["plotplot"]);
});

test("bytes whose digest is not the one the package carries are refused and nothing is placed", async (t) => {
  const bytes = asset(t);
  const target = installer.target("linux", "x64");
  const { fetch } = serving(installer.assetUrl(VERSION, target), bytes);
  const dir = scratch(t);
  const pinned = "0".repeat(64);

  await assert.rejects(
    installer.install({
      platform: "linux",
      arch: "x64",
      version: VERSION,
      checksums: { [installer.assetName(target)]: pinned },
      dir,
      fetch,
    }),
    (error) => error.message.includes(pinned) && error.message.includes(sha256(bytes)),
  );
  assert.deepEqual(fs.readdirSync(dir), []);
});

test("a package that carries no digest for this platform fetches nothing", async (t) => {
  const { fetch, asked } = serving("https://example.invalid/never", Buffer.alloc(0));
  const dir = scratch(t);

  await assert.rejects(
    installer.install({ platform: "linux", arch: "x64", version: VERSION, checksums: {}, dir, fetch }),
    /carries no sha256/,
  );
  assert.deepEqual(asked, []);
  assert.deepEqual(fs.readdirSync(dir), []);
});

test("a platform the release does not build is named, and nothing is fetched", async (t) => {
  const { fetch, asked } = serving("https://example.invalid/never", Buffer.alloc(0));

  await assert.rejects(
    installer.install({ platform: "linux", arch: "ia32", version: VERSION, checksums: {}, dir: scratch(t), fetch }),
    /linux-ia32/,
  );
  assert.deepEqual(asked, []);
});
