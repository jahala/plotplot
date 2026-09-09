# The stem build: the brief every stem worker reads

You are building one part of `plotplot`, the stem: the Rust binary at the root of this
repository that plants the garden into any harness and proves it is planted. Read, in this
order, before touching code: `CLAUDE.md`, `docs/building-the-garden.md` (the law),
`docs/plans/stem.md` (the plan), `docs/tend2/stem.tend2.html` (the loop; read-only for you),
then this page. `docs/plans/friction-ledger.md` §3 and §5 and `docs/plans/receipts.md` §3 and
§4 matter for the faces. Your node's prompt names your slice; this page is the shared
contract so that slices built in parallel fit.

Status: brief, 2026-09-08, written by the stem builder before the first node ran, against
contracts v1.2.0.

## 1. Rules that bind every node

- The failing test is the spec. Write it first, watch it fail, then make it pass. A test
  that passes without the work is not a test.
- No stubs, mocks, TODOs, placeholders, fallbacks, or `unimplemented!`. A face that is not
  in your slice does not exist yet; do not add it as an empty arm.
- Library code never panics on input: no `unwrap`, `expect`, `panic!`, or indexing that can
  fail on data from a file, the network, stdin, or the environment. Failure lives in the
  return type (`crate::error::Result`). `main.rs` is the only place that turns an error into
  an exit code.
- Side effects at the edge. Generators and deciders are pure functions over values
  (`&[Bed]` in, `FileTree` out; `&Payload` in, `Vec<Record>` out). Reading and writing files,
  spawning processes and touching git happen in thin functions that call the pure ones.
- Dependencies are parameters, never globals or `std::env::current_dir()` inside library
  code. The repository root is passed in.
- Delete with `trash`, never `rm`. Never `git reset`. Never touch the stash. Do not push,
  tag or publish.
- Match the vendor facts in §5 exactly; they were verified against the binaries installed
  on the build machine, not against documentation.
- Scratch and probe files go under `.loop-scratch/` only.
- Your final message states what changed, the test output before and after, what you could
  not verify, and what you assumed. No time estimates, no promises about later work.

## 2. Toolchain and crate

- Rust stable, edition 2024, `rust-version = "1.85"`. `Cargo.toml` at the repository root,
  package `plotplot`, version `0.1.0`, one binary `plotplot` (`src/main.rs`) over one library
  (`src/lib.rs`). `Cargo.lock` is committed.
- Dependencies, and nothing beyond these without a sentence in your final message saying
  why: `clap` (derive), `serde`, `serde_json`, `toml`, `sha2`, `hex`, `jsonschema`
  (default-features off), `ureq` (rustls, for the lock later). Dev: `assert_cmd`,
  `predicates`, `tempfile`. No async runtime. No `anyhow`; the crate has its own error type.
- Static binary. Nothing at runtime beyond libc; hooks must run where Node is absent.
- Gates every node passes before it stops, in this order:

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

- Tests: unit tests beside the code (`#[cfg(test)] mod tests`), integration tests under
  `tests/` driving the binary with `assert_cmd`, fixtures under `tests/fixtures/`. Every
  fixture payload is real: copied from a harness's documented shape and named by harness and
  event (`tests/fixtures/payloads/claude/PreToolUse.json`).

## 3. Layout on disk: what the stem writes into a planted repository

Human-authored files stay under `docs/tend2/` and root config; everything machine-written
goes under `.plotplot/` (the footprint rule, jahala/plotplot issue 18). The only files the
stem writes outside `.plotplot/` are the ones a vendor or git requires there.

```
garden.lock                       the pinned judges (lock schema); committed
garden.json                       the planted repository's own manifest, kind ["repository"] (v1.3.0); committed
AGENTS.md                         the garden block between <!-- plotplot:begin --> and <!-- plotplot:end -->
.githooks/pre-commit              git hooks; core.hooksPath points here; committed
.githooks/pre-push
.githooks/pre-rebase
.githooks/post-commit
.plotplot/bin/<judge>             fetched judges, sha256-verified; ignored
.plotplot/beds/<name>/garden.json the manifest read from each judge's artifact; ignored
.plotplot/bundles/claude/         the Claude Code plugin
.plotplot/bundles/gemini/         the Gemini CLI extension
.plotplot/bundles/codex/          the Codex plugin
.plotplot/friction/<yyyy-mm>.jsonl
.plotplot/friction/state/<session_id>.json
.plotplot/receipts/drafts/<session_id>.json
```

`src/layout.rs` owns these paths as functions of the repository root; no other module
spells them.

## 4. The module contract

Module names and the public shapes below are fixed. Add fields and functions; do not rename
or remove. A node that needs a change to a shape it does not own says so in its final
message rather than making it.

### `error.rs`

```rust
pub enum Error {
    Io { path: PathBuf, source: std::io::Error },
    Json { path: Option<PathBuf>, source: serde_json::Error },
    Toml { path: PathBuf, message: String },
    Manifest { bed: String, problem: String },
    Harness { problem: String },
    Git { command: String, stderr: String },
    Bed { bed: String, problem: String },
}
pub type Result<T> = std::result::Result<T, Error>;
impl std::fmt::Display for Error   // one line, the path first when there is one
impl std::error::Error for Error
```

### `harness.rs`: the three vendors as the stem sees them

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Harness { Claude, Gemini, Codex }

impl Harness {
    pub const ALL: [Harness; 3];
    pub fn name(self) -> &'static str;                 // "claude" | "gemini" | "codex"
    pub fn events(self) -> &'static [&'static str];    // exactly the lists in §5
    pub fn has_event(self, event: &str) -> bool;
    pub fn session_end_event(self) -> &'static str;    // Claude "SessionEnd", Gemini "SessionEnd", Codex "Stop"
    pub fn before_tool_event(self) -> &'static str;    // Claude "PreToolUse", Gemini "BeforeTool", Codex "PreToolUse"
    pub fn bundle_root_variable(self) -> &'static str; // "${CLAUDE_PLUGIN_ROOT}" | "${extensionPath}" | "${CLAUDE_PLUGIN_ROOT}"
}
impl std::str::FromStr for Harness;   // the three names, nothing else
impl std::fmt::Display for Harness;

/// The fields of a hook payload the stem reads. Parsed leniently: a missing field is None,
/// an unknown field is ignored, and `raw` keeps the whole document for the beds.
pub struct Payload {
    pub harness: Harness,
    pub event: String,                       // hook_event_name
    pub session_id: Option<String>,
    pub cwd: Option<PathBuf>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_call_id: Option<String>,        // Claude tool_use_id; Gemini/Codex when present
    pub tool_response: Option<serde_json::Value>,
    pub tool_error: Option<String>,          // Claude PostToolUseFailure error; Gemini AfterTool error
    pub model: Option<String>,
    pub transcript_path: Option<PathBuf>,
    pub turn_id: Option<String>,
    pub agent_id: Option<String>,
    pub reason: Option<String>,              // Claude SessionEnd `reason`; Gemini `reason`
    pub trigger: Option<String>,             // PreCompact / PreCompress trigger
    pub source: Option<String>,              // SessionStart source
    pub raw: serde_json::Value,
}
pub fn parse_payload(harness: Harness, json: &str) -> Result<Payload>;
// Error::Harness when the document is not a JSON object or has no hook_event_name.

/// What a hook can say back. The dispatcher renders it in the vendor's shape.
pub enum Answer {
    Allow,
    Deny { reason: String },     // a tool call refused (PreToolUse / BeforeTool)
    Block { reason: String },    // a Stop / AfterAgent refused
    Cannot { reason: String },   // the hook could not judge; exit 3
}
pub fn render_answer(harness: Harness, event: &str, answer: &Answer) -> String;
pub fn exit_code(answer: &Answer) -> i32;     // Allow 0, Deny 2, Block 2, Cannot 3
```

### `bed.rs`: what the stem needs to know about a planted bed

This is the stem's internal view. `manifest.rs` produces it from `garden.json`; every other
module consumes it and never reads a manifest directly.

```rust
pub struct Bed {
    pub name: String,
    pub version: String,
    pub binary: Option<String>,                  // the executable's file name under .plotplot/bin/; None for a bed with no cli and no binary_name
    pub skill: Option<PathBuf>,                  // SKILL.md, relative to the bed's artifact root
    pub hooks: BTreeMap<Harness, Vec<HookEntry>>,
    pub git_hooks: Vec<GitHook>,
    pub mcp: Option<McpServer>,                  // channel beds only
    pub check: Option<String>,                   // the gate command, SARIF on stdout
}
pub struct HookEntry { pub event: String, pub matcher: Option<String> }   // "PreToolUse:Bash" → event PreToolUse, matcher Bash
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GitHook { PreCommit, PrePush, PreRebase, PostCommit }
impl GitHook { pub fn file_name(self) -> &'static str; }   // "pre-commit" …
pub struct McpServer { pub command: String, pub args: Vec<String>, pub env: Vec<String> }   // env: the variable NAMES the server reads

pub fn beds_registered_for(beds: &[Bed], harness: Harness, event: &str) -> Vec<&Bed>;
```

### `manifest.rs`: `garden.json`, the contracts' manifest

The schema is `contracts/manifest.schema.json` (contracts v1.3.0; the bed shape is v1.2.0's,
v1.3.0 adds the `kind: ["repository"]` case a planted repository's own `garden.json` takes,
which is never a `Bed`),
embedded with `include_str!` and enforced with `jsonschema` before serde parsing, so a
manifest the contracts reject never becomes a `Bed`. The fixtures under
`contracts/fixtures/manifest/` are the test corpus: every positive fixture parses, every
`invalid-*` fixture is refused with `Error::Manifest` naming the bed and the problem.

What the stem reads, and how it maps to `Bed`:

| manifest | `Bed` |
|---|---|
| `name`, `version` | `name`, `version` |
| `install.binary_name` when present, else `faces.cli` | `binary` (the executable's file name inside the artifact and under `.plotplot/bin/`); a bed with neither has no binary and registers no hooks |
| `faces.skill` | `skill` |
| `faces.hooks.<harness>` entries `"Event"` or `"Event:Matcher"` | `hooks[harness]` as `HookEntry { event, matcher }`; a harness name the stem does not know is `Error::Manifest`; an event the harness does not have (§5) is `Error::Manifest` |
| `faces.git` | `git_hooks`; `reference-transaction` is accepted by the schema but the stem does not install it (`Error::Manifest`, so the gap is loud) |
| `faces.mcp.command`, `faces.mcp.args`, `faces.mcp.env` | `mcp` (`McpServer { command, args, env }`, env being the variable names the server reads); when only `tools` is present the bed is a channel the stem cannot plant yet, `mcp` is `None`, and `bundle build` says so on stderr |
| `check` | `check` (null stays `None`) |

```rust
pub struct Manifest { /* serde model of garden.json; unknown fields kept in `extra` */ }
pub fn parse_manifest(json: &str) -> Result<Manifest>;   // schema first, then serde
pub fn to_bed(manifest: &Manifest) -> Result<Bed>;
pub fn load_beds(root: &Path) -> Result<Vec<Bed>>;       // every .plotplot/beds/*/garden.json, sorted by name
```

### `lock.rs`: `garden.lock`, the pinned judges

The schema is `contracts/lock.schema.json`, embedded and enforced the same way after the
TOML is parsed to a JSON value. Fixture: `contracts/fixtures/garden.lock`.

```rust
pub struct Lock { pub season: String, pub judges: BTreeMap<String, Judge> }
pub struct Judge { pub version: String, pub npm: Option<String>, pub platforms: BTreeMap<String, Artifact> }
pub struct Artifact { pub url: String, pub sha256: String }
pub fn parse_lock(toml: &str) -> Result<Lock>;
pub fn read_lock(root: &Path) -> Result<Option<Lock>>;   // None when garden.lock is absent
pub fn platform() -> &'static str;                        // the Rust target triple this binary was built for
```

Fetching and verifying judges (`plotplot lock verify`) is a later node; this wave reads
the lock for `version` and `doctor`.

### `bundle/`: the three vendor bundles from one input

```rust
pub struct SkillFile { pub bed: String, pub content: String }
pub struct BundleInput<'a> {
    pub stem_version: &'a str,
    pub season: &'a str,
    pub beds: &'a [Bed],
    pub skills: &'a [SkillFile],
}
/// Relative path → bytes. BTreeMap so output order is deterministic.
pub struct FileTree(pub BTreeMap<PathBuf, Vec<u8>>);
pub fn generate(harness: Harness, input: &BundleInput) -> Result<FileTree>;   // pure
pub fn write(tree: &FileTree, root: &Path) -> Result<Vec<PathBuf>>;          // the edge
pub enum Difference { Missing(PathBuf), Extra(PathBuf), Changed(PathBuf) }
pub fn diff(tree: &FileTree, root: &Path) -> Result<Vec<Difference>>;        // byte for byte; doctor uses it
```

Every hook entry in every bundle is one dispatcher call and nothing else:
`"<root-variable>/bin/plotplot" hook <harness> <event>` where the root variable is
`Harness::bundle_root_variable`. One entry per event, without a vendor matcher: the
dispatcher applies each bed's declared matcher itself (a bed on `PreToolUse:Bash` is called
only when `tool_name` is `Bash`), so a tool call never fires the dispatcher twice. Timeouts
are in the vendor's unit (§5): 10 s for tool events, 5 s for a stop, 1 s for the session-end
event. `bin/plotplot` inside a bundle is a copy of the running stem
binary, written by the install step, excluded from `generate` and compared by sha256 in
`doctor`. The stem registers its own entries on every harness even when no bed declared the
event: the before-tool event (the deny list), every event a friction kind derives from
(friction plan §3, table of kinds), and the session-end event (the receipt draft). Beds'
entries come from `Bed::hooks`; matchers are carried where the vendor has them.

An MCP entry is written as `{"command": <command>, "args": [<args>], "env": {<NAME>: "${<NAME>}"}}`
with the args as declared (relative to the artifact root, which the install step lays out
under the bundle's `bin/<bed>/`, so the entry's `cwd`-free form works from any directory
only when the command resolves on PATH; a relative arg is prefixed with the vendor's root
variable and `bin/<bed>/`). The `${NAME}` form is what all three vendors expand from the
planter's environment.

Three bundles must carry identical hook, skill and MCP sets; a test asserts that from the
generated trees, harness names aside.

### `deny.rs`: the hard-limit deny list

A pure decision over a `Payload` for the before-tool event: `git commit`, `git push` or
`git merge` carrying `--no-verify` or `-n`; a `Write`/`Edit`/`tilth_write`/shell redirection
targeting `garden.lock`, `.githooks/`, `.plotplot/bin/` or `.plotplot/beds/`. Nothing else.
The reason names the rule and the fix.

```rust
pub fn decide(payload: &Payload) -> Answer;
```

### `friction.rs`: the emitter face

The record shape, the envelope, the pinned kinds and the per-kind required attributes are
`contracts/friction-profile.md`, exactly; `docs/plans/friction-ledger.md` §3 to §5 is the
reasoning behind it. `plotplot.harness` values are `claude-code`, `gemini-cli`, `codex-cli`.
Required on every record: `time` (UTC, ending in `Z`), `event.name` (`plotplot.friction`),
`plotplot.kind`, `plotplot.harness`, `gen_ai.conversation.id`, `plotplot.count`.
`gen_ai.request.model` is present only when the payload carries a model. Never present:
prompt text, tool output, file content, user names, absolute home paths; a path outside the
repository becomes `plotplot.path.kind: "outside"` with no `plotplot.path`. Codex has no
SessionEnd (§5): its `session.ended` record derives from `Stop`.

```rust
pub struct Record { /* serde, field names as the profile spells them */ }
pub struct SessionState { /* path counters, last input hashes, test commands seen */ }
pub fn derive(root: &Path, payload: &Payload, state: &mut SessionState) -> Vec<Record>;  // pure over its arguments
pub fn emit(root: &Path, payload: &Payload, now: OffsetDateTimeLike) -> Result<usize>;   // load state, derive, append, save; number of lines written
```

`emit` appends to `.plotplot/friction/<yyyy-mm>.jsonl`, keeps state under
`.plotplot/friction/state/<session_id>.json`, prunes the state at the session-end event, and
never returns an error that would block a hook: the dispatcher logs and continues.

### `receipt.rs`: the draft face (v0)

```rust
pub struct Draft { /* the counters a SessionEnd can know: harness, version if the payload has it, model, session id, tool-call counts by name, wall seconds when derivable, friction summary digest */ }
pub fn draft(root: &Path, payload: &Payload) -> Result<PathBuf>;   // writes .plotplot/receipts/drafts/<session_id>.json
```

Fields follow `docs/plans/receipts.md` §3; anything the payload cannot supply is `null`,
never an estimate.

### `hook.rs`: the dispatcher

```rust
pub struct Registered { pub bed: String, pub program: PathBuf }   // resolved to .plotplot/bin/<binary>
pub struct BedResult { pub bed: String, pub code: i32, pub stdout: String, pub stderr: String }
pub struct Dispatch { pub code: i32, pub stdout: String, pub stderr: String }
pub fn fan_out(harness: Harness, payload_json: &str, beds: &[Registered], timeout: Duration) -> Vec<BedResult>;  // parallel, one thread per bed
pub fn merge(results: &[BedResult]) -> Dispatch;   // any 2 → 2, first denial's stdout leads, the rest follow on stderr; else any 3 → 3 with each reason on stderr; else 0
pub fn dispatch(root: &Path, harness: Harness, event: &str, payload_json: &str) -> Dispatch;  // deny list first; then beds; then friction; then receipt draft on the session-end event
```

The contract with beds (cape-town, 2026-09-08): `<bed> hook <harness>` reads the vendor's
payload on stdin, writes the vendor's answer on stdout, exits 0 to allow, 2 to deny, 3 when
it cannot judge. The dispatcher's own overhead, measured from process start to exit with no
beds registered and the friction and receipt faces running, is under 50 ms; an integration
test asserts it.

### `doctor.rs`: static mode

```rust
pub struct Finding { pub check: &'static str, pub ok: bool, pub detail: String }
pub fn run_static(root: &Path, home: &Path) -> Result<Vec<Finding>>;
pub fn render(findings: &[Finding]) -> String;   // a table; exit 0 when every finding is ok, else 3
```

Checks, each one finding: every bundle present and byte-identical to the regenerated one;
every hook entry in every bundle names the stem binary; `core.hooksPath` is `.githooks`;
each of the four hook files present and executable; the garden block in `AGENTS.md`
current; the receipts refspec present in the fetch and push refspecs; Codex trust for this
repository (`<home>/.codex/config.toml`, `[projects."<abs path>"] trust_level = "trusted"`),
reported as awaiting trust when absent, never as failure of the stem. Judges against the
lock are added once `lock.rs` exists.

### `plant/`: the pieces `init` writes, as pure renderers

```rust
pub mod garden_block;   // render(season, beds) -> String; replace_in(agents_md: &str, block: &str) -> String  (markers, in place; untouched elsewhere)
pub mod githooks;       // render(hook: GitHook, beds: &[Bed]) -> String  (sh, calls .plotplot/bin/<bed> guard <hook> for beds that declared it; post-commit calls plotplot receipt seal)
pub mod gitconfig;      // desired(root) -> Vec<(key, value)>; read(root) -> Result<Vec<(key, value)>>; apply(root, missing) -> Result<()>  via the git binary
```

The garden block is at most ten lines: what is planted and at which versions, the season,
how to orient (`tend2 next docs/tend2`), the gates (`plotplot check`), help
(`plotplot doctor`), and that hard limits are enforced at the boundary.

### `cli.rs` and `main.rs`

`clap` derive. `main.rs` parses, calls `cli::run(args, root, &mut stdout, &mut stderr) -> i32`,
exits with it. Each face's handler lives in its module; `cli.rs` holds only the argument
types and the match. Faces land as their nodes land; the enum grows one variant per face.

```
plotplot version                          the binary's version; the lock's season and each judge when garden.lock is present
plotplot hook <harness> <event>           the dispatcher (stdin: the payload)
plotplot bundle build [claude|gemini|codex]   regenerate under .plotplot/bundles/<harness>/ from .plotplot/beds/*/garden.json
plotplot friction emit --harness <h>      the emitter alone (stdin: the payload)
plotplot receipt draft --harness <h>      the draft alone (stdin: the payload)
plotplot doctor                           static mode; exit 0 or 3
```

## 5. Vendor facts, verified 2026-09-08 on the build machine

Claude Code 2.1.265, Gemini CLI 0.46.0, Codex CLI 0.133.0. Where these differ from
`docs/plans/stem.md` §2 or `docs/plans/friction-ledger.md` §4, these win, and the
difference is recorded on the stem loop's Tried.

| | Claude Code | Gemini CLI | Codex CLI |
|---|---|---|---|
| Bundle | `.claude-plugin/plugin.json` (name, version, description, author), `hooks/hooks.json`, `.mcp.json`, `skills/<name>/SKILL.md`, `bin/` | `gemini-extension.json` (name, version, description, mcpServers, contextFileName), `hooks/hooks.json`, `skills/<name>/SKILL.md`, `bin/` | `.codex-plugin/plugin.json` (name, version, description, `"skills": "./skills/"`, `"hooks": "./hooks.json"`, `"mcpServers": "./.mcp.json"`), `hooks.json`, `.mcp.json`, `skills/<name>/SKILL.md`, `bin/` |
| Root variable in commands | `${CLAUDE_PLUGIN_ROOT}` | `${extensionPath}` | `${CLAUDE_PLUGIN_ROOT}` (the binary also accepts `${PLUGIN_ROOT}`) |
| Hook file shape | `{"hooks": {"<Event>": [{"matcher": "...", "hooks": [{"type": "command", "command": "...", "timeout": <seconds>}]}]}}` | same shape; `timeout` in milliseconds; `matcher` optional | same shape as Claude; `timeout` in seconds |
| Events | PreToolUse, PostToolUse, PostToolUseFailure, PermissionDenied, PermissionRequest, Notification, UserPromptSubmit, SessionStart, SessionEnd, Stop, StopFailure, SubagentStart, SubagentStop, PreCompact, PostCompact, Setup, Elicitation, ElicitationResult, ConfigChange, WorktreeCreate, WorktreeRemove, InstructionsLoaded, FileChanged, CwdChanged, TeammateIdle, TaskCompleted | BeforeTool, AfterTool, BeforeAgent, AfterAgent, BeforeModel, AfterModel, BeforeToolSelection, Notification, SessionStart, SessionEnd, PreCompress | PreToolUse, PermissionRequest, PostToolUse, PreCompact, PostCompact, SessionStart, UserPromptSubmit, SubagentStart, SubagentStop, Stop |
| No session-end event | | | true: Codex has no SessionEnd; the stem uses Stop |
| Deny a tool call | stdout `{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"…"}}`, exit 2 with the reason on stderr | stdout `{"decision":"deny","reason":"…"}` | as Claude |
| Refuse a stop | stdout `{"decision":"block","reason":"…"}`, exit 2 with the reason on stderr | on AfterAgent: `{"decision":"block","reason":"…"}` | as Claude |
| Payload fields | `session_id`, `cwd`, `hook_event_name`, `transcript_path`, `permission_mode`, `tool_name`, `tool_input`, `tool_use_id`, `tool_response`, `error` (PostToolUseFailure), `stop_hook_active`, `trigger`, `source`, `reason`, `model`, `agent_id` | `session_id`, `cwd`, `hook_event_name`, `transcript_path`, `tool_name`, `tool_input`, `tool_response`, `llm_request`, `llm_response`, `trigger`, `source`, `reason` | `session_id`, `cwd`, `hook_event_name`, `transcript_path`, `permission_mode`, `turn_id`, `model`, `tool_name`, `tool_input`, `tool_use_id`, `tool_response`, `trigger`, `source`, `prompt` |
| Project-scope install | `claude plugin install <name>@<marketplace> --scope project` from a local marketplace directory | `gemini extensions install <path>` (user scope only; `gemini extensions validate <path>` checks a bundle) | `codex plugin marketplace add <dir>` (a directory holding `marketplace.json` with `plugins: [{name, source: {source: "local", path: "./plugins/<name>"}}]`; the personal one is `<home>/.agents/plugins/marketplace.json`) then `codex plugin add <name>`; hooks load only when the project is trusted (`[projects."<abs path>"] trust_level = "trusted"` in `<home>/.codex/config.toml`) |
| Budget | Stop and SessionEnd hooks share 1.5 s | | |

Added 2026-09-09, verified against the installed Codex 0.133.0 binary: every model profile it
ships sets `"apply_patch_tool_type": "freeform"` (six of six), so Codex's `apply_patch` reaches
a hook as patch text and not as an object with a field the stem can name. The text may be the
whole `tool_input`, or sit under `input`, or under `patch`, or inside a `command` array. What
the vendor's own grammar does fix is the patch: its file operations are `*** Add File:`,
`*** Update File:` and `*** Delete File:` headers inside a `*** Begin Patch` / `*** End Patch`
envelope. A rule that has to know which file an `apply_patch` touches reads those headers out
of the strings in the input, and does not bet on the shape around them.

## 6. The contracts

`contracts/manifest.schema.json` and `contracts/lock.schema.json` are the contracts' product,
at v1.2.0 (0ddfafa). The stem reads them, never owns them: embedded with
`include_str!`, enforced before parsing, and their fixtures are the stem's test corpus.
A gap the stem finds in a schema is reported to the umbrella agent (cape-town, on pollen),
never patched around in the stem; the four gaps reported on 2026-09-08 (the MCP launch line,
the commit pin for git installs, the binary name, the skill path inside the artifact) landed
as v1.2.0 the same evening.

## 7. Fit evidence

`scripts/fit/stem.sh <check>` is the evidence the loop names. It builds the crate in release
mode and runs one check per subcommand, exit 0 on pass. Each subcommand simulates a clean
machine with a temporary `HOME` and `CODEX_HOME` so nothing on the build machine's user
scope is touched. The checks a node adds are named in its prompt; a check the node cannot
close honestly is left absent from the script, never written to exit 0.
