# lwn-vulns automation

## How It Works

In the root of this repository is a file titled `db`. It is a new-line
separated list of vulnerability IDs from the LWN database.

The [`new`][new] tool screen-scrapes the database until it finds two
full pages with no new vulnerabilities.

The [`reformat`][reformat] tool updates an issue in progress to
highlight remaining items to do.

When a roundup issue is closed, the contents of it are sent to the
[`updatedb`][updatedb] command which outputs a list of the
vulnerability IDs marked as done in the roundup. This should then
appended to the `db` file.

The shell script [`ported.sh`][ported] looks for cherry-picked commits
from master to stable, to make most of a security announcement.

### Tool Interface

These tools are a bit like plumbing right now, and I would like some
simpler user interfaces to be developed on top. Right now, I think
they work okay.

## Lifecycle of an Issue

Here is a typical workflow. I'll be using `pbcopy` and `pbpaste` to
copy and paste to/from my system clipboard. On Linux, it may be
`xclip -sel clip -i` and `xclip -sel clip -o`.

### Build the tools

Start with `nix-build ./default.nix -A lwnvulns.pkg`. This will create
the `result` symlink referenced here. Note, if you're doing
development, you can enter a `nix-shell` (just run `nix-shell`) and
replace `./result/bin/` with `cargo run --bin <command>`:

    security$ nix-shell

    [nix-shell:~/projects/security/lwnvulns]$ pbpaste | cargo run --bin reformat | pbcopy
        Finished debug [unoptimized + debuginfo] target(s) in 0.0 secs
         Running `target/debug/reformat`


### Making a new issue

    $ ./result/bin/new | pbcopy
    Page 0 yielded 30 issues, after 0 pages with nothing
    Page 1 yielded 0 issues, after 0 pages with nothing
    Page 2 yielded 0 issues, after 1 pages with nothing

My clipboard now contains a full report to open as an issue. It has a
few sections of things in here for you to do. Starting with:

    # POSTING TODO (DELETE PRIOR TO POSTING)

     - [ ] Title it "Vulnerability Roundup <n>"
     - [ ] Update the last roundup link
     - [ ] CC everyone who participated in the previous roundup
     - [ ] Label with `security`


Where, obviously, `<n>` is the last one +1. For example, if the last
one was Roundup 9, this one would be Roundup 10. A bit later there
will be a place to put a link to the previous roundup:


    Here are all the vulnerabilities from https://lwn.net/Vulnerabilities
    since our [last roundup]()

between those two `()`. Make sure to correctly find the latest roundup
and update this link accordingly.

Then:

    cc: .

Visit the last roundup and review all the contributors in the sidebar.
It will say something like "7 participants". Make sure each one of
those people are CC'd in the new issue. This way it is easy for people
to join and drop out of roundups. If they participate in one, they'll
be tagged in the next one. If they don't participate in that one, they
won't be tagged on the one after that.

### Updating an issue

1. Refresh the issue's page. This is important to make sure we don't
accidentally delete progress not loaded on your page.
2. Click edit on the issue, and copy the full markdown contents.
3. Run it through the [`reformat`][reformat] tool like this: `pbpaste
   | ./result/bin/reformat | pbcopy`.
4. Paste the newly altered contents in to the issue, and click Save.

### Closing an issue

After the roundup is done and all the issues are solved, make sure to
finish out this list:

    ## Upon Completion ...
     - [ ] Run the issue through `reformat` one last time
       through `reformat` again to show all the issues again.
     - [ ] Review commits since last roundup for backport candidates
     - [ ] Update https://github.com/NixOS/nixpkgs/issues/13515 with a
     summary.
     - [ ] Update the database at https://github.com/NixOS/security

### Updating the database

1. Refresh the issue's page. This is important to make sure we don't
accidentally delete progress not loaded on your page.
2. Click edit on the issue, and copy the full markdown contents.
3. `pbpaste | ./result/bin/updatedb >> db`
4. Commit these changes, and open a PR with the new changes.

### Review and backport commits from master to  stable (`release-16.09`)

Page through commits to `master` and try and find all commits which
contained security fixes. Make sure any security fixes to master are
applied to the stable branch as well. If not, cherry-pick them
yourself. If you're not sure, open a PR with the backported commits.

### Creating an Announcement

This tool is quite rough. It is [`./ported.sh`][ported] and it looks
at all the commits on the release which were cherry-picked from
master.

1. `cd` in to a nixpkgs clone and run [`ported.sh`][ported]. It will
   output a rough template of all the announcements to make, but make
   sure to audit it and review, by following the remainder of these
   steps.
2. Delete any backported commit lines which _were not_ for security.
3. Page through the commits to the release branch and identify commits
   which are security related, but are not cherrypicks from master.
   The [`ported`][ported] tool WILL miss these, so it is imperative to
   check.
4. Update the link at the end of the output to point to the latest
   security vulnerability roundup.
5. Commit and push and open a PR with the updated ported state file.

## Developing

Run `nix-shell` and within that, `cargo run --bin
new|updatedb|reformat` etc.

### Architecture
This tooling includes a tokenizer for tokenizing the issues, a parser
to build a "syntax tree", and then various AST transformations to
update the document later on. There is a tool for writing out a syntax
tree as text. The [`new`][new] tool which generates new reports emits
tokens to the parser, and then uses the writer to output the report.

It is important to me that code be well formatted and have tests. One
place in particular that I have failed to adhere to these standards in
particular is the [`new`][new] code... this isn't an excuse to get
sloppy. Sorry. :(

## Dataformat

### Terms

#### Document

A document is the entire markdown contents of a Github Issue. A
document begins with a **preamble**, and ends with a **report**.

#### Preamble

The preamble is arbitrary text and has no specific rules about its
content. When being parsed and generated, the preamble is left
completely alone and is to be preserved bit-for-bit when outputted.

The preamble is begins at the start of the document, and ends at the
first occurance of a **Header**.

#### Report

The Report is a collection of **Headers** and **Issues**, where a
Header is typically followed by zero or more Issues.

**Any lines of data inside the Report which is not a Header or an
Issue is considered garbage, and should be discarded.**

#### Header

A Header is defined by the following regular expression:

    ^### (.*) \((\d+) issues?\)( .*)?\n$
         (1)    (2)            (3)

         (1) Package Name
         (2) Issue Count
         (3) Additional Notes (optional)

The header is designed to start with arbitrary text describing the
affected packages, the number of issues affecting the package, and
then optional notes. Note that the number of issues in the header
_does not_ necessarily reflect the true number of contained issues,
however a well behaved parser will correctly update the counter. Note
that the plural `s` in `issues` is optional.

Here are some example headers:

    ### jasper (2 issues)
    ### jasper (0 issues)
    ### jasper (1 issue)
    ### foo bar baz tux!!! (1 issues) extra notes go here

#### Issue

An Issue is defined by the following regular expression:

    ^ ?- \[(x| )\] \[`#(.*)`\]\((.*)\) (.*)$
           (1)         (2)      (3)    (4)

           (1) Completion Indicator (x == complete)
           (2) Vulnerability ID
           (3) URL for the Vulnerability
           (4) Arbitraty text describing the vulnerability


The issue is designed to be in a markdown formatted list with a
beginning checkbox and a link. This checkbox may either be filled or
unfilled, following a link indicating the primary URL for this issue.
The link's text, currently, must start with a #, followed by an
identifier to identify this issue. It is assumed this identifier is
unique to this issue, and that this issue will never need to be
addressed again. Following the link may be arbitrary content. The
markdown list line may or may not be prefixed by a blank space.

Here are some example Issues:

     - [x] [`#705362`](https://lwn.net/Vulnerabilities/705362/) ([search](http://search.nix.gsc.io/?q=bind&i=fosho&repos=nixos-nixpkgs), [files](https://github.com/NixOS/nixpkgs/search?utf8=%E2%9C%93&q=bind+in%3Apath&type=Code))  bind: denial of service
     - [ ] [`#705917`](https://lwn.net/Vulnerabilities/705917/) ([search](http://search.nix.gsc.io/?q=java-1.8.0-openjdk-aarch32&i=fosho&repos=nixos-nixpkgs), [files](https://github.com/NixOS/nixpkgs/search?utf8=%E2%9C%93&q=java-1.8.0-openjdk-aarch32+in%3Apath&type=Code))  java-1.8.0-openjdk-aarch32: multiple vulnerabilities
    - [x] [`#705362`](https://lwn.net/Vulnerabilities/705362/) ([search](http://search.nix.gsc.io/?q=bind&i=fosho&repos=nixos-nixpkgs), [files](https://github.com/NixOS/nixpkgs/search?utf8=%E2%9C%93&q=bind+in%3Apath&type=Code))  bind: denial of service
    - [ ] [`#705917`](https://lwn.net/Vulnerabilities/705917/) ([search](http://search.nix.gsc.io/?q=java-1.8.0-openjdk-aarch32&i=fosho&repos=nixos-nixpkgs), [files](https://github.com/NixOS/nixpkgs/search?utf8=%E2%9C%93&q=java-1.8.0-openjdk-aarch32+in%3Apath&type=Code))  java-1.8.0-openjdk-aarch32: multiple vulnerabilities

### Example Document

    Here are all the vulnerabilities from https://lwn.net/Vulnerabilities
    ## Notes on the list
    1. The reports have been roughly grouped by the package name. This
       isn't perfect, but is intended to help identify if a whole group
    ### This is valid too, because it doesn't have an issue count!
     - [ ] even this isn't counted!


    ### Assorted (31 issues)
     - [ ] [`#705568`](https://lwn.net/Vulnerabilities/705568/) ([search](http://search.nix.gsc.io/?q=libvirt&i=fosho&repos=nixos-nixpkgs), [files](https://github.com/NixOS/nixpkgs/search?utf8=%E2%9C%93&q=libvirt+in%3Apath&type=Code))  libvirt: privilege escalation
     - [ ] [`#705361`](https://lwn.net/Vulnerabilities/705361/) ([search](http://search.nix.gsc.io/?q=java&i=fosho&repos=nixos-nixpkgs), [files](https://github.com/NixOS/nixpkgs/search?utf8=%E2%9C%93&q=java+in%3Apath&type=Code))  java: unspecified vulnerability
     - [ ] [`#705578`](https://lwn.net/Vulnerabilities/705578/) ([search](http://search.nix.gsc.io/?q=qemu&i=fosho&repos=nixos-nixpkgs), [files](https://github.com/NixOS/nixpkgs/search?utf8=%E2%9C%93&q=qemu+in%3Apath&type=Code))  qemu: multiple vulnerabilities
    This stuff is garbage and will be deleted when the parser is run again.
    ### tiff (2 issues)
     - [x] [`#705364`](https://lwn.net/Vulnerabilities/705364/) ([search](http://search.nix.gsc.io/?q=tiff&i=fosho&repos=nixos-nixpkgs), [files](https://github.com/NixOS/nixpkgs/search?utf8=%E2%9C%93&q=tiff+in%3Apath&type=Code))  tiff: multiple vulnerabilities
     - [x] [`#635993`](https://lwn.net/Vulnerabilities/635993/) ([search](http://search.nix.gsc.io/?q=tiff&i=fosho&repos=nixos-nixpkgs), [files](https://github.com/NixOS/nixpkgs/search?utf8=%E2%9C%93&q=tiff+in%3Apath&type=Code))  tiff: multiple vulnerabilities

# Addendum

## Why LWN? Why not NVD?

LWN nicely aggregates reports from distributions, who also aggregate
CVE IDs they are responding to. This means instead of checking several
CVE IDs individually, we know we just need to update a package.

NVD and other CVE databases are frequently dreadfully out of date, and
are won't have data for a CVE data for a long time, where as LWN will
already have information about the report.

## Has LWN approved this?

Yes.

## `new` emits `Problem with the SSL CA cert (path? access rights?)`?

I was missing a `/etc/ssl/certs/ca-certificates.crt` and copied my
`/etc/ssl/certs/ca-bundle.crt` to be there... _shrug_.


## Why rust?

I originally wrote this tooling in python, but wanted to have strong
typing to provide structure to the parser and tokenizer. I don't any
functional languages that are vogue in the Nix ecosystem.

[new]: ./lwnvulns/src/bin/new.rs
[reformat]: ./lwnvulns/src/bin/reformat.rs
[updatedb]: ./lwnvulns/src/bin/updatedb.rs
[ported]: ./ported.sh
