use envie::Envie;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::{fs, io, path::Path, sync::LazyLock};
use tokio::sync::OnceCell;

pub static DB: OnceCell<SqlitePool> = OnceCell::const_new();

#[derive(Debug, Deserialize)]
pub struct Configs {
    pub server: Server,
    pub log: Log,
    pub database: DataBase,
    pub jwt: Jwt,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    #[serde(default)] // Skip YAML deserialization for this field
    pub address: String,
    pub cors_allow_origin: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DataBase {
    #[serde(default)] // Skip YAML deserialization for this field
    pub database_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Log {
    pub filter_level: String,
    pub with_ansi: bool,
    pub to_stdout: bool,
    pub directory: String,
    pub file_name: String,
    pub rolling: String,
}

#[derive(Debug, Deserialize)]
pub struct Jwt {
    #[serde(default)] // Skip YAML deserialization for this field
    pub jwt_secret: String,
    pub jwt_exp: i64,
}

const CONFIG_FILE: &str = "config/config.yml";

pub static CFG: LazyLock<Configs> = LazyLock::new(|| match Configs::init() {
    Ok(config) => config,
    Err(e) => panic!("Failed to initialize configuration: {}", e),
});

impl Configs {
    pub fn init() -> io::Result<Self> {
        // Load .env file
        let env = Envie::load().map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to load .env file: {}", e),
            )
        })?;

        // Load YAML config
        let cfg_contents = read_file_to_string(CONFIG_FILE)?;
        let mut config: Configs = serde_yml::from_str(&cfg_contents).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to parse configuration file: {}", e),
            )
        })?;

        // Override values from .env file
        config.server.address = env
            .get("SERVER_ADDRESS")
            .unwrap_or_else(|| "127.0.0.1:8080".to_string());

        config.database.database_url = env
            .get("DATABASE_URL")
            .unwrap_or_else(|| "sqlite:data/demo.db".to_string());

        config.jwt.jwt_secret = env
            .get("JWT_SECRET")
            .unwrap_or_else(|| "default_secret".to_string());

        Ok(config)
    }
}

fn read_file_to_string<P>(path: P) -> io::Result<String>
where
    P: AsRef<Path> + Copy,
{
    fs::read_to_string(path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to read file {}: {}", path.as_ref().display(), e),
        )
    })
}

pub async fn init_db_conn() {
    DB.get_or_init(|| async {
        SqlitePool::connect(&CFG.database.database_url)
            .await
            .expect("Database connection failed.")
    })
    .await;
}
