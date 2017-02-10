# POSTING TODO (DELETE PRIOR TO POSTING)

 - [ ] Title it "Vulnerability Roundup <n>"
 - [ ] Update the last roundup link
 - [ ] CC everyone who participated in the previous roundup
 - [ ] Tag with `security`

---

Here are all the vulnerabilities from https://lwn.net/Vulnerabilities
since our [last roundup]().

cc: .

_Note:_ The list of people CC'd on this issue participated in the last
roundup. If you participate on this roundup, I'll cc you on the next
one. If you don't participate in the next one, you won't be CC'd on
the one after that. If you would like to be CC'd on the next roundup,
add a comment to the most recent vulnerability roundup.

Permanent CC's: @joepie91, @phanimahesh, @the-kenny, @7c6f434c, @k0001, @peterhoeg
@NixOS/security-notifications
If you would like to be CC'd on _all_ roundups (or removed from the
list), open a PR editing
https://github.com/NixOS/security/blob/master/lwnvulns/src/bin/instructions.md.

## Notes on the list
1. The reports have been roughly grouped by the package name. This
   isn't perfect, but is intended to help identify if a whole group
   of reports is resolved already.
2. Some issues will be duplicated, because it affects multiple
   packages. For example, there are sometimes problems that impact
   thunderbird, and firefox. LWN might report in one vulnerability
   "thunderbird firefox". These names have been split to make sure
   both packages get addressed.
3. By each issue is a link to code search for the package name, and
   a Github search by filename. These are to help, but may not return
   results when we do in fact package the software. If a search
   doesn't turn up, please try altering the search criteria or
   looking in nixpkgs manually before asserting we don't have it.
4. This issue is created by https://github.com/NixOS/security

# Instructions:

1. Triage a report: If we don't have the software or our version isn't
   vulnerable, tick the box or add a comment with the report number,
   stating it isn't vulnerable.
2. Fix the issue: If we do have the software and it is vulnerable,
   either leave a comment on this issue saying so, even open a pull
   request with the fix. If you open a PR, make sure to tag this
   issue so we can coordinate.
3. When an entire section is completed, move the section to the
   "Triaged and Resolved Issues" `details` block below.



## Upon Completion ...

 - [ ] Run the issue through `reformat` one last time
 - [ ] Review commits since last roundup for backport candidates
 - [ ] Send an update e-mail to nix-security-announce@googlegroups.com
 - [ ] Update the database at https://github.com/NixOS/security

Without further ado...
