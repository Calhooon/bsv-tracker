# Changelog

## 0.1.0 (2026-10-09, unpublished)

- Implement the tracker charter's nine words and Lean transition machine.
- Require an immutable `CheckedProof` from `Headers::check` for mined words.
- Run its default checker against the current snapshot even when the host overrides `check` with a cached capability.
- Bind SDK BRC-74 paths to the proven txid and retain the checked header and depth.
- Record broadcaster hints without writing chain words, including the ARC vector and Arcade reorg vocabulary.
- Route reorgs by affected mined heights, invalidations by checked header hash and tips by suspect txid.
- Return re-ask values for host execution, with fail-closed header errors and explicit envelope faults.
- Clear the completed trigger on a matching recheck, following the landed Lean machine at f95520f.
- Expose the four host traits, with no platform clock, network or storage calls.
- Replay seven tracker witnesses and four named randomized Lean properties locally; compile the wasm32 target.

The crate has no deployed predecessor. Three host integrations and #32 wire reconciliation remain separate lanes. Successful rechecks clear their completed triggers as in the landed Lean machine.
