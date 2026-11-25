# Evidence, citations and refs

A module of `AGENTS.md`, as binding as the core. Read it in full when any
line below applies; when unsure, read it.

- Scope: where evidence lives, how a commit is cited and preserved, branches and tags.
- Kind: provenance work; a phase close.
- Paths: `analysis/`, `tools/results/`, `tools/diag/citation_foreign.tsv`, `.gitignore`, `.gitattributes`.
- Before: writing a commit hash, branch or tag into a document; deleting or keeping a branch; deleting a tag; `git add -f`, `git rm` or untracking a file; archiving a binary, log or run output; asking for a pull request.

## Evidence

- Development and raw evidence live on this machine; macOS and Windows on ARM
  are compatibility-test hosts. Git holds source, build and CI files, reusable
  tools, required fixtures and concise records; raw runs, logs, executables,
  traces and bundles stay in ignored `analysis/artifacts/` or `tools/results/`
  with their paths, recipes and hashes recorded (`analysis/README.md`). Small
  frozen datasets that tests or tools consume stay versioned. Compatibility
  checks never depend on this machine's private outputs.
- Information lives in the tracked documents, not in refs. A ledger row, PLAN
  record or analysis states what changed, why, the result and the decision,
  and carries the exact recipe (values inline, or a small patch file in Git)
  plus a fingerprint proving a rebuild matched. A hash, branch or tag is at
  most a pointer beside that information, never its only carrier (RAR-S54,
  `analysis/ledger_commits_2026-09-27.md`).
- A commit is preserved by its recipe in Git, never by a kept branch or a
  new tag: a few lines of values in the citing record, or a small patch
  file in `analysis/arm_patches/` proved to reproduce the commit's tree from
  a base on a kept ref; the citing document then names the recipe, not the
  hash (maintainer decision 2026-11-04, replacing the annotated tags of
  2026-10-09). The tags kept are the releases (`v*`) and the oracle's three
  (`oracle/*`, a vendored Stockfish tree too large for a patch), which
  retire by the conditions their records state.
- No mess: a throwaway work branch is deleted once its information is
  recorded, and at each phase close tags and branches are reviewed and any
  whose retirement condition fired are removed. Before
  removing one, check what still cites it and whether anything else holds
  its commits (`git for-each-ref --contains <sha>`; empty means it would
  dangle).

## Cited commits

- A cited commit stays reachable: on `master`, under a PR (GitHub keeps
  every PR's commits as `refs/pull/<n>/head`), on a kept tag, or on the
  local branch of work in flight. The landed state is cited by its squash
  commit on `master`; a branch's own commits stay citable for the step each
  made. A gate arm is cited by its recipe file once its branch is deleted.
  `python tools/diag/check_citations.py` fails on any other commit, and on
  hex that is neither a commit here nor listed in
  `tools/diag/citation_foreign.tsv`; it needs the PR refs fetched once per
  clone (`git fetch origin "+refs/pull/*/head:refs/remotes/origin/pr/*"`)
  (maintainer decisions 2026-10-09). A one-off fetch does not keep them:
  `refs/remotes/origin/pr/*` lies under the branch refspec's destination, so
  the next `git fetch --prune` deletes them as branches the remote lacks. Add
  the refspec to the remote once per clone instead,
  `git config --add remote.origin.fetch "+refs/pull/*/head:refs/remotes/origin/pr/*"`,
  and every fetch, pruning or not, keeps them (maintainer decision
  2026-10-11).
