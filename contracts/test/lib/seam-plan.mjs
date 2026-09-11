#!/usr/bin/env node
// contracts/test/lib/seam-plan.mjs: the plan side of contracts/test/seam.test.sh. Reads a
// map of tend2 loops with its own small reader of the loop format (FORMAT v1: the payload
// in <script type="text/markdown" id="loop">, one check per line under ## Tests), so what
// the plan must carry is derived from the map and never from tend2's own parser.
//
// Usage:
//   node seam-plan.mjs evidence-complete <map-dir> <repo-root>
//     prints the id of each loop whose every check is (code) and names an evidence file
//     that exists under <repo-root>: the loops the seam test closes with tend2 verify.
//   node seam-plan.mjs schema-errors <ajv-log>
//     reads what `ajv validate --errors=json` wrote for a refused plan and prints one line
//     naming each place the plan breaks the schema.
//   node seam-plan.mjs assert <plan.json> <map-dir> <repo-root> <cast.json> <runner> <emit-log>
//     prints one "ok - " or "not ok - " line per assertion; exit 0 when every one holds,
//     1 otherwise, 2 on a usage error.
//
// What a loop must yield, by its checks (jahala/tend 158 and 163, contracts/identifiers.md):
//   fully verified, or no open code check: no node; a fully verified loop is named in the
//     emit's preflight output.
//   every open check is (code) with an evidence path: one phased node per open check,
//     id <loop>.c<N>, phases red, impl, green, whose test is that check's verify command;
//     with two or more, a command node carrying the loop's own id closes the chain.
//   any other open check beside an open code check (a mixed loop): one prompt node, id <loop>.

import { readFileSync, readdirSync, existsSync } from "node:fs";
import { join, relative, resolve } from "node:path";

const LOOP_SUFFIX = ".tend2.html";
const LOOP_ID = /^[a-z][a-z0-9-]*$/;
const LOOP_CHECK = /^[a-z][a-z0-9-]*:c[1-9][0-9]*$/;
const PER_CHECK_NODE = /^(.+)\.c([1-9][0-9]*)$/;
const CHECK_LINE = /^- \[([ x!~])\] \(([a-z]+)\) (.*)$/;
const SHELL_OPERATORS = new Set(["&&", "||", "|", ";", ">", ">>", "<", "&", "2>", "2>&1"]);
const PHASES = ["red", "impl", "green"];

// POSIX word splitting as pleach's exec performs it (contract v1.1.4): quotes and
// backslashes honoured, no expansion. Returns null on unterminated quoting.
function toArgv(command) {
  const argv = [];
  let current = "";
  let hasToken = false;
  let quote = null;
  for (let i = 0; i < command.length; i += 1) {
    const ch = command[i];
    if (quote === null && /\s/.test(ch)) {
      if (hasToken) argv.push(current);
      current = "";
      hasToken = false;
    } else if (quote === null && (ch === '"' || ch === "'")) {
      quote = ch;
      hasToken = true;
    } else if (quote !== null && ch === quote) {
      quote = null;
    } else if (ch === "\\" && quote !== "'" && i + 1 < command.length) {
      current += command[i + 1];
      hasToken = true;
      i += 1;
    } else {
      current += ch;
      hasToken = true;
    }
  }
  if (quote !== null) return null;
  if (hasToken) argv.push(current);
  return argv;
}

// A command wrapped once for a shell (`bash -lc '<cmd>'`, `sh -c '<cmd>'`) is the command inside.
function unwrapShell(argv) {
  if (argv.length === 3 && /^(ba)?sh$/.test(argv[0].split("/").pop()) && /^-l?c$/.test(argv[1])) {
    return toArgv(argv[2]);
  }
  return argv;
}

function parseCheck(line, ordinal) {
  const match = CHECK_LINE.exec(line);
  if (match === null) return null;
  const [, box, method, rest] = match;
  let body = rest.replace(/ · by \S+$/, "");
  const stamp = / @([0-9a-f]{6,40})$/.exec(body);
  if (stamp !== null) body = body.slice(0, stamp.index);
  const sep = body.indexOf(" · ");
  const claim = sep === -1 ? body : body.slice(0, sep);
  const evidence = sep === -1 ? "" : body.slice(sep + 3).trim();
  return { ordinal, method, claim, evidence, pass: box === "x" && stamp !== null };
}

function readMap(mapDir, repoRoot) {
  const loops = [];
  for (const file of readdirSync(mapDir).filter((f) => f.endsWith(LOOP_SUFFIX)).sort()) {
    const html = readFileSync(join(mapDir, file), "utf8");
    const payload = /<script type="text\/markdown" id="loop">\n([\s\S]*?)<\/script>/.exec(html)?.[1];
    if (payload === undefined) continue;
    const lines = payload.split("\n");
    const start = lines.findIndex((l) => l.trim() === "## Tests");
    if (start === -1) continue;
    const checks = [];
    for (const line of lines.slice(start + 1)) {
      if (line.startsWith("## ")) break;
      if (!line.startsWith("- [")) continue;
      const check = parseCheck(line, checks.length + 1);
      if (check !== null) checks.push(check);
    }
    const id = file.slice(0, -LOOP_SUFFIX.length);
    loops.push({ id, fileRel: relative(repoRoot, join(mapDir, file)), checks });
  }
  return loops;
}

function shapeOf(loop) {
  const open = loop.checks.filter((c) => !c.pass);
  const openCode = open.filter((c) => c.method === "code");
  if (open.length === 0) return { kind: "verified", openCode };
  if (openCode.length === 0) return { kind: "unbuildable", openCode };
  if (open.every((c) => c.method === "code" && c.evidence !== "")) return { kind: "phased", openCode };
  return { kind: "mixed", openCode };
}

function workKind(node) {
  const work = node.work ?? {};
  if (Array.isArray(work.phases) && typeof work.test === "string") return "phased";
  if (typeof work.command === "string") return "command";
  if (typeof work.prompt === "string") return "prompt";
  return "unknown";
}

function loopOfNode(id) {
  const match = PER_CHECK_NODE.exec(id);
  return match === null ? { loop: id, ordinal: null } : { loop: match[1], ordinal: Number(match[2]) };
}

// The check's verify command: the runner with the check's evidence substituted (what
// `tend2 verify` runs for that check), or a `verify <loop file> --check <N>` invocation.
function isVerifyCommandOf(test, loop, check, runner, repoRoot) {
  const argv = toArgv(test);
  if (argv === null) return false;
  const inner = unwrapShell(argv);
  if (inner === null) return false;
  const runnerArgv = toArgv(runner);
  const expected = runnerArgv === null ? null : runnerArgv.map((t) => t.replaceAll("{evidence}", check.evidence));
  if (expected !== null && expected.length === inner.length && expected.every((t, i) => t === inner[i])) return true;
  const file = inner.findIndex((t) => t === loop.fileRel || t === resolve(repoRoot, loop.fileRel));
  const checkAt = inner.findIndex((t) => t === "--check");
  const checkN = checkAt !== -1 ? inner[checkAt + 1] : inner.find((t) => t.startsWith("--check="))?.slice(8);
  return inner.includes("verify") && file !== -1 && checkN === String(check.ordinal);
}

function describe(nodes) {
  return nodes.length === 0 ? "none" : nodes.map((n) => `${n.id} (${workKind(n)})`).join(", ");
}

// ajv-cli's --errors=json log: a "<file> invalid" line, then the error array.
function schemaErrors(log) {
  const start = log.indexOf("[");
  let errors;
  try {
    errors = start === -1 ? null : JSON.parse(log.slice(start));
  } catch {
    errors = null;
  }
  if (!Array.isArray(errors) || errors.length === 0) return log.trim().split("\n").join(" ");
  return errors.map((e) => {
    const at = e.instancePath === "" ? "the root" : e.instancePath;
    return e.keyword === "additionalProperties"
      ? `the plan carries ${e.params.additionalProperty} at ${at}, a key the schema does not allow there`
      : `${at} ${e.message}`;
  }).join("; ");
}

function main(argv) {
  const [mode, ...rest] = argv;
  if (mode === "evidence-complete" && rest.length === 2) {
    const [mapDir, repoRoot] = rest;
    for (const loop of readMap(mapDir, repoRoot)) {
      const complete = loop.checks.length > 0
        && loop.checks.every((c) => c.method === "code" && c.evidence !== "" && existsSync(join(repoRoot, c.evidence)));
      if (complete) console.log(loop.id);
    }
    return 0;
  }
  if (mode === "schema-errors" && rest.length === 1) {
    console.log(schemaErrors(readFileSync(rest[0], "utf8")));
    return 0;
  }
  if (mode !== "assert" || rest.length !== 6) {
    console.error("usage: seam-plan.mjs evidence-complete <map-dir> <repo-root>");
    console.error("       seam-plan.mjs schema-errors <ajv-log>");
    console.error("       seam-plan.mjs assert <plan.json> <map-dir> <repo-root> <cast.json> <runner> <emit-log>");
    return 2;
  }
  const [planPath, mapDir, repoRoot, castPath, runner, emitLogPath] = rest;

  let failed = false;
  const assert = (holds, message, reason) => {
    if (holds) {
      console.log(`ok - ${message}`);
    } else {
      console.log(`not ok - ${message}${reason ? ` (${reason})` : ""}`);
      failed = true;
    }
  };

  let plan;
  try {
    plan = JSON.parse(readFileSync(planPath, "utf8"));
  } catch (error) {
    assert(false, "the emitted plan parses as JSON", String(error.message));
    return 1;
  }
  const nodes = Array.isArray(plan.nodes) ? plan.nodes : [];
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const cast = JSON.parse(readFileSync(castPath, "utf8"));
  const emitLog = readFileSync(emitLogPath, "utf8");
  const loops = readMap(mapDir, repoRoot);
  const loopIds = new Set(loops.map((l) => l.id));
  const anyPhased = nodes.some((n) => workKind(n) === "phased");

  const uncastLoops = loops.filter((l) => typeof cast[l.id]?.provider !== "string" || typeof cast[l.id]?.model !== "string");
  assert(uncastLoops.length === 0, "the cast file names a provider and a model for every loop in the map",
    `no cast for ${uncastLoops.map((l) => l.id).join(", ")}`);

  const wrongCast = nodes
    .filter((n) => workKind(n) === "prompt" || workKind(n) === "phased")
    .filter((n) => {
      const want = cast[loopOfNode(n.id).loop];
      return want === undefined || n.worker?.provider !== want.provider || n.worker?.model !== want.model;
    });
  assert(wrongCast.length === 0,
    "every node pleach dispatches a worker for (prompt or phased) carries the provider and model the cast file names for its loop",
    wrongCast.map((n) => `${n.id} carries ${JSON.stringify(n.worker ?? null)}`).join("; "));

  const badSetup = nodes.flatMap((n) => {
    if (typeof n.setup !== "string") return [`${n.id} carries no setup`];
    const argv = toArgv(n.setup);
    if (argv === null) return [`${n.id}'s setup has unterminated quoting`];
    const ops = argv.filter((t) => SHELL_OPERATORS.has(t));
    return ops.length > 0 ? [`${n.id}'s setup holds bare ${ops.join(" ")}`] : [];
  });
  assert(nodes.length > 0 && badSetup.length === 0,
    "every node carries the setup, as one argv with no bare shell operator token",
    nodes.length === 0 ? "the plan has no nodes" : badSetup.join("; "));

  for (const loop of loops) {
    const shape = shapeOf(loop);
    const own = nodes.filter((n) => loopOfNode(n.id).loop === loop.id);

    if (shape.kind === "verified" || shape.kind === "unbuildable") {
      assert(own.length === 0,
        `${loop.id} yields no node, since ${shape.kind === "verified" ? "every check is verified" : "it has no open code check"}`,
        `the plan carries ${describe(own)}`);
      if (shape.kind === "verified") {
        const idWord = new RegExp(`(^|[^a-z0-9-])${loop.id}([^a-z0-9-]|$)`);
        const named = emitLog.split("\n").some((l) => idWord.test(l) && /\b(verified|done)\b/i.test(l));
        assert(named, `emit-plan's preflight output names ${loop.id} as fully verified and left out`,
          `no line of the emit output names ${loop.id} as verified or done, so a left-out loop is silent, jahala/tend 177`);
      }
      continue;
    }

    if (shape.kind === "mixed") {
      const node = byId.get(loop.id);
      assert(own.length === 1 && node !== undefined && workKind(node) === "prompt",
        `${loop.id}, a mixed loop, yields one prompt node with its own id`,
        `the plan carries ${describe(own)}`);
      continue;
    }

    const chain = shape.openCode.length >= 2;
    for (const check of shape.openCode) {
      const perCheckId = `${loop.id}.c${check.ordinal}`;
      const node = byId.get(perCheckId) ?? (chain ? undefined : byId.get(loop.id));
      const message = `${loop.id}:c${check.ordinal} yields a phased node ${perCheckId}, phases red, impl, green, whose test is that check's verify command`;
      if (node === undefined) {
        const whole = byId.get(loop.id);
        const split = whole !== undefined && workKind(whole) !== "command"
          ? `; ${loop.id} is one ${workKind(whole)} node for the whole loop, so the tend2 under test lacks the node per open code check that jahala/tend 158 added`
          : "";
        const phased = anyPhased ? "" : "; no node in the plan is phased, since tend2 emit-plan emits no {test, phases} work, jahala/tend 163";
        assert(false, message, `no node ${perCheckId}${split}${phased}`);
        continue;
      }
      if (workKind(node) !== "phased") {
        assert(false, message,
          `${node.id} is a ${workKind(node)} node, since tend2 emit-plan emits no {test, phases} work, jahala/tend 163`);
        continue;
      }
      const phases = node.work.phases.map((p) => p.phase);
      const prompts = node.work.phases.every((p) => typeof p.prompt === "string" && p.prompt.trim() !== "");
      const inOrder = phases.length === PHASES.length && PHASES.every((p, i) => phases[i] === p);
      const verifies = isVerifyCommandOf(node.work.test, loop, check, runner, repoRoot);
      assert(inOrder && prompts && verifies, message, [
        inOrder ? "" : `its phases are ${phases.join(", ") || "empty"}`,
        prompts ? "" : "a phase carries no prompt",
        verifies ? "" : `its test ${JSON.stringify(node.work.test)} is not the verify command of ${check.evidence}`,
      ].filter(Boolean).join("; "));
    }

    if (chain) {
      const closing = byId.get(loop.id);
      const upstream = new Set();
      const walk = (id) => {
        for (const need of byId.get(id)?.needs ?? []) {
          if (!upstream.has(need)) { upstream.add(need); walk(need); }
        }
      };
      if (closing !== undefined) walk(closing.id);
      const argv = closing !== undefined && workKind(closing) === "command" ? unwrapShell(toArgv(closing.work.command) ?? []) ?? [] : [];
      const verifiesLoop = argv.includes("verify") && argv.some((t) => t === loop.fileRel || t === resolve(repoRoot, loop.fileRel));
      const behind = shape.openCode.map((c) => `${loop.id}.c${c.ordinal}`).filter((id) => !upstream.has(id));
      assert(closing !== undefined && workKind(closing) === "command" && verifiesLoop && behind.length === 0,
        `${loop.id} closes with a command node carrying its own id that verifies ${loop.fileRel} after every per-check node`,
        closing === undefined ? `no node ${loop.id}`
          : workKind(closing) !== "command" ? `${loop.id} is a ${workKind(closing)} node, so the tend2 under test lacks the closing command node that jahala/tend 158 added`
          : [verifiesLoop ? "" : `its command does not verify ${loop.fileRel}`,
            behind.length === 0 ? "" : `it does not need ${behind.join(", ")}`].filter(Boolean).join("; "));
    }
  }

  const unresolved = nodes.flatMap((n) => {
    const { loop, ordinal } = loopOfNode(n.id);
    const home = loops.find((l) => l.id === loop);
    if (!loopIds.has(loop) || !LOOP_ID.test(loop)) return [`${n.id} names no loop id in the map`];
    if (ordinal === null) return [];
    const check = home.checks[ordinal - 1];
    if (!LOOP_CHECK.test(`${loop}:c${ordinal}`)) return [`${n.id} does not read as a loop id and check ordinal`];
    if (check === undefined || check.method !== "code" || check.pass) return [`${n.id} names no open code check of ${loop}`];
    return [];
  });
  assert(nodes.length > 0 && unresolved.length === 0,
    "every node id resolves to a loop id in the map, and a per-check id <loop>.c<N> to that loop's open code check <loop-id>:c<N>, per contracts/identifiers.md",
    nodes.length === 0 ? "the plan has no nodes" : unresolved.join("; "));

  const kinds = new Set(nodes.map(workKind));
  const missing = ["prompt", "phased", "command"].filter((k) => !kinds.has(k));
  assert(missing.length === 0, "the plan covers prompt, phased and command nodes",
    `it carries no ${missing.join(" and no ")} node`);

  return failed ? 1 : 0;
}

process.exit(main(process.argv.slice(2)));
