pub const APP_NAME: &str = "FinSight AI";
pub const VERSION: &str = "0.1.0";

use std::env;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub app_env: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").map_err(|_| String::from("DATABASE_URL is not set"))?;

        let port = env::var("PORT")
            .unwrap_or_else(|_| String::from("3000"))
            .parse::<u16>()
            .map_err(|_| String::from("PORT must be a valid number"))?;

        let app_env = env::var("APP_ENV").unwrap_or_else(|_| String::from("development"));

        Ok(Self {
            database_url,
            port,
            app_env,
        })
    }
}
