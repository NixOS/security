#!/usr/bin/env bash
#!/usr/bin/env nix-shell
#!nix-shell --pure -i bash -p git

set -eu
set -o pipefail

TMPDIR="/tmp"
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

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

commits() {
    local cache
    local branchname
    local range
    branchname="$1"
    range="$2"
    cache="$TMPTO/${branchname}_commits"
    if [ ! -f "$cache" ]; then
        git rev-list "${range}" | tee "$cache"
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

commits_with_notes() {
    local cache
    local branchname
    local range
    branchname="$1"
    range="$2"

    cache="$TMPTO/${branchname}_commits_with_notes"
    if [ ! -f "$cache" ]; then
        local commit
        (for commit in $(commits "$branchname" "$range"); do
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

The following issues have been resolved in NixOS in release-16.09,
release-17.03, and unstable. They remain potentially vulnerable on
older major releases.

These patches will be released to the release-16.09, release-17.03,
and unstable channels when Hydra finishes building the "tested" job
for each channel:

 - https://hydra.nixos.org/job/nixos/release-16.09/tested
 - https://hydra.nixos.org/job/nixos/release-17.03/tested
 - https://hydra.nixos.org/job/nixos/trunk-combined/tested

Currently, 17.03 is considered beta. It will be released around the
end of March. NixOS typically only supports one release at a time.
This means when 17.03 is released you should upgrade as soon as
possible. To ease this transition, I've decided to extend 16.09
security patches for one month after 17.03 is released.

Please consider helping with the next security roundup by commenting on
LATEST_ROUNDUP_URL.

EOF


changes_for() {
    local branch
    local range
    branch="$1"
    range="$2"

    echo "The following changes were applied to ${branch}"

    (for commit in $(commits_with_notes "$branch" "$range"); do
         log_commit "$commit"
     done) | cat
}

separator() {
    echo "======================================================================"
    echo ""
    echo ""
    echo ""
}

changes_for "release-16.09" "$RELEASE_16_09_SENT..origin/release-16.09"

separator

changes_for "unstable" "$MASTER_SENT..origin/master"

cat <<EOF

Thank you very much,
Graham Christensen
NixOS Security Team
https://github.com/nixos/security
EOF

update_state() {
    echo "RELEASE_16_09_SENT=$(git rev-parse "origin/release-16.09")"
    echo "MASTER_SENT=$(git rev-parse "origin/master")"
}

update_state > "$DIR/state/port_state.sh"
