# Subject: one product or project and its exact identifiers

What the garden listens about: a product, a project, a package, an app. A subject joins records across sources only by exact identifiers. A fuzzy match ("these two names are probably the same tool") is a judgment, and belongs to cultivate or glean.

Schema: `contracts/subject.schema.json`. Fixtures: `contracts/fixtures/subject/`. Test: `test/contracts.test.ts`.

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | sha256 of `subject`, a newline, and the first identifier written as `scheme:value`, as lowercase hex |
| `source` | yes | the adapter that read it, or `capture-plan` when a project names it |
| `kind` | yes | `product`, `project`, `package`, `repository`, `app`, `site` or `organization` |
| `name` | yes | the name the source gives it |
| `identifiers` | yes | at least one `{scheme, value}`; the first is the one the source knows it by |
| `fetched_at`, `private`, `raw_ref` | yes | as for every record |

## Identifier schemes

A value is written in its scheme's form before it is stored, so the same thing always matches.

| Scheme | Value |
|---|---|
| `domain` | the host name in lowercase, without a trailing dot and without a leading `www.` |
| `github` | `owner/repository` in lowercase |
| `npm` | the package name as registered, in lowercase, with its scope |
| `pypi` | the project name in lowercase, each run of `-`, `_` and `.` written as one `-` |
| `crates` | the crate name in lowercase |
| `app-store` | the numeric app id |
| `play-store` | the application id, such as `com.example.app` |
| `no-orgnr` | a Norwegian organisation number from the Brønnøysund registers: nine digits, no spaces |
| `gb-company` | a UK Companies House company number: eight characters, digits and capital letters, leading zeros kept |
| `url` | an absolute `https://` address, as written by the source |

A new scheme is a change to this contract.

## Rules a reader may rely on

- **Shared identifier, same subject.** Two subject records with any identifier in common are one subject. mull keeps the links; `mull query` answers with every id a subject is known by.
- **A subject is never a person.** People are counted through author hashes, never named as subjects.
