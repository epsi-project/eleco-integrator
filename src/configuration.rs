use config::{Config, File};
use secrecy::Secret;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub rabbitmq: RabbitMQSettings,
    pub supabase: SupabaseSettings
}

#[derive(serde::Deserialize, Clone)]
pub struct AuthSettings {
    pub username: String,
    pub password: Secret<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct RabbitMQSettings {
    pub protocol: String,
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub auth: AuthSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct SupabaseSettings {
    pub uri: String,
    pub key: Secret<String>,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let config = Config::builder()
        .add_source(File::from(configuration_directory.join("base")).required(true))
        .add_source(File::from(configuration_directory.join(environment.as_str())).required(true))
        .add_source(
            config::Environment::with_prefix("APP")
                .try_parsing(true)
                .separator("_"),
        )
        .build()?;

    config.try_deserialize()
}

pub enum Environment {
    Local,
    Production,
}
impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}
impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
