#!/usr/bin/env bash
# scripts/fit/lib.sh: the fit runner.
#
# Resolves a bed's pinned version and artifact from a garden.lock, fetches the artifact
# into a fresh temp directory, verifies its sha256, refuses to run anything on a mismatch,
# and runs a named check with a PATH that holds only that fetched directory (plus the
# minimal system directories a shell needs to function), never whatever other garden
# tools happen to be installed on the caller's machine.
#
# Meant to be sourced by a bed-specific `scripts/fit/<bed>.sh` or by a test. It can also be
# invoked directly:
#   scripts/fit/lib.sh run <lockfile> <judge> <platform> <bin-name> -- <check-args...>
#
# Functions:
#   fit_repo_root                                   prints this repository's root
#   fit_platform                                    prints this machine's platform triple
#   fit_lock_lookup <lockfile> <judge> [platform]   prints the resolved judge as one JSON line
#   fit_fetch <url> <dest-file>                     downloads url (http(s):// or file://) to dest-file
#   fit_verify_sha256 <file> <expected-sha256>      0 if the file's sha256 matches, 1 otherwise
#   fit_extract <archive> <dest-dir>                extracts a .tar.gz or .zip, or copies a bare file
#   fit_clean_path <bin-dir>                        prints a PATH containing only bin-dir and the
#                                                    minimal system directories, never a garden tool
#   fit_run <lockfile> <judge> <platform> <bin-name> -- <check-args...>
#                                                    resolve, fetch, verify, run; non-zero on any failure

set -u

# Captured once, at the moment this file itself is sourced or executed, when
# BASH_SOURCE[0] reliably names this file. A function body cannot rely on the same
# expansion later: once control returns to a caller with no source file of its own (a
# `bash -c` snippet, an interactive shell), BASH_SOURCE is empty in that call frame and
# the array reference fails under `set -u`.
FIT_LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

fit_repo_root() {
  ( cd "$FIT_LIB_DIR/../.." && pwd )
}

fit_platform() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os" in
    Darwin)
      case "$arch" in
        arm64) echo "aarch64-apple-darwin" ;;
        x86_64) echo "x86_64-apple-darwin" ;;
        *) echo "unknown-apple-darwin" ;;
      esac
      ;;
    Linux)
      case "$arch" in
        x86_64) echo "x86_64-unknown-linux-musl" ;;
        aarch64) echo "aarch64-unknown-linux-musl" ;;
        *) echo "unknown-unknown-linux-musl" ;;
      esac
      ;;
    *)
      echo "unknown-unknown-unknown"
      ;;
  esac
}

fit_lock_lookup() {
  local lockfile="$1" judge="$2" platform="${3:-}"
  local root
  root="$(fit_repo_root)"
  node "$root/scripts/fit/lock-lookup.mjs" "$lockfile" "$judge" "$platform"
}

fit_fetch() {
  local url="$1" dest="$2"
  case "$url" in
    file://*)
      local src="${url#file://}"
      cp "$src" "$dest"
      ;;
    https://*|http://*)
      curl -fsSL "$url" -o "$dest"
      ;;
    *)
      echo "fit_fetch: unsupported url scheme in '$url'" >&2
      return 1
      ;;
  esac
}

fit_sha256() {
  local file="$1"
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print $1}'
  else
    echo "fit_sha256: neither shasum nor sha256sum is available" >&2
    return 1
  fi
}

fit_verify_sha256() {
  local file="$1" expected="$2" actual
  actual="$(fit_sha256 "$file")" || return 1
  if [ "$actual" = "$expected" ]; then
    return 0
  fi
  echo "fit_verify_sha256: checksum mismatch for $file: expected $expected, got $actual" >&2
  return 1
}

fit_extract() {
  local archive="$1" dest_dir="$2"
  mkdir -p "$dest_dir"
  case "$archive" in
    *.tar.gz|*.tgz)
      tar -xzf "$archive" -C "$dest_dir"
      ;;
    *.zip)
      unzip -q "$archive" -d "$dest_dir"
      ;;
    *)
      cp "$archive" "$dest_dir/"
      ;;
  esac
}

fit_clean_path() {
  local bin_dir="$1"
  # Only the fetched artifact's own directory plus the minimal system directories a
  # subprocess needs to exist at all (a shell, coreutils). No user PATH entry survives,
  # so no other garden tool the caller happens to have installed is reachable.
  echo "$bin_dir:/usr/bin:/bin"
}

fit_run() {
  local lockfile="$1" judge="$2" platform="$3" bin_name="$4"
  shift 4
  if [ "${1:-}" = "--" ]; then shift; fi
  local check_args=("$@")

  local lookup
  lookup="$(fit_lock_lookup "$lockfile" "$judge" "$platform")" || {
    echo "fit_run: could not resolve $judge for $platform from $lockfile" >&2
    return 1
  }

  local url sha256
  url="$(echo "$lookup" | node -e 'process.stdout.write(JSON.parse(require("fs").readFileSync(0,"utf8")).url || "")')"
  sha256="$(echo "$lookup" | node -e 'process.stdout.write(JSON.parse(require("fs").readFileSync(0,"utf8")).sha256 || "")')"

  if [ -z "$url" ] || [ -z "$sha256" ]; then
    echo "fit_run: $judge has no fetchable artifact for $platform in $lockfile (npm-only judge?)" >&2
    return 1
  fi

  local tmp_dir
  tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/plotplot-fit.XXXXXX")" || return 1

  local ext
  case "$url" in
    *.tar.gz) ext="tar.gz" ;;
    *.tgz) ext="tgz" ;;
    *.zip) ext="zip" ;;
    *) ext="${url##*.}" ;;
  esac
  local archive="$tmp_dir/artifact.$ext"
  if ! fit_fetch "$url" "$archive"; then
    echo "fit_run: fetch failed for $url" >&2
    fit_discard_scratch "$tmp_dir"
    return 1
  fi

  if ! fit_verify_sha256 "$archive" "$sha256"; then
    echo "fit_run: refusing to run $judge, checksum mismatch" >&2
    fit_discard_scratch "$tmp_dir"
    return 1
  fi

  local bin_dir="$tmp_dir/bin"
  fit_extract "$archive" "$bin_dir"
  chmod +x "$bin_dir/$bin_name" 2>/dev/null || true

  local clean_path
  clean_path="$(fit_clean_path "$bin_dir")"

  # "${check_args[@]}" alone would trip `set -u` on an empty array under bash 3.2
  # (macOS's shipped /bin/bash), which treats that expansion as an unbound-variable
  # reference rather than zero words the way bash 4.4+ does. The
  # "${arr[@]+"${arr[@]}"}" form expands to nothing when the array is empty and to the
  # words themselves otherwise, on every bash this repository runs on.
  env -i PATH="$clean_path" HOME="$HOME" "$bin_dir/$bin_name" "${check_args[@]+"${check_args[@]}"}"
  local status=$?

  fit_discard_scratch "$tmp_dir"
  return $status
}

fit_discard_scratch() {
  # Puts away a directory this library created itself with mktemp: a freshly fetched,
  # checksum-verified artifact and its extracted bin/, never anything the caller owns. The
  # garden deletes with trash, never rm, and this script keeps that rule everywhere: where
  # trash exists the scratch goes to the Trash; where it does not (a CI runner, a proof-run
  # machine) the directory is left in the temp root for the OS to clear, and its path is
  # printed so nothing disappears silently. Never widen this function's use beyond a path
  # this library itself returned from mktemp.
  local scratch_dir="$1"
  case "$scratch_dir" in
    "${TMPDIR:-/tmp}"/plotplot-fit.*)
      if command -v trash >/dev/null 2>&1; then
        trash "$scratch_dir"
      else
        echo "fit_discard_scratch: no trash on this machine; leaving '$scratch_dir' for the temp root to clear" >&2
      fi
      ;;
    *) echo "fit_discard_scratch: refusing to touch '$scratch_dir', not a path this library minted" >&2 ;;
  esac
}

# Allow direct invocation: scripts/fit/lib.sh run <lockfile> <judge> <platform> <bin-name> -- <args...>
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
  cmd="${1:-}"
  case "$cmd" in
    run)
      shift
      fit_run "$@"
      exit $?
      ;;
    platform)
      fit_platform
      exit 0
      ;;
    lookup)
      shift
      fit_lock_lookup "$@"
      exit $?
      ;;
    *)
      echo "usage: lib.sh {run <lockfile> <judge> <platform> <bin-name> -- <args...> | platform | lookup <lockfile> <judge> [platform]}" >&2
      exit 2
      ;;
  esac
fi
