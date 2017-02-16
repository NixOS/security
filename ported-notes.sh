#!/usr/bin/env bash
#!/usr/bin/env nix-shell
#!nix-shell --pure -i bash -p git

set -eu
set -o pipefail

TMPDIR="/tmp"
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

REMOTE="origin"
UPSTREAM="master"
TO_SHORT="16.09"
TO="release-${TO_SHORT}"

source "$DIR/state/port_state.sh"

readonly TMPTO=$(mktemp -d ported.XXXXXXXXXX -p "$TMPDIR")
cleanup() {
    rm -rf "$TMPTO"
}
trap cleanup EXIT

debug() {
    set +u
    if ! [ "x$DEBUG" = "x" ]; then
        echo "$@" >&2
    fi
    set -u
}

unstable_commits() {
    local cache
    cache="$TMPTO/unstable_commits"
    if [ ! -f "$cache" ]; then
        git rev-list "$UPSTREAM_OLDEST..$REMOTE/$UPSTREAM" | tee "$cache"
    else
        cat "$cache"
    fi
}

stable_commits() {
    local cache
    cache="$TMPTO/stable_commits"
    if [ ! -f "$cache" ]; then
        git rev-list "$TO_OLDEST..$REMOTE/$TO" | tee "$cache"
    else
        cat "$cache"
    fi
}

commit_has_note() {
    local commit
    commit="$1"

    if git notes --ref=security show "$commit" > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}


unstable_commits_with_notes() {
    local cache
    cache="$TMPTO/unstable_commits_with_notes"
    if [ ! -f "$cache" ]; then
        local commit
        (for commit in $(unstable_commits); do
             if commit_has_note "$commit"; then
                 echo "$commit"
             fi
        done) | tee "$cache"
    else
        cat "$cache"
    fi
}

stable_commits_with_notes() {
    local cache
    cache="$TMPTO/stable_commits_with_notes"
    if [ ! -f "$cache" ]; then
        local commit
        (for commit in $(stable_commits); do
             if commit_has_note "$commit"; then
                 echo "$commit"
             fi
        done) | tee "$cache"
    else
        cat "$cache"
    fi
}

log_commit() {
    local commit

    commit="$1"

    author=$(git show --no-patch --notes=security --pretty="format:%an" "${commit}")
    committer=$(git show --no-patch --notes=security --pretty="format:%cn" "${commit}")

    dontthank="Graham Christensen"
    if [ "x$author" = "x$committer" ]; then
        if [ "x$author" = "x$dontthank" ]; then
            thanks="";
        else
            thanks="(Thank you, $author)"
        fi
    elif [ "x$author" = "x$dontthank" ]; then
        thanks="(Thank you, ${committer} (committer))"
    elif [ "x$committer" = "x$dontthank" ]; then
        thanks="(Thank you, ${author} (author))"
    else
        thanks="(Thank you: ${author} (author), ${committer} (committer))"
    fi

    git show --no-patch --notes=security --pretty="format:
%h  %<(60,trunc)%s
" "${commit}"
    if [ "x$thanks" != "x" ]; then
        echo "$thanks"
    fi
    git show --no-patch --notes=security \
        --pretty="format:%N" "${commit}" \
        | sed -e 's/^/> /'

}

cat <<EOF
From: Graham Christensen <graham@grahamc.com>
To: nix-security-announce@googlegroups.com
Subject: Security fixes from $(date -u "+%F %R %Z")
--text follows this line--
<#secure method=pgp mode=sign>

The following issues have been resolved in NixOS in $TO and
unstable. They remain potentially vulnerable on older major
releases.

These patches will be released to the unstable and
$TO channels when Hydra finishes building the "tested" job
for each channel:

 - https://hydra.nixos.org/job/nixos/${TO}/tested
 - https://hydra.nixos.org/job/nixos/trunk-combined/tested

Please consider helping with the next security roundup by commenting on
LATEST_ROUNDUP_URL.

EOF

echo "The following changes were applied to release-16.09:"

(for commit in $(stable_commits_with_notes); do
    log_commit "$commit"
done) | cat

echo "======================================================================"
echo ""
echo ""
echo ""
echo "The following changes were applied to unstable:"

(for commit in $(unstable_commits_with_notes); do
    log_commit "$commit"
 done) | cat

cat <<EOF

Thank you very much,
Graham Christensen
NixOS Security Team
https://github.com/nixos/security
EOF

update_state() {
    echo "TO_OLDEST=$(git rev-parse "$REMOTE/$TO")"
    echo "UPSTREAM_OLDEST=$(git rev-parse "$REMOTE/$UPSTREAM")"
}

update_state > "$DIR/state/port_state.sh"
