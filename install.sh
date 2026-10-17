#!/bin/sh
# Run from the project root. See the project-bootstrap specification for the trust boundary.
set -eu

fail() { printf 'sdev: %s\n' "$*" >&2; exit 1; }
usage() {
    cat <<'EOF'
Usage: sh install.sh [--version X.Y.Z] [--install-dir DIR] [--verify] [--source-commit SHA] [--trusted-root FILE] [--init]
       sh install.sh --update [--path PATH] [--version X.Y.Z] [--verify] [--source-commit SHA] [--trusted-root FILE]

Install a public release into ./bin/sdev without changing PATH.
--install-dir chooses the directory for the binary.
The default version is the latest stable release. SHA-256 is always checked.
--verify requires gh and verifies the binary's GitHub Artifact Attestation before execution.
--source-commit pins an independently checked source commit (requires --verify).
--trusted-root supplies independently obtained trust material for offline verification.
--init runs the installed binary's init command to prepare project instructions and its launcher.
An existing generated launcher is updated in place, preserving project documents, even with --init.
--update requires an existing project launcher; --path chooses its relative location (default bin/sdev).
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
    install_dir=
    updating=false
    update_path=
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --version) [ "$#" -ge 2 ] || fail '--version needs X.Y.Z'; version=$2; shift 2 ;;
            --install-dir) [ "$#" -ge 2 ] || fail '--install-dir needs a directory'; [ -n "$2" ] || fail '--install-dir needs a directory'; install_dir=$2; shift 2 ;;
            --verify) verify=true; shift ;;
            --source-commit) [ "$#" -ge 2 ] || fail '--source-commit needs a SHA'; expected_commit=$2; shift 2 ;;
            --trusted-root) [ "$#" -ge 2 ] || fail '--trusted-root needs a file'; trusted_root=$2; shift 2 ;;
            --init) initialize=true; shift ;;
            --update) updating=true; shift ;;
            --path) [ "$#" -ge 2 ] || fail '--path needs a project-relative path'; [ -n "$2" ] || fail '--path needs a project-relative path'; update_path=$2; shift 2 ;;
            --help|-h) usage; return ;;
            *) fail "Unknown argument: $1" ;;
        esac
    done
    if [ -n "$update_path" ]; then
        [ "$updating" = true ] || fail '--path requires --update'
        [ -z "$install_dir" ] || fail '--path cannot be combined with --install-dir'
    fi
    if [ "$updating" = true ]; then
        [ -z "$install_dir" ] || fail '--update cannot be combined with --install-dir'
    fi
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
    if [ -t 2 ]; then curl_progress=--progress-bar; else curl_progress=--silent; fi
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
        printf '%s\n' 'sdev: Resolving the latest release...' >&2
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
    if [ -z "$install_dir" ]; then
        install_dir=./bin
    fi
    # Prefix relative paths so that a name starting with '-' is never an option.
    case "$install_dir" in /*|./*|../*) ;; *) install_dir=./$install_dir ;; esac
    install_dir=${install_dir%/}
    [ -n "$install_dir" ] || install_dir=/
    if [ "$updating" = true ]; then
        update_project "${update_path:-bin/sdev}"
        return
    fi
    check_directory "$install_dir"
    [ ! -L "$install_dir/sdev" ] || fail "$install_dir/sdev must not be a symbolic link"
    if [ -f "$install_dir/sdev" ] && head -c 64 "$install_dir/sdev" | LC_ALL=C grep -q '# sdev bootstrap'; then
        update_project "$install_dir/sdev"
        return
    fi
    check_destination
    mkdir -p "$install_dir" || fail 'Cannot create installation directory'
    temporary=$(mktemp -d "$install_dir/.sdev-install.XXXXXXXX") || fail 'Cannot create temporary directory'
    trap 'rm -rf "$temporary"' EXIT
    trap 'exit 1' HUP INT TERM
    asset=sdev-v$version-$platform-$arch
    base=$releases/download/v$version
    printf 'sdev: Installing %s into %s/sdev\n' "$asset" "$install_dir" >&2
    fetch "$base/$asset" "$temporary/$asset"
    fetch "$base/$asset.sha256" "$temporary/checksum"
    printf '%s\n' 'sdev: Checking SHA-256...' >&2
    if [ "$hash_command" = sha256sum ]; then
        actual=$(sha256sum "$temporary/$asset")
    else
        actual=$(shasum -a 256 "$temporary/$asset")
    fi
    actual=${actual%% *}
    expected=$(cat "$temporary/checksum")
    [ "$expected" = "$actual  $asset" ] || fail 'SHA-256 checksum mismatch'
    if [ "$verify" = true ]; then
        printf '%s\n' 'sdev: Verifying provenance...' >&2
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
    chmod +x "$temporary/$asset" || fail 'Cannot make downloaded executable runnable'
    actual_version=$("$temporary/$asset" --version) || fail 'Downloaded binary could not start'
    [ "$actual_version" = "sdev $version" ] || fail 'Downloaded binary version does not match'
    check_destination
    mv -f "$temporary/$asset" "$install_dir/sdev" || fail 'Cannot replace installed executable'
    printf 'Installed sdev %s into %s/sdev\n' "$version" "$install_dir"
    if [ "$initialize" = true ]; then
        "$install_dir/sdev" init
    else
        printf 'Run %s/sdev help, or %s/sdev init to add project instructions.\n' "$install_dir" "$install_dir"
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
    printf 'sdev: Downloading %s...\n' "${1##*/}" >&2
    curl -q -fL "$curl_progress" --show-error --proto '=https' --proto-redir '=https' --tlsv1.2 --connect-timeout 15 --max-time 120 -o "$2" "$1" || fail "Download failed: $1"
}
check_directory() {
    [ ! -L "$1" ] || fail "$1 must not be a symbolic link"
    if [ -e "$1" ]; then
        [ -d "$1" ] || fail "$1 must be a directory"
    fi
    case "$1" in ''|/|.) return ;; esac
    case "$1" in */*) check_directory "${1%/*}" ;; esac
}
check_destination() {
    check_directory "$install_dir"
    [ ! -L "$install_dir/sdev" ] || fail "$install_dir/sdev must not be a symbolic link"
    if [ -e "$install_dir/sdev" ]; then
        [ -f "$install_dir/sdev" ] || fail "$install_dir/sdev must be a regular file"
        if head -c 64 "$install_dir/sdev" | LC_ALL=C grep -q '# sdev bootstrap'; then
            fail 'Destination became a launcher; rerun installation or use sdev update'
        fi
    fi
}
update_project() {
    update_root=$(pwd -P)
    update_relative=$1
    case "$update_relative" in
        "$update_root"/*) update_relative=${update_relative#"$update_root"/} ;;
        ./*) update_relative=${update_relative#./} ;;
    esac
    case "$update_relative" in
        ''|/*|*/|*//*|*[!a-zA-Z0-9._/-]*) fail 'Update path must be a relative project launcher path' ;;
        .sdev|.sdev/*|.git|.git/*|AGENTS.md|AGENTS.md/*|.gitignore|.gitignore/*|sdev.toml|sdev.toml/*) fail 'Update path is reserved' ;;
    esac
    case "/$update_relative/" in */./*|*/../*) fail 'Update path must not contain . or .. components' ;; esac
    update_target=$update_root/$update_relative
    update_parent=${update_target%/*}
    check_directory "$update_parent"
    if [ -L "$update_target" ] || [ ! -f "$update_target" ]; then
        fail 'Update requires an existing regular generated launcher'
    fi
    update_old_version=$(sed -n '2s/^# sdev bootstrap — generated by sdev \([0-9][0-9.]*\); commit this file\.$/\1/p' "$update_target")
    [ -n "$update_old_version" ] || fail 'Update requires a generated sdev launcher, not another file'
    check_directory "$update_root/.sdev"
    mkdir -p "$update_root/.sdev"
    update_lock=$update_root/.sdev/update.lock
    mkdir "$update_lock" 2>/dev/null || fail 'Update is locked: inspect .sdev/update.lock for another update or a lock left after termination'
    update_temporary=
    update_neighbor=
    trap 'if [ -n "$update_neighbor" ]; then rm -f "$update_neighbor"; fi; if [ -n "$update_temporary" ]; then rm -rf "$update_temporary"; fi; rmdir "$update_lock" 2>/dev/null || :' EXIT
    trap 'exit 1' HUP INT TERM
    update_temporary=$(mktemp -d "$update_root/.sdev/.sdev-update.XXXXXXXX") || fail 'Cannot prepare update directory'
    cp -p "$update_target" "$update_temporary/original"
    printf 'sdev: Checking the original %s launcher...\n' "$update_old_version" >&2
    set -- --version "$update_old_version" --install-dir "$update_temporary/old"
    if [ "$verify" = true ]; then set -- "$@" --verify; fi
    if [ -n "$trusted_root" ]; then set -- "$@" --trusted-root "$trusted_root"; fi
    ( main "$@" ) >/dev/null || fail 'Could not verify the original launcher version; nothing updated'
    mkdir "$update_temporary/old-project"
    (
        cd "$update_temporary/old-project"
        "$update_temporary/old/sdev" init --path "$update_relative"
    ) >/dev/null || fail 'Could not reproduce the original launcher; nothing updated'
    cmp -s "$update_temporary/original" "$update_temporary/old-project/$update_relative" || fail 'Launcher has been edited; review it before updating'

    set -- --version "$version" --install-dir "$update_temporary/new"
    if [ "$verify" = true ]; then set -- "$@" --verify; fi
    if [ -n "$trusted_root" ]; then set -- "$@" --trusted-root "$trusted_root"; fi
    if [ -n "$expected_commit" ]; then set -- "$@" --source-commit "$expected_commit"; fi
    ( main "$@" ) >/dev/null || fail 'Could not verify the new release; nothing updated'
    mkdir "$update_temporary/new-project"
    (
        cd "$update_temporary/new-project"
        "$update_temporary/new/sdev" init --path "$update_relative"
    ) >/dev/null || fail 'Could not prepare the new launcher; nothing updated'
    update_generated=$update_temporary/new-project/$update_relative
    if [ -L "$update_generated" ] || [ ! -f "$update_generated" ]; then
        fail 'Selected release does not generate a project launcher'
    fi
    [ "$(sed -n '2p' "$update_generated")" = "# sdev bootstrap — generated by sdev $version; commit this file." ] || fail 'Generated launcher version does not match'
    check_directory "$update_parent"
    if [ -L "$update_target" ] || [ ! -f "$update_target" ]; then
        fail 'Launcher file type changed during update'
    fi
    cmp -s "$update_temporary/original" "$update_target" || fail 'Launcher changed during update; nothing replaced'
    if [ "$version" = "$update_old_version" ]; then
        if [ "${SDEV_UPDATE_LANGUAGE:-en}" = ja ]; then
            printf 'sdev %sはすでに./%sで選択されています。\n' "$version" "$update_relative"
        else
            printf 'sdev %s is already selected by ./%s.\n' "$version" "$update_relative"
        fi
        return
    fi
    update_cache=$update_root/.sdev/bin/$version/$platform-$arch
    check_directory "$update_cache"
    [ ! -L "$update_cache/sdev" ] || fail 'Cached executable must not be a symbolic link'
    if [ -e "$update_cache/sdev" ]; then [ -f "$update_cache/sdev" ] || fail 'Cached executable must be a regular file'; fi
    mkdir -p "$update_cache"
    update_neighbor=$(mktemp "$update_parent/.sdev-update.XXXXXXXX") || fail 'Cannot prepare adjacent launcher file'
    cp -p "$update_temporary/original" "$update_neighbor"
    case "$platform" in
        macos) update_mode=$(stat -f '%Lp' "$update_temporary/original") || fail 'Cannot read launcher permissions' ;;
        linux) update_mode=$(stat -c '%a' "$update_temporary/original") || fail 'Cannot read launcher permissions' ;;
    esac
    case "$update_mode" in ''|*[!0-7]*) fail 'Cannot read launcher permissions' ;; esac
    chmod u+w "$update_neighbor"
    cat "$update_generated" > "$update_neighbor"
    chmod "$update_mode" "$update_neighbor"
    check_directory "$update_cache"
    [ ! -L "$update_cache/sdev" ] || fail 'Cached executable changed to a symbolic link'
    mv -f "$update_temporary/new/sdev" "$update_cache/sdev"
    check_directory "$update_parent"
    if [ -L "$update_target" ] || [ ! -f "$update_target" ]; then
        fail 'Launcher file type changed during update'
    fi
    cmp -s "$update_temporary/original" "$update_target" || fail 'Launcher changed during update; nothing replaced'
    mv -f "$update_neighbor" "$update_target"
    update_neighbor=
    if [ "${SDEV_UPDATE_LANGUAGE:-en}" = ja ]; then
        printf 'sdev %sから%sへ./%sを更新しました。\n差分を確認し、起動スクリプトをコミットしてください。\n' "$update_old_version" "$version" "$update_relative"
    else
        printf 'Updated sdev %s -> %s at ./%s.\nReview and commit the updated launcher.\n' "$update_old_version" "$version" "$update_relative"
    fi
}
# A complete function definition is read before it runs, including when piped to sh.
main "$@"
