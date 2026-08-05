# Roadmap

Planned features for this project. All of these are open to outside contribution (see
[CONTRIBUTING.md](CONTRIBUTING.md)) for how to approach a PR or open an issue.

None of the items below have been started.

## 1. Grid sizes and triplets

- Drums: the step grid is currently hardcoded to 16 steps. Allow it to
  rescale based on the current time signature instead of always assuming
  4/4 at a 16th-note resolution.
- Drums: add triplet subdivisions as an option, alongside the existing
  straight (16th-note) grid.
- Drums: add ableton-style keybindings to change the grid.
- Bass: allow changing the step size. The sequencer should remain at 16 steps as a traditional 303, but allowing for switching step size.

## 2. Time signatures

Support time signatures other than 4/4. Currently 4/4 is always assumed by the shared clock in all sequenced instruments. This extends the first point but requires overhauling the shared Transport clock.

## 3. Master channel overall improvements

- Route effects to master channel
- Add a brick wall limiter option to reduce clipping.

## 4. Additional BeatFx

The mixer is roughly based on a Pioneer DJM-900. Currently, the only effects implemented are flanger, delay and reverb. From a DJM-900 perspective, the effects to add, in order of priority are:

1. Spiral
2. Phaser
3. Trans
4. Roll
5. Slip Roll
6. Ping Pong
7. Vinyl Break
8. Filter
9. Pitch

## 5. MIDI inputs
If viable, allow midi inputs from a controller into the app, not just regular keyboard.

## 6. Desktop UX

Knob controls on desktop are troublesome, adding the option to show the current value and double click the value to edit would be great.

## 7. Export for DAW

Currently, sessions can be recorded and downloaded as `.wav` files. It would be great to also export the session to `.mid` or `.txt` files that can be pulled to a user's DAW.

## 8. Build-yourself (hardware)

Schematics for building a physical hardware version of each instrument
(Synth, Drums, Bass, Arp), targeting ESP32 or Raspberry Pi-class boards,
plus the firmware/code to actually run them on that hardware..

## 9. Mobile support

This is where I need the most outside help. My frontend skills are not
good enough to fix this properly on my own. Known issues right now:

- A first note gets stuck/held on and doesn't release properly on mobile
  touch input.
- The sequenced instruments (Drums, Bass, Arp) have an inaccessible layout
  on mobile: the grids/controls aren't usable at touch sizes.
- There's no proper mobile UI for any instrument or the mixer at all,
  except the Synth's on-screen keyboard. Drums, Bass, Arp, and the Mixer
  have nothing mobile-specific: they just render their desktop layout in a semi-responsive fashion Examples of how this could be improved.

    - Selector for 303 to pick between sequencer or controls.
    - Panel selector for mixer, always showing master output somewhere.
    - In desktop, you can play lead synth on any view with your keyboard, there needs to be a similar way to do that on mobile.
    - And I honestly have no clue on what to do for the drums.
