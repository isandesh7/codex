use codex_common::CliConfigOverrides;
use codex_core::CodexAuth;
use codex_core::auth::CLIENT_ID;
use codex_core::auth::OPENAI_API_KEY_ENV_VAR;
use codex_core::auth::login_with_api_key;
use codex_core::auth::logout;
use codex_core::config::Config;
use codex_core::config::ConfigOverrides;
use codex_login::{ServerOptions, run_login_server, build_auth_url_only};
use codex_login::run_login_server;
use codex_protocol::mcp_protocol::AuthMode;
use std::env;
use std::io;
use std::path::PathBuf;

pub async fn login_with_chatgpt(codex_home: PathBuf, originator: String) -> std::io::Result<()> {
    let opts = ServerOptions::new(codex_home, CLIENT_ID.to_string(), originator);

    // opt-out of localhost for headless/remote
    let no_localhost = std::env::var_os("CODEX_NO_LOCALHOST").is_some()
        || std::env::args().any(|a| a == "--no-localhost");

    if no_localhost {
        // Phase-1 headless: don’t bind a local port; just print the URL & tips.
        let auth_url = build_auth_url_only(&opts)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("auth url: {e}")))?;

        eprintln!("\nOpen this URL in any browser to authenticate:\n{}\n", auth_url);
        eprintln!("Headless/remote tips:");
        eprintln!("  • SSH tunnel:  ssh -N -L 127.0.0.1:1455:127.0.0.1:1455 <user@host>");
        eprintln!("  • Or use your IDE’s port forward (codespaces/code-server).");
        eprintln!("Re-run this command after completing login if the tool doesn't auto-detect it.\n");
        // We intentionally *don't* block here; exit with a non-zero so scripts can detect.
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Headless mode: localhost callback disabled; complete login in browser and re-run.",
        ));
    }

    // Default (existing) behavior
    let server = run_login_server(opts)?;
    eprintln!(
        "Starting local login server on http://localhost:{}.\nIf your browser did not open, navigate to this URL to authenticate:\n\n{}",
        server.actual_port, server.auth_url,
    );
    
    if no_localhost {
        server.block_until_done().await
    }
}

pub async fn run_login_with_chatgpt(cli_config_overrides: CliConfigOverrides) -> ! {
    let config = load_config_or_exit(cli_config_overrides);

    match login_with_chatgpt(
        config.codex_home,
        config.responses_originator_header.clone(),
    )
    .await
    {
        Ok(_) => {
            eprintln!("Successfully logged in");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error logging in: {e}");
            std::process::exit(1);
        }
    }
}

pub async fn run_login_with_api_key(
    cli_config_overrides: CliConfigOverrides,
    api_key: String,
) -> ! {
    let config = load_config_or_exit(cli_config_overrides);

    match login_with_api_key(&config.codex_home, &api_key) {
        Ok(_) => {
            eprintln!("Successfully logged in");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error logging in: {e}");
            std::process::exit(1);
        }
    }
}

pub async fn run_login_status(cli_config_overrides: CliConfigOverrides) -> ! {
    let config = load_config_or_exit(cli_config_overrides);

    match CodexAuth::from_codex_home(
        &config.codex_home,
        config.preferred_auth_method,
        &config.responses_originator_header,
    ) {
        Ok(Some(auth)) => match auth.mode {
            AuthMode::ApiKey => match auth.get_token().await {
                Ok(api_key) => {
                    eprintln!("Logged in using an API key - {}", safe_format_key(&api_key));

                    if let Ok(env_api_key) = env::var(OPENAI_API_KEY_ENV_VAR)
                        && env_api_key == api_key
                    {
                        eprintln!(
                            "   API loaded from OPENAI_API_KEY environment variable or .env file"
                        );
                    }
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Unexpected error retrieving API key: {e}");
                    std::process::exit(1);
                }
            },
            AuthMode::ChatGPT => {
                eprintln!("Logged in using ChatGPT");
                std::process::exit(0);
            }
        },
        Ok(None) => {
            eprintln!("Not logged in");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error checking login status: {e}");
            std::process::exit(1);
        }
    }
}

pub async fn run_logout(cli_config_overrides: CliConfigOverrides) -> ! {
    let config = load_config_or_exit(cli_config_overrides);

    match logout(&config.codex_home) {
        Ok(true) => {
            eprintln!("Successfully logged out");
            std::process::exit(0);
        }
        Ok(false) => {
            eprintln!("Not logged in");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error logging out: {e}");
            std::process::exit(1);
        }
    }
}

fn load_config_or_exit(cli_config_overrides: CliConfigOverrides) -> Config {
    let cli_overrides = match cli_config_overrides.parse_overrides() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing -c overrides: {e}");
            std::process::exit(1);
        }
    };

    let config_overrides = ConfigOverrides::default();
    match Config::load_with_cli_overrides(cli_overrides, config_overrides) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading configuration: {e}");
            std::process::exit(1);
        }
    }
}

fn safe_format_key(key: &str) -> String {
    if key.len() <= 13 {
        return "***".to_string();
    }
    let prefix = &key[..8];
    let suffix = &key[key.len() - 5..];
    format!("{prefix}***{suffix}")
}

#[cfg(test)]
mod tests {
    use super::safe_format_key;

    #[test]
    fn formats_long_key() {
        let key = "sk-proj-1234567890ABCDE";
        assert_eq!(safe_format_key(key), "sk-proj-***ABCDE");
    }

    #[test]
    fn short_key_returns_stars() {
        let key = "sk-proj-12345";
        assert_eq!(safe_format_key(key), "***");
    }
}
