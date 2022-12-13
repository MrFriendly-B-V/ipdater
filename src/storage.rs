use std::collections::HashMap;
use std::path::{Path, PathBuf};
use color_eyre::eyre::{Error, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Deserialize, Serialize)]
pub struct Storage {
    old_ips: HashMap<String, Vec<String>>,
}

impl Storage {
    pub async fn open_with_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            let parent = path.parent()
                .ok_or(Error::msg("Storage path has no parent".into()))?;
            fs::create_dir_all(parent).await?;

            return Ok(Self {
                old_ips: HashMap::default()
            });
        }

        let mut f = fs::File::open(&path).await?;
        let mut buf = Vec::new();

        f.read_to_end(&mut buf).await?;
        let de: Self = serde_json::from_slice(&buf)?;
        Ok(de)
    }

    pub async fn open() -> Result<Self> {
        Self::open_with_path(&PathBuf::from("/var/ipdater/storage.json")).await
    }

    pub async fn write_with_path(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            let parent = path.parent()
                .ok_or(Error::msg("Storage path has no parent".into()))?;
            fs::create_dir_all(parent).await?;
        }

        let ser = serde_json::to_vec_pretty(&self)?;
        let mut f = fs::File::create(path).await?;
        f.write_all(&ser).await?;
        Ok(())
    }

    pub async fn write(&self) -> Result<()> {
        self.write_with_path(&PathBuf::from("/var/ipdater/storage.json")).await
    }
}