# Contributing

Thanks for taking a look. This is a personal project first, so only changes to the rust crate and ui for the instruments are allowed. Any PR modifying `music.html` or `presskit.html` will be closed on sight.

## I'm not a developer, but I want to contribute
As a developer, I might assume stuff to be obvious for me that is not obvious for the user. Additionally, knowing the expected behavior of the program may blind me to potential bugs. If you are not a developer, you can allways [open an issue](https://github.com/chnnxyz/versasha-com/issues/new/choose).

The issue should be labeled with one of these labels:

- `Bug`: When something doesnt work as expected
- `Enhancement`: When requesting an improvement to an existing functionality.
- `Feature Request`: If you want a new module, new effect or anything not currently implementd.
- `Question`: Overall questions about functionality or how to install/run.

I provide a template for when opening issues, but keep in mind that all issues that are not `Question` must contain:

- Current functionality
- Expected functionality
- How to replicate (Only necessary for bugs)

Be as detailed as you can, without having to put additional effort to do something you currently dont know how to. For instance, for a `Feature Request` or `Enhancement`, UI mockups visual or audio samples, or even pseudocode suggestions are greatly appreciated, but not required if making those is outside your current knowledge.

## I am a developer and I want to contribute

### Requirements

Essentially, all you need is to [install rust](https://rust-lang.org/tools/install/) and have any modern Gecko/Chromium/Webkit based browser. A way to start an HTTP server is also recommended, I personally use [the base python HTTP server](https://docs.python.org/3/library/http.server.html), but anything works.

### About using AI

Any PR looking fully AI-generated will be discarded. AI-assisted code is good though. As a disclaimer: I am not and I have never been a frontend dev, so, as of publishing this file **all frontend code was AI generated** (specifically I used Claude Sonnet 5). This said, I would expect people contributing to frontend to actually know frontend. 

For Rust code, AI assistance is accepted exclusively for these cases and **nothing else**:

- Generation of unit tests
- Module and function documentation
- Folder structure refactors

Keep in mind that the rust crate is the backbone of this app, and I will protect it with my life.

### Rust conventions

- No `pub` fields in `struct`. Always use getters and setters.
- Avoid using lifetimes unless completely necessary.
- All imports have to be `crate::` and not `super::` (test modules can use `super::`, the rest cannot).
- Unit tests belong in their specific module. Due to the project structure, commonly tests ran in engines that consume other modules (e.g. anything in `src/engine`) may work as integration tests.
  - There are no rules of expected coverage percentage by tests. If you add a getter/setter pair and dont add a test for it, tht is fine. As a rule of thumb, if it implies any mathematical or logical calculation, it **WILL** need a test.
- No `panic!()` ever allowed, either explicit or implicit, since this breaks the full audio engine. For cases that might implicitly panic (e.g. index out of range), **ALWAYS** wrap on an `Option<T>` or any other sort of pattern matching.
- **NEVER ALLOCATE MEMORY ON AUDIO CALLBACKS**: `next_step()` and `process()` can never assign new things to memory.
- Clock advances no matter what. Always get next sample, advance clock and afterwrds do the remaining DSP.
- No dead code. I will not be explicitly checking for all the cases but lets try and keep the project structure, functions and imports as clean as possible.
- Floats are always `f32` and ideally using an alias fore the type such as the ones in `src/dsp/types.rs`

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

### Tests

Before sending a PR, always run

```bash
cargo test
cargo clippy --all-targets
```

Both need to be clean before a PR is reviewed. New DSP/engine logic gets
unit tests alongsidem since this codebase leans on exact arithmetic (tiny
power-of-2 sample rates, hand-computed expected values) over loose-epsilon
assertions wherever the math allows it, so a regression can't hide behind a
tolerance that's too forgiving.

### Frontend changes

All frontend changes must include a before -> after comparison with screenshots, audio samples or videos. Frontend changes with no evidence will be closed on sight.


### Commit / PR style

- PRs must always come from a forkof my codebase.
- PRs must be up to date with `main` and cause not merge conflict.
- Keep PRs scoped to one change. A drum-machine bug fix and a CSS tweak
  belong in separate PRs.
- Explain *why*, not just *what*.
- Don't bundle formatting-only diffs with a functional change; if a file
  genuinely needs a reformat, say so and do it as its own PR.
