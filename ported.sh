#!/usr/bin/env nix-shell
#!nix-shell -i bash -p git

set -eu

REMOTE="origin"
UPSTREAM="master"
TO="release-16.09"

readonly TMPTO=$(mktemp -d ported.XXXXXXXXXX -p "$TMPDIR")
cleanup() {
    rm -rf "$TMPTO"
}
trap cleanup EXIT


git fetch "$REMOTE"
git cherry "$REMOTE/$TO" "$REMOTE/$UPSTREAM" $(git rev-parse "$REMOTE/$UPSTREAM~200") \
    | grep -v "+" \
    | cut -d' ' -f2 \
    | xargs git show \
    | git patch-id --stable > $TMPTO/upstream

git log --pretty=format:%H "$REMOTE/$TO"~200..."$REMOTE/$TO" \
    | xargs git show \
    | git patch-id --stable > $TMPTO/against

cat <<EOF
### Security fixes from $(date -u "+%F %R %Z")

The following issues have been resolved in NixOS in unstable and
$TO. They remain potentially vulnerable on older major releases.

These patches will be released to the unstable and $TO channels when
Hydra finishes building the "tested" job for each channel:

 - https://hydra.nixos.org/job/nixos/${TO}/tested
 - https://hydra.nixos.org/job/nixos/trunk-combined/tested

EOF
echo "$UPSTREAM|$TO|Message|Notes"
echo "---|---|---|---"
for id in `cat $TMPTO/upstream | cut -d' ' -f1`; do
    master=$(cat $TMPTO/upstream | grep -G "^${id}" | cut -d' ' -f2)
    release=$(cat $TMPTO/against | grep -G "^${id}" | cut -d' ' -f2)
    message=$(git log --pretty=format:%s $master...$master~1)
    echo "$master|$release|$message|n/a"
done
