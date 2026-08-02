pub mod delay;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FxRoute {
    Osc1,
    Osc2,
    Master,
}
