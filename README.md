# versasha.com

The personal site of Sasha Ruiz de Aguirre — DJ, producer, and the excuse for
everything in this repo. It also happens to contain a full subtractive synth,
a TR-909-style drum machine, a TB-303-style acid bass sequencer, and a step
arpeggiator, all written from scratch in Rust, compiled to WebAssembly, and
running in real time in the browser — plus a mixer that ties all four
together, complete with a Pioneer-style Beat FX send. No audio libraries, no
samplers-as-a-service — raw f32 samples, computed one at a time, on an
`AudioWorklet` thread that never touches the DOM.

## What's actually here

**A polyphonic synth** — two detunable oscillators, a resonant multimode
filter (state-variable, unconditionally stable even with the cutoff maxed
out), an LFO that can modulate pitch, vibrato, volume, or filter cutoff, and
a feedback delay routable to either oscillator or the master bus. Playable
with a computer keyboard (`A W S E D F T G Y H U J`, one octave, black keys
and all) or an on-screen keyboard on mobile.

**A drum machine** — 11 tracks modeled after a real 909's kit (kick, snare,
clap, rimshot, three toms, two hi-hats, two cymbals), a 16-step sequencer,
mute/solo per track, and per-instrument shaping knobs where they actually
matter: Tune on anything pitched, Attack/Decay on the kick and toms, Snappy
noise-mixing on the snare. Click a track name to fire it live; draw on the
grid to program a pattern.

**A bass sequencer** — a TB-303-style acid voice (saw/square oscillator,
diode-ladder filter, filter and amp envelopes, glide, accent) driven by its
own 16-step sequencer with per-step note/gate/accent/slide. Shares the same
clock as the drum machine, so patterns on both stay locked together.

**An arpeggiator** — 4 chord slots, one per beat of a bar, each holding a
whole chord drawn into a piano-roll editor (not just one note). Two
independent rates layer on top: Note Rate controls how fast notes cycle
within whichever chord is currently held, Chord Rate controls how long each
of the 4 slots holds before advancing to the next — both synced to the same
shared BPM as everything else. Up/Down/Up-Down/Random pattern modes, its own
waveform choice, and an octave-range knob that both widens how far the
arpeggiation reaches *and* resizes the piano roll to match.

**A mixer** — one channel strip per instrument (3-band EQ, pan, volume
fader, mute/solo, a segmented VU meter) plus a master section with its own
stereo VU meter and volume knob. All four instruments — and the live synth
— run through it into one shared stereo bus. A Pioneer DJM-style Beat FX
send sits alongside the master strip: one effect (Delay, Reverb, or Flanger)
live at a time, routable to any channel, its rate synced to the shared BPM,
a single Dry/Wet knob, and an on/off toggle.

**All five tabs, one engine** — Drums and Bass share a single Transport
(one Play/Pause/Stop/BPM, in the header, above every tab); the synth plays
live on top, untouched by the transport; the arp rides the same shared
Transport for *which* chord is active, but keeps two further clocks of its
own (Note Rate, Chord Rate) that Drums/Bass don't need; the mixer reaches
all four. One `AudioContext`, one `AudioWorkletNode`. Recording (the dot in
the header) captures whatever the mixer is currently outputting to a
downloadable WAV, regardless of which tab is visible.

## How it's built

```
Rust DSP engine  →  wasm-bindgen  →  compiled .wasm  →  AudioWorklet  →  speakers
```

The entire signal path — oscillators, filters, envelopes, the sequencer's
sample-accurate clock, noise generation, the mixer's own EQ/pan/metering,
everything — is plain Rust with no `unsafe`, no allocation in the audio
callback, and never a panic on bad input from JS (out-of-range indices are
handled with `Option`, not indexing). It compiles to a `cdylib` via
`wasm-bindgen`, gets loaded into an `AudioWorkletProcessor` on its own
real-time thread, and is driven sample-by-sample from `process()`. The main
thread only ever talks to it through `postMessage` — knob turns, note-on/off,
step toggles — so the audio thread is never blocked waiting on the DOM.

```
src/
├── dsp/           generic signal-processing building blocks, organized by
│                  kind: oscillators, filters (TPT/Cytomic SVF + a
│                  diode-ladder model), envelopes (ADSR + a percussive
│                  AD variant), eq (3-band), fx (delay, algorithmic
│                  reverb, flanger), modulation (LFO + matrix), noise
│                  (xorshift32), tune, shared types
├── drums/         drum-machine-specific pieces: sample playback, the
│                  step sequencer, the percussive attack/decay envelope
├── acid_bass/     TB-303-style voice: oscillator + diode-ladder filter +
│                  envelopes + glide, its own step sequencer and step type
├── arp/           the arpeggiator: a 4-chord-slot pattern (one slot per
│                  beat), its own dedicated synth voice, and the
│                  up/down/up-down/random note-walk logic
├── synth/         the polyphonic synth's own voice and (pre-mixer) mixer
├── sequencing/    the sample-accurate step clock, Transport (the shared
│                  Play/Pause/Stop/BPM clock Drums and Bass ride), and
│                  NoteDivision (a beat-fraction control shared by the
│                  arp's own rates and the mixer's Beat FX rate)
├── mixer/         InstrumentChannel (one channel strip's EQ/pan/volume/
│                  mute-solo/VU-peak logic) and EffectsUnit (the shared
│                  Beat FX send wrapping Delay/Reverb/Flanger)
├── engine/        the top-level engines -- Synth, DrumMachine, AcidSynth,
│                  MixerEngine, and Session (wires all four instruments
│                  together behind one shared Transport)
├── params/        parameter structs
└── wasm.rs        the wasm-bindgen surface exposed to JS -- SessionEngine
                   is what the live site actually uses; the standalone
                   SynthEngine/DrumMachineEngine/AcidSynthEngine bindings
                   back the older single-instrument test pages

js/                AudioWorkletProcessor glue -- session-worklet.js is the
                   one the live site uses (wraps SessionEngine); the
                   single-instrument *-worklet.js files back the
                   standalone test pages; recorder-worklet.js taps
                   whatever's playing for WAV export
css/               site styles
samples/           909 one-shot samples (WAV), decoded client-side

index.html         the site: synth + drums + bass + arp + mixer behind a
                   tab switch, one shared engine
drums.html         standalone drum machine test page (its own engine)
acid.html          standalone bass sequencer test page (its own engine)
presskit.html
music.html
```

## Running it locally

You'll need the Rust toolchain and [`wasm-pack`](https://rustwasm.github.io/wasm-pack/).

```bash
# build the wasm bindings (pkg/ is gitignored, generate it fresh)
wasm-pack build --target web --out-dir pkg

# serve the repo root with any static file server
python3 -m http.server 8000
```

Then open `http://localhost:8000/index.html`. Browsers require a user
gesture before audio can start — the small circle next to the transport
controls turns green once it has (yellow means it's still idle); pressing
Play, a pad, or a key all trigger it too.

## Testing

```bash
cargo test          # unit tests across the DSP/drum/acid/arp/mixer/engine layers
cargo clippy --all-targets
```

Tests lean on exact arithmetic wherever possible — tiny power-of-2 sample
rates, hand-computed expected values — rather than approximate assertions,
so a regression in the math doesn't hide behind a loose epsilon.

## A note on how this was built

The Rust/DSP/audio-engine code is hand-written and hand-reviewed — that's
the part of this project that's actually being learned and owned line by
line. The frontend (`index.html`, its CSS, and the JS gluing it to the
wasm engine) was built with AI assistance (Claude), not written by hand.
If you're contributing, see [CONTRIBUTING.md](CONTRIBUTING.md) for what
that means in practice for each part of the codebase, and
[ROADMAP.md](ROADMAP.md) for planned features looking for outside help.

## License

MIT — see [LICENSE](LICENSE).
