## What does this change, and why?

<!-- Explain *why*, not just *what*. If this is bigger than a typo/one-line
     fix, link the issue you opened first. -->

## Scope and process

- [ ] This PR comes from a fork
- [ ] This PR is up to date with `main` and has no merge conflicts
- [ ] This only touches the Rust crate (`src/`) and/or the instrument UI --
      not `music.html` or `presskit.html`
- [ ] This is scoped to one change (a drum-machine bug fix and a CSS tweak
      belong in separate PRs), with no formatting-only diffs bundled in

## About AI use

- [ ] This PR is not fully AI-generated
- [ ] If this touches `src/`: any AI assistance used here was limited to
      unit test generation, module/function documentation, or folder
      structure refactors -- nothing else

## If this touches `src/` (Rust)

- [ ] `cargo test` passes
- [ ] `cargo clippy --all-targets` is clean
- [ ] No `pub` fields on any `struct` -- getters/setters only
- [ ] No lifetimes unless completely necessary
- [ ] Imports are `crate::`, not `super::` (test modules are the exception)
- [ ] Unit tests live in their own module, and anything involving a
      mathematical or logical calculation has one
- [ ] No `panic!()`, explicit or implicit -- anything that could panic
      (an index out of range, for example) is wrapped in `Option<T>` or
      otherwise pattern-matched instead
- [ ] No memory allocated in `next_sample()`/`process()`
- [ ] The clock advances no matter what: get the next sample, advance the
      clock, then do the remaining DSP -- never the other order
- [ ] No dead code -- no unused fields/imports/commented-out blocks
- [ ] Floats are `f32`, using the type aliases in `src/dsp/types.rs` where
      one applies

## If this touches the instrument UI

- [ ] Before/after comparison attached (screenshots, audio samples, or a
      video) -- PRs with no evidence get closed on sight

## Anything else the reviewer should know?
