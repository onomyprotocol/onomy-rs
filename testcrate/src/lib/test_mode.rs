use clap::clap_derive::ArgEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum TestMode {
    /// The most basic test with the health of `equity_core` being checked
    Health,
    GetResponse,
}

impl TestMode {
    pub fn typed(&self) -> &str {
        match self {
            TestMode::Health => "health",
            TestMode::GetResponse => "get-response",
        }
    }
}
