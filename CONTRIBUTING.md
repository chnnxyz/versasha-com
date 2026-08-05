# Contributing

Thanks for taking a look. This is a personal project first, so the bar for
"does this fit" is a bit particular — read below before sending a big PR out
of the blue, but small fixes and clear bug reports are always welcome.

## Before you start

For anything bigger than a typo fix or a one-line bug fix, open an issue
first describing what you want to change and why. That saves both of us the
time of a PR that doesn't end up matching the project's direction.

## What's hand-written vs AI-assisted

This matters for how to approach a change:

- **`src/`, the Rust DSP/audio engine** — hand-written and hand-reviewed by
  the maintainer, who is deliberately learning Rust and real-time audio
  programming through this project. If you send a Rust PR, expect scrutiny
  on *why* a change is correct, not just *that* it compiles and passes
  tests — that's the whole point of doing this by hand. Bug fixes and small,
  well-explained improvements are genuinely welcome; large unsolicited
  rewrites or "let me modernize this for you" PRs are not, even if
  technically an improvement.
- **`index.html`, `css/`, `js/`** — built with AI assistance (Claude), not
  hand-written. This code is held to a normal correctness/quality bar like
  any other code, but there's no "learning by hand" constraint on it —
  clean refactors and straightforward fixes here are easier to land than
  the same kind of change in `src/`.

## Rust conventions this project actually enforces

The audio callback runs on a real-time thread and cannot glitch, so:

- **Never panic in code that runs per-sample.** Indices that come from
  outside Rust (a JS-supplied track/step index, for example) are validated
  with `Option`/bounds checks, never indexed directly. This also covers
  indices your own code computes with floating-point math (a swept delay
  read position, say) — `rem_euclid`/similar are only guaranteed in-range
  *mathematically*; rounding can still land exactly on one past the last
  valid index. Clamp the final integer index, not just the float leading
  up to it.
- **No allocation in the audio callback.** `next_sample()`/`process()` paths
  should not `Vec::push`, `format!`, or otherwise allocate.
- **"Always advance, decide at output."** When something is muted, soloed
  out, or otherwise excluded from the current mix, its own `next_sample()`/
  `process()` still has to run every sample — only the *decision to include
  it in the sum* gets skipped. Skipping the call itself freezes that
  component's internal state (a filter's phase, an envelope's stage) and
  produces an audible glitch when it's un-excluded later.
- **`crate::` imports outside of tests.** `use super::*;` is fine (and
  expected) inside `#[cfg(test)] mod tests` blocks; everywhere else, prefer
  the absolute path.
- **No dead code.** No unused fields, no unused imports, no commented-out
  blocks left "just in case." Before removing something because it looks
  unused from `index.html`/`SessionEngine`, check whether it's still
  reachable from one of the *other* wasm bindings in `wasm.rs`
  (`SynthEngine`/`DrumMachineEngine`/`AcidSynthEngine`) and their standalone
  pages (`drums.html`, `acid.html`) — several core types are shared across
  more than one binding, so "unused by the unified page" isn't the same as
  "unused."

## Tests

```bash
cargo test
cargo clippy --all-targets
```

Both need to be clean before a PR is reviewed. New DSP/engine logic gets
unit tests alongside it — this codebase leans on exact arithmetic (tiny
power-of-2 sample rates, hand-computed expected values) over loose-epsilon
assertions wherever the math allows it, so a regression can't hide behind a
tolerance that's too forgiving.

## Frontend changes

If you're touching `index.html`/`js/`, actually load the page and click
through what you changed before opening the PR — a description of the
change, not just "should work," and ideally a before/after screenshot or
GIF for anything visual.

## Commit / PR style

- Keep PRs scoped to one change. A drum-machine bug fix and a CSS tweak
  belong in separate PRs.
- Explain *why*, not just *what*, in the PR description — same standard
  the maintainer holds their own Rust changes to.
- Don't bundle formatting-only diffs with a functional change; if a file
  genuinely needs a reformat, say so and do it as its own PR.
