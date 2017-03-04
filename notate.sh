#!/usr/bin/env nix-shell
#!nix-shell -i bash -p git

set -eu
set -o pipefail

# Disable Ctrl-C since it can be very frustrating to lose progress
trap '' 2

readonly height=$(stty -a | grep rows | cut -d";" -f2 | cut -d' ' -f3)
readonly DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"


extract_cherrypick() {
    git show "$1" -- \
        | grep "(cherry picked from " \
        | sed -e "s/.*commit //" -e "s/)//"
}

mark_commit_ui() {
    sha="$1"
    clear
    set +e
    git show --notes=security --color=always "$sha" -- | head -n $((height - 8));
    echo "..."

    local picked_from;
    picked_from=$(extract_cherrypick "$sha")
    if [ "x$picked_from" != "x" ] && test -f "./$picked_from"; then
        echo "Found these notes from $picked_from:"
        cat "$picked_from"
        echo ""
        echo "Want to use them? [Y/n]: "

        read -r x;
        if [ "x$x" != "xn" ]; then
            cp "./$picked_from" "./$sha"
            git add "./$sha"
        fi
    fi

    echo -n "Does this need security notes or editing? [y/N or: kernel, browser]: ";
    read -r x;
    EDITED=0
    if [ "x$x" = "xy" ]; then
        $EDITOR "$sha"
        EDITED=1
    elif [ "x$x" = "xbrowser" ]; then
        echo "All browser patches are considered security-sensitive." >> "$sha"
        EDITED=1
    elif [ "x$x" = "xkernel" ]; then
        echo "All kernel patches are considered security-sensitive." >> "$sha"
        EDITED=1
    fi

    if [ $EDITED -eq 1 ]; then
        git add "./$sha"

        if [ "x$picked_from" != "x" ] && ! test -f "./$picked_from"; then
            echo "This commit was cherry-picked from $picked_from."
            echo "Backport your notes? [Y/n]:"

            read -r x;
            if [ "x$x" != "xn" ]; then
                cp "./$sha" "./$picked_from"
                git add "./$picked_from"
            fi
        fi
    fi
    set -e
}

git fetch origin
git fetch origin refs/notes/security:refs/notes/security
git checkout refs/notes/security
cleanup_basic() {
    git checkout -
}
trap cleanup_basic EXIT


if [ "x${1:-}" != "x" ]; then
    mark_commit_ui "$1"
else
    . "$DIR/state/notate_state.sh"

    next_MASTER_SEEN=$MASTER_SEEN
    next_RELEASE_16_09_SEEN=$RELEASE_16_09_SEEN
    cleanup() {
        echo "MASTER_SEEN=$next_MASTER_SEEN" > "$DIR/state/notate_state.sh"
        echo "RELEASE_16_09_SEEN=$next_RELEASE_16_09_SEEN" >> "$DIR/state/notate_state.sh"
        cleanup_basic
    }
    trap cleanup EXIT

    echo "Processing $MASTER_SEEN...origin/master"
    echo "Press enter to continue."
    read
    for sha in $(git rev-list --reverse --no-merges "$MASTER_SEEN...origin/master"); do
        mark_commit_ui "$sha"
        next_MASTER_SEEN="$sha"
    done

    echo "Processing $RELEASE_16_09_SEEN...origin/release-16.09"
    echo "Press enter to continue."
    read
    for sha in $(git rev-list --reverse --no-merges "$RELEASE_16_09_SEEN...origin/release-16.09"); do
        mark_commit_ui "$sha"
        next_RELEASE_16_09_SEEN="$sha"
    done

    echo "Going to commit these changes now"
fi

git commit
git update-ref refs/notes/security "$(git rev-parse HEAD)"
git push origin refs/notes/security:refs/notes/security
