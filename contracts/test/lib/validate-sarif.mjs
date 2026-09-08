#!/usr/bin/env node
// contracts/test/lib/validate-sarif.mjs: validates one SARIF log against the vendored
// official SARIF 2.1.0 JSON schema (contracts/vendor/sarif-schema-2.1.0.json), after
// checking that vendored file's sha256 against the value pinned in contracts/pins.json.
// The schema itself declares JSON Schema draft-04 ($schema:
// http://json-schema.org/draft-04/schema#), so this uses ajv-draft-04 rather than the
// default ajv (which targets draft-07/2019-09/2020-12). The point of pinning a vendored
// copy is to validate against exactly what OASIS published, not a lookalike.
//
// Usage: node validate-sarif.mjs <sarif-file>
// Exit 0 and "valid" on stdout when the file validates; exit 1 with the ajv error list
// when it does not, or when the vendored schema's checksum does not match its pin.

import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import AjvDraft04 from "ajv-draft-04";
import addFormats from "ajv-formats";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const schemaPath = join(repoRoot, "contracts", "vendor", "sarif-schema-2.1.0.json");
const pinsPath = join(repoRoot, "contracts", "pins.json");

const target = process.argv[2];
if (!target) {
  console.error("usage: validate-sarif.mjs <sarif-file>");
  process.exit(1);
}

const pins = JSON.parse(readFileSync(pinsPath, "utf8"));
const pinnedSha256 = pins.sarif_schema && pins.sarif_schema.sha256;
if (!pinnedSha256) {
  console.error(`validate-sarif: ${pinsPath} has no sarif_schema.sha256 to check against`);
  process.exit(1);
}

const schemaBytes = readFileSync(schemaPath);
const actualSha256 = createHash("sha256").update(schemaBytes).digest("hex");
if (actualSha256 !== pinnedSha256) {
  console.error(
    `validate-sarif: vendored schema checksum mismatch: pins.json says ${pinnedSha256}, ${schemaPath} hashes to ${actualSha256}`,
  );
  process.exit(1);
}

const schema = JSON.parse(schemaBytes.toString("utf8"));
const ajv = new AjvDraft04({ strict: false, allErrors: true });
addFormats(ajv);
const validate = ajv.compile(schema);

const data = JSON.parse(readFileSync(target, "utf8"));
const ok = validate(data);

if (ok) {
  console.log(`valid: ${target}`);
  process.exit(0);
} else {
  console.error(`invalid: ${target}`);
  for (const err of validate.errors ?? []) {
    console.error(`  ${err.instancePath || "/"} ${err.message}`);
  }
  process.exit(1);
}
