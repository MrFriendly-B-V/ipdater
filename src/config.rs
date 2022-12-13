use std::path::{Path, PathBuf};
use color_eyre::eyre::{Error, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub zones: Vec<ZoneConfig>
}

#[derive(Deserialize, Serialize)]
pub struct ZoneConfig {
    pub domains: Vec<String>,
    pub credentials: ZoneAuth,
}

#[derive(Deserialize, Serialize)]
pub struct ZoneAuth {
    pub key: String,
}

impl Config {
    pub async fn open() -> Result<Self> {
        let dir = PathBuf::from("/etc/ipdater");
        if !dir.exists() {
            fs::create_dir_all(&dir).await?;
        }

        let file_path = dir.join("config.json");
        if !file_path.exists() {
            let ser = serde_json::to_vec_pretty(&Self::example())?;
            let mut f = fs::File::create(&file_path).await?;
            f.write_all(&ser).await?;

            return Err(Error::msg("No config file exists, a default has been generated".into()));
        }

        Self::open_with_path(&file_path).await
    }

    fn example() -> Self {
        Self {
            zones: vec![
                ZoneConfig {
                    credentials: ZoneAuth {
                        key: "my_example_key".into(),
                    },
                    domains: vec![
                        "example.com".into(),
                        "foo.example.com".into()
                    ]
                }
            ]
        }
    }

    pub async fn open_with_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(Error::msg(format!("Config file at {path:?} does not exist")));
        }

        let mut f = fs::File::open(path).await?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).await?;

        let this = serde_json::from_slice(&buf)?;
        Ok(this)
    }
}
