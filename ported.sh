#!/usr/bin/env nix-shell
#!nix-shell -i bash -p git

set -eu

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

REMOTE="origin"
UPSTREAM="master"
TO_SHORT="16.09"
RELEASE_NAMED="release-${TO_SHORT}"

source "$DIR/state/port_state.sh"

readonly TMPTO=$(mktemp -d ported.XXXXXXXXXX -p "$TMPDIR")
cleanup() {
    rm -rf "$TMPTO"
}
trap cleanup EXIT


git fetch "$REMOTE"
git cherry "$REMOTE/$RELEASE_NAMED" "$REMOTE/$UPSTREAM" "$UPSTREAM_OLDEST" \
    | grep -v "+" \
    | cut -d' ' -f2 \
    | xargs git show \
    | git patch-id --stable > $TMPTO/upstream

git log --pretty=format:%H "$TO_OLDEST"..."$REMOTE/$RELEASE_NAMED" \
    | xargs git show \
    | git patch-id --stable > $TMPTO/against

cat <<EOF
From: Graham Christensen <graham@grahamc.com>
To: nix-security-announce@googlegroups.com
Subject: Security fixes from $(date -u "+%F %R %Z")
--text follows this line--
<#secure method=pgpmime mode=sign>

The following issues have been resolved in NixOS in unstable and
$RELEASE_NAMED. They remain potentially vulnerable on older major
releases.

These patches will be released to the unstable and
$RELEASE_NAMED channels when Hydra finishes building the "tested" job
for each channel:

 - https://hydra.nixos.org/job/nixos/${RELEASE_NAMED}/tested
 - https://hydra.nixos.org/job/nixos/trunk-combined/tested

Please consider helping with the next security roundup by commenting on
LATEST_ROUNDUP_URL.

EOF

table() {
  echo "$UPSTREAM|$TO_SHORT|Message|Notes"
  echo "---|---|---|---"
  for id in `cat $TMPTO/upstream | cut -d' ' -f1`; do
      master=$(git rev-parse --short $(cat $TMPTO/upstream | grep -G "^${id}" | cut -d' ' -f2))
      release=$(git rev-parse --short $(cat $TMPTO/against | grep -G "^${id}" | cut -d' ' -f2))
      message=$(git log --pretty=format:%s $master...$master~1)
      if [ $(echo "$message" | wc -c) -gt 50 ]; then
         message=$(echo -n "$message" | head -c47; echo "...");
      fi
      echo "$master|$release|$message|n/a"
  done
}

table | column -s"|" -t

update_state() {
    echo "TO_OLDEST=$(git rev-parse "$REMOTE/$RELEASE_NAMED")"
    echo "UPSTREAM_OLDEST=$(git rev-parse "$REMOTE/$UPSTREAM")"
}

update_state > "$DIR/state/port_state.sh"
