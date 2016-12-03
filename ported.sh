#!/usr/bin/env bash

#!/usr/bin/env nix-shell
#!nix-shell -i bash -p git

set -eu
set -o pipefail

TMPDIR="/tmp"
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

debug() {
    set +u
    if ! [ "x$DEBUG" = "x" ]; then
        echo "$@" >&2
    fi
    set -u
}

#git fetch "$REMOTE"

cherries() {
    debug "cherries: Fetching"
    git cherry "$REMOTE/$RELEASE_NAMED" "$REMOTE/$UPSTREAM" "$UPSTREAM_OLDEST"
    debug "cherries: fetched"
}

logs() {
    debug "logs: Fetching"
    git log --pretty=format:%H "$TO_OLDEST"..."$REMOTE/$RELEASE_NAMED"
    debug "logs: fetched"
}

cherries \
    | grep -v "+" \
    | cut -d' ' -f2 \
    | xargs git show \
    | git patch-id --stable > $TMPTO/upstream

logs \
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
  for id in `cat $TMPTO/upstream | cut -d' ' -f1`; do
      debug "${id}: identify master hash"
      master_hash=$(cat $TMPTO/upstream | grep -G "^${id}" | cut -d' ' -f2 | tail -n1)
      debug "${id}: master -> $master_hash"

      set +e
      master_short=$(git rev-parse --short "${master_hash}")
      if [ "x" = "x$master_short" ]; then
          debug "${id}: BUG? Cannot find hash ${master_hash}"
          exit 1
      fi
      set -e

      debug "${id}: master -> $master_hash -> ${master_short}"

      debug "${id}: identify release hash"
      set +e
      release_hash=$(cat $TMPTO/against | grep -G "^${id}" | cut -d' ' -f2)
      if [ "x" = "x$release_hash" ]; then
          debug "${id}: did not identify candidate in release, skipping."
          continue
      fi
      set -e
      debug "${id}: release -> $release_hash"
      release_short=$(git rev-parse --short "${release_hash}")
      debug "${id}: master -> $master_hash -> ${release_short}"

      debug "${id}: fetch  message"
      message=$(git log --pretty=format:%s $master_hash...$master_hash~1)
      if [ $(echo "$message" | wc -c) -gt 50 ]; then
         message=$(echo -n "$message" | head -c47; echo "...");
      fi
      echo "$master_short|$release_short|$message|n/a" >> $TMPTO/table
      debug ""
  done

  echo "$UPSTREAM|$TO_SHORT|Message|Notes"
  echo "---|---|---|---"

  cat "$TMPTO/table" | sort -t\| -k3
}

table | column -s"|" -t

update_state() {
    echo "TO_OLDEST=$(git rev-parse "$REMOTE/$RELEASE_NAMED")"
    echo "UPSTREAM_OLDEST=$(git rev-parse "$REMOTE/$UPSTREAM")"
}

update_state > "$DIR/state/port_state.sh"
