pub mod debug_sandbox;
mod exit_status;
pub mod login;
pub mod proto;

use clap::Parser;
use codex_common::CliConfigOverrides;

use anyhow::Result;

pub fn build_auth_url_only(opts: &ServerOptions) -> Result<String> {
    // whatever your existing code does internally to produce `auth_url`,
    // re-use it here without starting the Hyper server.
    //
    // e.g., if you have something like:
    // let (auth_url, _state, _code_verifier) = oauth::build_authorize_url(&opts)?;
    // Ok(auth_url)
    crate::server::build_authorize_url_for(opts) // <- refactor to a shared helper if needed
}

// if you don't have a clean helper yet, expose one from server.rs:
#[derive(Debug, Parser)]
pub struct SeatbeltCommand {
    /// Convenience alias for low-friction sandboxed automatic execution (network-disabled sandbox that can write to cwd and TMPDIR)
    #[arg(long = "full-auto", default_value_t = false)]
    pub full_auto: bool,

    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,

    /// Full command args to run under seatbelt.
    #[arg(trailing_var_arg = true)]
    pub command: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct LandlockCommand {
    /// Convenience alias for low-friction sandboxed automatic execution (network-disabled sandbox that can write to cwd and TMPDIR)
    #[arg(long = "full-auto", default_value_t = false)]
    pub full_auto: bool,

    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,

    /// Full command args to run under landlock.
    #[arg(trailing_var_arg = true)]
    pub command: Vec<String>,
}
