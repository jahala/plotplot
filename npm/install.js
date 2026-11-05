#!/usr/bin/env node
"use strict";

// The npm package's postinstall: fetch the stem built for this machine from the GitHub
// release at the package's own version, refuse it unless its sha256 is the one this package
// carries in checksums.json (written by the release workflow from the release's own
// SHA256SUMS), and place it under bin/. Bytes that are not the pinned bytes are never
// unpacked, the same rule the stem keeps for every judge it fetches.

const crypto = require("node:crypto");
const fs = require("node:fs");
const https = require("node:https");
const os = require("node:os");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const REPOSITORY = "https://github.com/jahala/plotplot";
const FROM_SOURCE = `cargo install --locked --git ${REPOSITORY} plotplot`;

// The most bytes one asset may be. The stem is a few megabytes; this is far past it and far
// short of a machine's memory.
const MAX_BYTES = 128 * 1024 * 1024;

const TARGETS = {
  "darwin-arm64": "aarch64-apple-darwin",
  "darwin-x64": "x86_64-apple-darwin",
  "linux-arm64": "aarch64-unknown-linux-musl",
  "linux-x64": "x86_64-unknown-linux-musl",
  "win32-x64": "x86_64-pc-windows-msvc",
};

/** The Rust target the release builds for this platform and architecture, or null. */
function target(platform, arch) {
  return TARGETS[`${platform}-${arch}`] ?? null;
}

/** The release asset's file name for a target. */
function assetName(target) {
  return `plotplot-${target}.tar.gz`;
}

/** Where the release keeps that asset at a version. */
function assetUrl(version, target) {
  return `${REPOSITORY}/releases/download/v${version}/${assetName(target)}`;
}

/** The executable's name on a platform. */
function binaryName(platform) {
  return platform === "win32" ? "plotplot.exe" : "plotplot";
}

/** The bytes at an https url, following redirects, up to MAX_BYTES. */
function download(url, redirects = 5) {
  return new Promise((resolve, reject) => {
    if (!url.startsWith("https://")) {
      reject(new Error(`${url} is not https, and the stem is fetched over nothing else`));
      return;
    }
    https
      .get(url, (response) => {
        const { statusCode, headers } = response;
        if (statusCode >= 300 && statusCode < 400 && headers.location && redirects > 0) {
          response.resume();
          resolve(download(new URL(headers.location, url).toString(), redirects - 1));
          return;
        }
        if (statusCode !== 200) {
          response.resume();
          reject(new Error(`${url}: the server answered ${statusCode}`));
          return;
        }
        const chunks = [];
        let size = 0;
        response.on("data", (chunk) => {
          size += chunk.length;
          if (size > MAX_BYTES) {
            response.destroy(new Error(`${url}: more than ${MAX_BYTES} bytes`));
            return;
          }
          chunks.push(chunk);
        });
        response.on("end", () => resolve(Buffer.concat(chunks)));
        response.on("error", reject);
      })
      .on("error", (error) => reject(new Error(`${url}: ${error.message}`)));
  });
}

/**
 * Fetch, verify and place the stem for one platform under `dir/bin/`, and answer its path.
 * Every failure is a rejection naming what failed; nothing is placed unless the bytes are
 * the pinned bytes.
 */
async function install({ platform, arch, version, checksums, dir, fetch }) {
  const wanted = target(platform, arch);
  if (wanted === null) {
    throw new Error(
      `no release is built for ${platform}-${arch}; the release builds ${Object.keys(TARGETS).join(", ")}`,
    );
  }
  const name = assetName(wanted);
  const pinned = checksums[name];
  if (typeof pinned !== "string" || !/^[0-9a-f]{64}$/.test(pinned)) {
    throw new Error(`this package carries no sha256 for ${name}, so it installs nothing`);
  }

  const url = assetUrl(version, wanted);
  const bytes = await fetch(url);
  const actual = crypto.createHash("sha256").update(bytes).digest("hex");
  if (actual !== pinned) {
    throw new Error(`${url}: expected sha256 ${pinned}, got ${actual}; nothing was installed`);
  }

  const unpacked = fs.mkdtempSync(path.join(os.tmpdir(), "plotplot-npm-"));
  try {
    const archive = path.join(unpacked, name);
    fs.writeFileSync(archive, bytes);
    execFileSync("tar", ["-xzf", archive, "-C", unpacked], { stdio: "ignore" });
    const binary = binaryName(platform);
    const from = path.join(unpacked, binary);
    if (!fs.existsSync(from)) {
      throw new Error(`${url} carries no ${binary} at its root`);
    }
    const bin = path.join(dir, "bin");
    fs.mkdirSync(bin, { recursive: true });
    const placed = path.join(bin, binary);
    fs.copyFileSync(from, placed);
    fs.chmodSync(placed, 0o755);
    return placed;
  } finally {
    fs.rmSync(unpacked, { recursive: true, force: true });
  }
}

/** The digests the release workflow wrote into this package, or none. */
function packagedChecksums(dir) {
  const file = path.join(dir, "checksums.json");
  return fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, "utf8")) : {};
}

if (require.main === module) {
  const dir = __dirname;
  const { version } = require("./package.json");
  install({
    platform: process.platform,
    arch: process.arch,
    version,
    checksums: packagedChecksums(dir),
    dir,
    fetch: download,
  }).then(
    (placed) => console.log(`plotplot ${version}: installed at ${placed}`),
    (error) => {
      console.error(`plotplot: ${error.message}`);
      console.error(`plotplot: to build it from source instead: ${FROM_SOURCE}`);
      process.exit(1);
    },
  );
}

module.exports = { target, assetName, assetUrl, binaryName, install };
