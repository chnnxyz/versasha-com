pub mod acid_bass;
pub mod arp;
pub mod drums;
pub mod dsp;
pub mod engine;
pub mod mixer;
pub mod params;
pub mod sequencing;
pub mod synth;
pub mod wasm;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}
