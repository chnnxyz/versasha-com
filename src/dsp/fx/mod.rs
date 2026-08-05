pub mod delay;
pub mod flanger;
pub mod reverb;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FxRoute {
    Osc1,
    Osc2,
    Master,
}
