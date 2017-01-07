#!/usr/bin/env bash
#!/usr/bin/env nix-shell
#!nix-shell -i bash -p git -p ncurses

set -eu
set -o pipefail

readonly height=$(stty -a | grep rows | cut -d";" -f2 | cut -d' ' -f3)
readonly RELEASE_BRANCH=release-16.09
readonly DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

. "$DIR/state/notate_state.sh"

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

    echo -n "Does this need security notes or editing? [y/N]: ";
    read -r x;
    if [ "x$x" = "xy" ]; then
        $EDITOR "$sha"
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
git checkout refs/notes/security
cleanup_basic() {
    git checkout -
}
trap cleanup_basic EXIT


if [ "x${1:-}" != "x" ]; then
    mark_commit_ui "$1"
else
    cleanup() {
        echo "UPSTREAM_OLDEST=$next_UPSTREAM_OLDEST" > "$DIR/state/notate_state.sh"
        echo "TO_OLDEST=$next_TO_OLDEST" >> "$DIR/state/notate_state.sh"
        cleanup_basic
    }
    next_UPSTREAM_OLDEST=$UPSTREAM_OLDEST
    next_TO_OLDEST=$TO_OLDEST
    trap cleanup EXIT


    for sha in $(git rev-list --reverse --no-merges "$UPSTREAM_OLDEST...origin/master"); do
        mark_commit_ui "$sha"
        next_UPSTREAM_OLDEST="$sha"
    done

    for sha in $(git rev-list --reverse --no-merges "$TO_OLDEST...origin/$RELEASE_BRANCH"); do
        mark_commit_ui "$sha"
        next_TO_OLDEST="$sha"
    done

    echo "Going to commit these changes now"
fi

git commit
git update-ref refs/notes/security "$(git rev-parse HEAD)"
