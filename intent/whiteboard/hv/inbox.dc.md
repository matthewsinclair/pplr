# inbox: dc -> hv

## (2026-11-03 11:34Z)

# pplr 2.0.0: hv's one sitting

From dc, for vc decision 24. vc verified main in a fresh clone at c8d5318 (check all 0, test all 0). On top of that are f39b6a1 (comments and the AT rows) and 6b1328f (the signer runs Gatekeeper's check after notarising). Run every step in iTerm on rhadamanth, because Contacts reaches a CLI only from iTerm (step 8). Allow about an hour, most of it builds, CI and Apple. Each step says what it should print. If it prints anything else, stop and paste it to dc. Nothing public happens before step 3.

## 0. Before you start

Tell vc (or dc) you're starting. vc keeps the slot desk clear, since the cut's gates bind the replay's port. vc also holds every node's commits to main until step 7, so the shared checkout stays a fast-forward of the release.

```
export HOMEBREW_NO_AUTO_UPDATE=1
git -C ~/Devel/prj/Pplr log -1 --format='%h %s'
```

This prints main's tip: 6b1328f, or a later commit vc names.

## 1. The three notary credentials (once)

```
security add-generic-password -a "$USER" -s geodica.apple.APPLE_ID -w
security add-generic-password -a "$USER" -s geodica.apple.APPLE_APP_PASSWORD -w
security add-generic-password -a "$USER" -s geodica.apple.APPLE_TEAM_ID -w
```

Each asks for its value twice ("password data for new item:", then "retype password for new item:") and prints nothing. The values are:

- the Apple ID of the Geodica developer account;
- an app-specific password for that account, made at account.apple.com under Sign-In and Security;
- the team id, 76BQL8L47U.

The signer reads them in exactly this form: `security find-generic-password -a $USER -s geodica.apple.<NAME> -w`.

## 2. The first signing and notarisation, before anything is public

This is a proof run at main, in a release clone. Its asset is thrown away: step 4 builds the release's asset at the tag.

```
git clone -q ~/Devel/prj/Pplr ~/Devel/scratch/pplr-release
cd ~/Devel/scratch/pplr-release
git remote remove origin
git remote add local ~/Dropbox/Repositories/Devel/Pplr
git remote add upstream git@github.com-matthewsinclair:matthewsinclair/pplr.git
cp ~/Devel/prj/Pplr/.git/hooks/pre-commit ~/Devel/prj/Pplr/.git/hooks/pre-commit.intent .git/hooks/
bin/devbin build --release
bin/devbin macos prepare
```

- At `-- signing pplr as com.geodica.pplr`, macOS may ask whether codesign can use the Developer ID's key. Type the login password and click Always Allow. If you click Allow instead, it asks once per binary.
- Each of the three binaries then shows `ok: requirement, hardened runtime`. `pplr-contacts` adds `entitlement`.
- Next come `-- submitting to Apple (notarytool waits)` and notarytool's own lines, ending `status: Accepted`. If macOS asks whether `security` can read a geodica.apple item, click Always Allow.
- Pack's verify adds Gatekeeper: `ok: requirement, hardened runtime, spctl` for each binary.
- The last line is `packed: dist/release/pplr-2.0.0-aarch64-apple-darwin.tar.gz (signed and notarised)`, and the exit code is 0.

## 3. The cut: candidate/v2.0.0 and its CI, the tag, the pushes, the release

```
bin/devbin release v2.0.0
```

It asks once, naming everything it will do: push the release commit to upstream as candidate/v2.0.0 for CI, tag v2.0.0, push the commit and tag to local and upstream, and make the release object "v2.0.0". Answer `y`. Then it shows:

- stamp: `bin/devbin lock-sync`, and CHANGELOG's 2.0.0 section dated today;
- commit: `release: v2.0.0`;
- legs: the gates here (check all, test all, doctor) and CI on candidate/v2.0.0, running together, for about ten minutes;
- the tag, the pushes to local and upstream, and `gh release create v2.0.0`;
- after: skipped, since none is declared;
- a closing note: once the tap has the tag, `brew upgrade pplr`, then `pplr use brew`, after which the version line reads brew.

The exit code is 0. If it stops partway, run `bin/devbin release v2.0.0` again: it carries on from where it stopped.

## 4. The release asset, built at the tag

```
git log -1 --format='%h %s'
bin/devbin build --release
target/release/pplr --version
bin/devbin macos prepare
bin/devbin release attach v2.0.0 --from dist/release
```

- `git log` prints `<sha> release: v2.0.0`. The `--version` line names that commit, `pplr 2.0.0 (dev, <its sha12>)`, with no `+dirty`.
- prepare prints what it printed in step 2: a second `status: Accepted`, then `packed: dist/release/...`. If you clicked Always Allow in step 2, there is no keychain dialog this time.
- attach finds one file under dist/release, `pplr-2.0.0-aarch64-apple-darwin.tar.gz`. It uploads it, writes SHA256SUMS, reads both back, and exits 0.

## 5. The formula and the tap's description

```
bin/devbin formula publish
gh repo edit matthewsinclair/homebrew-pplr --description "Homebrew tap for pplr"
gh repo view matthewsinclair/homebrew-pplr --json description
```

- publish prints `formula: pplr v2.0.0 at <sha> is ready to push to the tap at ssh://git@github.com/matthewsinclair/homebrew-pplr.git (Formula/pplr.rb)`, then asks `formula: push it?`. Answer `y`. It ends with `formula: published pplr v2.0.0 ...`, naming the tap's new head.
- The view prints `{"description":"Homebrew tap for pplr"}`.

## 6. Install from the tap

```
brew tap matthewsinclair/pplr
brew trust --formula matthewsinclair/pplr/pplr
brew install matthewsinclair/pplr/pplr
brew test pplr
$(brew --prefix)/bin/pplr --version
```

- `brew trust` is your call. Homebrew 7 loads a formula from a non-official tap only once it is trusted, which is how the estate's other formulae are installed (eg matthewsinclair/ccusage/ccusage).
- install verifies each signature, writes the manifest, and prints the caveats: `pplr use brew`, the data root and the Contacts grant.
- `brew test pplr` exits 0.
- The version line reads `pplr 2.0.0 (brew, <sha12>)`, where the sha12 is the release commit's.

## 7. Brew's pplr answers: the shared checkout, T10 and the handler

```
cd ~/Devel/prj/Pplr
git fetch -q local && git merge --ff-only local/main
pplr handler --uninstall
bin/devbin build --release
PPLR_OPT_PREFIX=~/Devel/scratch/pplr-opt target/release/pplr install
PPLR_OPT_PREFIX=~/Devel/scratch/pplr-opt bin/devbin t10-probe ~/Devel/prj/Pplr
hash -r
pplr --version
pplr doctor
pplr handler --install
pplr handler --status
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -dump Bundle | grep -E '^path:.*Pplr Links\.app'
ls -l ~/.local/bin/pplr
```

- The merge fast-forwards to `release: v2.0.0`. After this, vc lets the nodes commit again.
- `handler --uninstall` unregisters the dev tree's app. It has to run while dev's pplr still answers.
- t10-probe prints `ok` for each tree's version line, doctor check and shadow check, in the order dev, opt, brew. It ends on brew and exits 0.
- `pplr --version` reads `pplr 2.0.0 (brew, <sha12>)`, and `pplr doctor` is clean.
- `handler --status` names brew's `Pplr Links.app`, and lsregister prints exactly one path, under brew's keg. Then click one `pplr://` link: it should open in brew's pplr. If lsregister shows any other Pplr Links.app, unregister it with that tree's own pplr, run by its full path (eg `~/Devel/scratch/pplr-opt/bin/pplr handler --uninstall`).
- `ls -l ~/.local/bin/pplr` says No such file or directory.

## 8. The Contacts grant across a reinstall, still in iTerm

```
pplr sync --check
brew reinstall pplr
pplr sync --check
```

- The first `sync --check` reads the live store. macOS may ask once for Contacts access for the sidecar signed with the Developer ID. Allow it, and note what the prompt said.
- After the reinstall, the second `sync --check` reads the store with no prompt.

## 9. Back to dc

Paste each step's output to dc, who records the evidence:

- AT-07.1's green, from step 4;
- AT-04.1's green, from step 6;
- ST0002's criteria: AC-00.1 from step 6, AC-00.2 and AC-00.3 from step 7, AC-04.2 from step 5, AC-06.1 from steps 3 and 5, AC-06.2 from step 8, and AC-07.2 from step 4.

After that, the release clone and the opt tree under ~/Devel/scratch can be deleted.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
