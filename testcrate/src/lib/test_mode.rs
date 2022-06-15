use clap::clap_derive::ArgEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum TestMode {
    /// The most basic test with the health of `equity_core` being checked
    Health,
}
