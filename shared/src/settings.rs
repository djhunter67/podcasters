use config::{Config, File};
use serde::Deserialize;
use std::{env, path};
use tracing::instrument;

/// Global setting for exposing all preconfigured variables
#[derive(Deserialize, Clone)]
pub struct Settings {
    pub application: Application,
    pub debug: bool,
    pub mongo: Mongo,
    pub redis: Redis,
    pub secret: Secret,
    pub frontend_url: String,
}

#[derive(Deserialize, Clone)]
pub struct Secret {
    pub secret_key: String,
    pub token_expiration: i64,
    pub hmac_secret: String,
}

/// Redis setting for the entire application
#[derive(Deserialize, Clone, Debug)]
pub struct Redis {
    pub uri: String,
    pub key: String,
    pub pool_size: u32,
    pub pool_max_open: u32,
    pub pool_max_idle: u32,
    pub pool_timeout_seconds: u32,
    pub pool_expire_seconds: u32,
}

/// Mongo setting for the entire application
#[derive(Deserialize, Clone, Debug)]
pub struct Mongo {
    pub uri: String,
    pub db: String,
    pub collection: String,
    pub require_auth: bool,
    pub pool_size: u8,
    pub connection_timeout: u8,
}

/// Application's specific settings to expose `port`,
/// `host`, `protocol`, and possible URL of the application
/// during and after development
#[derive(Deserialize, Clone)]
pub struct Application {
    pub port: u32,
    pub host: String,
    pub base_url: String,
    pub protocol: String,
}

/// The possible runtime environment for our application
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use either 'production' or 'development'."
            )),
        }
    }
}

/// # Result
///   - Returns a `Result` of `Settings` if successful
/// # Errors
///   - Returns a `config::ConfigError` if there is an error loading the settings
/// # Panics
///   - Panics if the current directory cannot be determined
/// # Notes
///   - Multipurpose function that helps detect the current environment the application
///     is running using the `APP_ENVIRONMENT` environment variable.
///
/// \\\
/// ``APP_ENVIRONMENT`` = development | production.
/// \\\
///
/// After detection, it loads the appropriate .yaml file
/// then it loads the environment variable that overrides whatever is set in the .yaml file.
/// For this to work, you the environment variable MUST be in uppercase and starts with `APP`,
/// a "_" separator then the category of settings,
/// followed by "__" separator,  and then the variable.
/// # Example
///   - ``APP__APPLICATION_PORT=5001`` for "port" to be set as "5001"
#[instrument(name = "Get Settings", level = "info", target = "Project Settings")]
pub fn get() -> Result<Settings, config::ConfigError> {
    // eprintln because during startup `tracing` doesn't work

    eprintln!("Getting the system config settings");
    // let manual_path = std::env::current_dir();
    // eprint!("\n\nThe home dir: {manual_path:#?}");

    let base_path: path::PathBuf = match std::env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Unable to procure the current dir: {err}");
            panic!("Application cannot find the current directory to load the settings")
        }
    };
    eprintln!("The base path is {base_path:?}");
    let setting_directory = base_path.join("settings");
    eprintln!("The setting directory is {setting_directory:?}");

    let environment: Environment = match env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
    {
        Ok(env) => env,
        Err(err) => return Err(config::ConfigError::Message(err)),
    };
    eprintln!("The environment is {:#?}", environment.as_str());
    let environment_filename = format!("{}.yaml", environment.as_str());

    eprintln!(
        "Building the settings for the {} environment",
        environment.as_str()
    );
    let settings: Config = match Config::builder()
        .add_source(File::from(setting_directory.join("base.yaml")))
        .add_source(File::from(setting_directory.join(environment_filename)))
        // Add in settings from environment variables (with a prefix of APP and '__' as seperator)
        // e.g `APP_APPLICATION__PORT_5001 would set `Setting.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()
    {
        Ok(settings) => {
            eprintln!(
                "Successfully loaded the settings: {:#?}",
                settings
                    .clone()
                    .try_deserialize::<Settings>()
                    .expect("Failed to deserialize the settings")
                    .mongo
            );

            settings
        }
        Err(err) => {
            eprintln!("Failed to load the settings");
            return Err(err);
        }
    };

    settings.try_deserialize::<Settings>()
}
