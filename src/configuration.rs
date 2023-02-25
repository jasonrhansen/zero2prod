use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        let Self { database_name, .. } = self;
        let connection_string = self.connection_string_without_db();
        format!("{connection_string}/{database_name}")
    }

    pub fn connection_string_without_db(&self) -> String {
        let Self {
            username,
            password,
            port,
            host,
            ..
        } = self;
        format!("postgres://{username}:{password}@{host}:{port}")
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.toml",
            config::FileFormat::Toml,
        ))
        .build()?;

    settings.try_deserialize::<Settings>()
}
