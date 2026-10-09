# Releasing bsv-tracker

1. Every change lands through a PR to `main`; nothing is pushed to `main` directly.
2. The PR is green on the five required checks: `test`, `fmt`, `clippy`, `vectors`, `wasm32`.
3. The release PR sets `version` in `Cargo.toml` (and `Cargo.lock`) and the `CHANGELOG.md` heading.
4. Merge by rebase (linear history); the owner's account merges its own PR.
5. Tag the merge commit on `main`: `git tag vX.Y.Z && git push origin vX.Y.Z` (the tag ruleset lets the owner alone create `v*`).
6. `.github/workflows/release.yml` checks the tag against `Cargo.toml` and `main`, reruns the checks, and publishes.
7. Nothing publishes from a laptop; no crates.io token is stored anywhere.
8. crates.io Trusted Publishing names owner `Calhooon`, repository `bsv-tracker`, workflow `release.yml`, no environment.
9. Every action in the two workflows is pinned by commit sha; a bump is its own PR, the sha read with `git ls-remote`.
10. Publishing waits for the door: the repository is private until the owner's word, and 0.1.0 is unpublished.
