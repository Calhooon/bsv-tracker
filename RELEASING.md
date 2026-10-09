# Releasing bsv-tracker

1. Every change lands through a PR to `main`; nothing is pushed to `main` directly.
2. The PR is green on the five required checks: `test`, `fmt`, `clippy`, `vectors`, `wasm32`.
3. The release PR sets `version` in `Cargo.toml` (and `Cargo.lock`) and the `CHANGELOG.md` heading.
4. Merge by rebase (linear history); the owner's account merges its own PR.
5. Tag the merge commit on `main`: `git tag vX.Y.Z && git push origin vX.Y.Z` (the tag ruleset lets the owner alone create `v*`).
6. `.github/workflows/release.yml` checks the tag against `Cargo.toml` and `main`, reruns the checks, and publishes.
7. Nothing publishes from a laptop and no crates.io token is stored anywhere, with one recorded exception: the birth publish of 0.1.0 came from the captain's machine with the token, because crates.io creates a trusted-publishing entry only for a crate that already exists; the entry was created the minute after, and every later version publishes from the tag.
8. crates.io Trusted Publishing names owner `Calhooon`, repository `bsv-tracker`, workflow `release.yml`, no environment.
9. Every action in the two workflows is pinned by commit sha; a bump is its own PR, the sha read with `git ls-remote`.
10. The door opened on the owner's word of 2026-10-09 ("tracker public"): the repository is public, the branch protection and the two tag rulesets are on, and 0.1.0 is on crates.io.
