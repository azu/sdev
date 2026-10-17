#!/bin/sh
# Run from the project root. See the project-bootstrap specification for the trust boundary.
set -eu

fail() { printf 'sdev: %s\n' "$*" >&2; exit 1; }
usage() {
    cat <<'EOF'
Usage: sh install.sh [--version X.Y.Z] [--verify] [--source-commit SHA] [--trusted-root FILE] [--init]

Install a public release into ./bin/sdev without changing PATH.
The default version is the latest stable release. SHA-256 is always checked.
--verify requires gh and verifies the binary's GitHub Artifact Attestation before execution.
--source-commit pins an independently checked source commit (requires --verify).
--trusted-root supplies independently obtained trust material for offline verification.
--init also adds project instructions and gitignore entries with ./bin/sdev init.
Without --init, AGENTS.md and .gitignore are not changed.
Trust this installer before running it; --verify does not verify the running script itself.
EOF
}
main() {
    version=
    verify=false
    initialize=false
    trusted_root=
    expected_commit=
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --version) [ "$#" -ge 2 ] || fail '--version needs X.Y.Z'; version=$2; shift 2 ;;
            --verify) verify=true; shift ;;
            --source-commit) [ "$#" -ge 2 ] || fail '--source-commit needs a SHA'; expected_commit=$2; shift 2 ;;
            --trusted-root) [ "$#" -ge 2 ] || fail '--trusted-root needs a file'; trusted_root=$2; shift 2 ;;
            --init) initialize=true; shift ;;
            --help|-h) usage; return ;;
            *) fail "Unknown argument: $1" ;;
        esac
    done
    if [ -n "$expected_commit" ]; then
        [ "$verify" = true ] || fail '--source-commit requires --verify'
        [ "${#expected_commit}" -eq 40 ] || fail 'Source commit must be a 40-character lowercase SHA'
        case "$expected_commit" in *[!0-9a-f]*) fail 'Source commit must be a 40-character lowercase SHA' ;; esac
    fi
    if [ -n "$trusted_root" ]; then
        [ "$verify" = true ] || fail '--trusted-root requires --verify'
        [ -f "$trusted_root" ] || fail 'Trusted root file not found'
    fi
    if [ "$verify" = true ]; then check_verifier; fi
    command -v curl >/dev/null 2>&1 || fail 'curl is required'
    if command -v sha256sum >/dev/null 2>&1; then
        hash_command=sha256sum
    elif command -v shasum >/dev/null 2>&1; then
        hash_command=shasum
    else
        fail 'sha256sum or shasum is required'
    fi
    case "$(uname -s)" in
        Darwin) platform=macos ;;
        Linux) platform=linux ;;
        *) fail 'This installer supports macOS and Linux only' ;;
    esac
    case "$(uname -m)" in
        arm64|aarch64) arch=arm64 ;;
        x86_64|amd64) arch=x64 ;;
        *) fail 'Unsupported CPU architecture' ;;
    esac
    releases=https://github.com/azu/sdev/releases
    if [ -z "$version" ]; then
        latest=$(curl -q -fsSL --proto '=https' --proto-redir '=https' --tlsv1.2 --connect-timeout 15 --max-time 120 \
            -o /dev/null -w '%{url_effective}' "$releases/latest") || fail 'Cannot resolve latest release'
        case "$latest" in
            "$releases/tag/v"*) version=${latest#"$releases/tag/v"} ;;
            *) fail 'No public stable release found; specify --version X.Y.Z' ;;
        esac
    fi
    # Only the stable release format accepted by the publishing workflow is allowed.
    case "$version" in ''|*[!0-9.]*|.*|*.) fail 'Version must be X.Y.Z without v' ;; esac
    old_ifs=$IFS
    IFS=.
    # Version was restricted to digits and dots; split its three numeric components.
    # shellcheck disable=SC2086
    set -- $version
    IFS=$old_ifs
    [ "$#" -eq 3 ] || fail 'Version must be X.Y.Z without v'
    for part do
        case "$part" in ''|0[0-9]*) fail 'Invalid version component' ;; esac
    done
    check_destination
    mkdir -p ./bin
    temporary=$(mktemp -d ./bin/.sdev-install.XXXXXXXX) || fail 'Cannot create temporary directory'
    trap 'rm -rf "$temporary"' EXIT
    trap 'exit 1' HUP INT TERM
    asset=sdev-v$version-$platform-$arch
    base=$releases/download/v$version
    fetch "$base/$asset" "$temporary/$asset"
    fetch "$base/$asset.sha256" "$temporary/checksum"
    if [ "$hash_command" = sha256sum ]; then
        actual=$(sha256sum "$temporary/$asset")
    else
        actual=$(shasum -a 256 "$temporary/$asset")
    fi
    actual=${actual%% *}
    expected=$(cat "$temporary/checksum")
    [ "$expected" = "$actual  $asset" ] || fail 'SHA-256 checksum mismatch'
    if [ "$verify" = true ]; then
        fetch "$base/attestation.sigstore.json" "$temporary/attestation.sigstore.json"
        fetch "$base/sdev-release.txt" "$temporary/sdev-release.txt"
        commit=
        verify_file "$temporary/sdev-release.txt"
        # Read only data from an already authenticated manifest; never source it as shell code.
        manifest_version=$(sed -n 's/^version=//p' "$temporary/sdev-release.txt")
        commit=$(sed -n 's/^source_commit=//p' "$temporary/sdev-release.txt")
        [ "$manifest_version" = "$version" ] || fail 'Attested release version does not match'
        [ "${#commit}" -eq 40 ] || fail 'Invalid source commit in attested release manifest'
        case "$commit" in *[!0-9a-f]*) fail 'Invalid source commit in attested release manifest' ;; esac
        if [ -n "$expected_commit" ]; then
            [ "$commit" = "$expected_commit" ] || fail 'Attested source commit does not match the pinned commit'
        fi
        verify_file "$temporary/sdev-release.txt"
        verify_file "$temporary/$asset"
    else
        printf '%s\n' 'sdev: SHA-256 checked; provenance NOT verified (use --verify to check it).' >&2
    fi
    chmod +x "$temporary/$asset"
    actual_version=$("$temporary/$asset" --version) || fail 'Downloaded binary could not start'
    [ "$actual_version" = "sdev $version" ] || fail 'Downloaded binary version does not match'
    check_destination
    mv -f "$temporary/$asset" ./bin/sdev
    printf 'Installed sdev %s into ./bin/sdev\n' "$version"
    if [ "$initialize" = true ]; then
        ./bin/sdev init
    else
        printf '%s\n' 'Run ./bin/sdev help, or ./bin/sdev init to add project instructions.'
    fi
}
check_verifier() {
    command -v gh >/dev/null 2>&1 || fail '--verify requires GitHub CLI (gh)'
    # Older versions may leak credentials while fetching trust material (GHSA-8xvp-7hj6-mcj9).
    gh --version | awk 'NR == 1 {
        if ($3 !~ /^[0-9]+\.[0-9]+\.[0-9]+$/) exit 1
        split($3, v, "."); exit !(v[1] > 2 || (v[1] == 2 && v[2] >= 93))
    } END { if (NR == 0) exit 1 }' || fail '--verify requires gh 2.93.0 or newer'
}
verify_file() {
    set -- attestation verify "$1" --hostname github.com --repo azu/sdev \
        --bundle "$temporary/attestation.sigstore.json" \
        --signer-workflow azu/sdev/.github/workflows/build.yml \
        --source-ref refs/heads/main --deny-self-hosted-runners
    if [ -n "$commit" ]; then
        set -- "$@" --source-digest "$commit" --signer-digest "$commit"
    fi
    if [ -n "$trusted_root" ]; then
        set -- "$@" --custom-trusted-root "$trusted_root"
    fi
    gh "$@" || fail 'Attestation verification failed; nothing installed'
}
fetch() {
    curl -q -fsSL --proto '=https' --proto-redir '=https' --tlsv1.2 --connect-timeout 15 --max-time 120 -o "$2" "$1" || fail "Download failed: $1"
}
check_destination() {
    [ ! -L ./bin ] || fail './bin must not be a symbolic link'
    if [ -e ./bin ]; then
        [ -d ./bin ] || fail './bin must be a directory'
    fi
    [ ! -L ./bin/sdev ] || fail './bin/sdev must not be a symbolic link'
    if [ -e ./bin/sdev ]; then
        [ -f ./bin/sdev ] || fail './bin/sdev must be a regular file'
    fi
}
# A complete function definition is read before it runs, including when piped to sh.
main "$@"
