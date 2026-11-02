use loco_rs::{config::Config, Error, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub uploaded_dir: PathBuf,
    pub upload_file_limit: usize,
}

impl Settings {
    pub fn from_config(config: &Config) -> Result<Self> {
        let value = config
            .settings
            .clone()
            .ok_or_else(|| Error::string("media settings are missing"))?;
        let settings: Self =
            serde_json::from_value(value).map_err(|e| Error::string(&format!("invalid media settings. {e}")))?;
        if settings.upload_file_limit == 0 || settings.upload_file_limit.checked_add(1_048_576).is_none() {
            return Err(Error::string(
                "upload file limit must be positive and leave room for multipart overhead",
            ));
        }
        Ok(settings)
    }

    pub fn prepare_media(&self) -> Result<()> {
        if !self.uploaded_dir.exists() {
            return Err(Error::string(&format!(
                "media root does not exist. {}",
                self.uploaded_dir.display()
            )));
        }
        if !self.uploaded_dir.is_dir() {
            return Err(Error::string(&format!(
                "media root is not a directory. {}",
                self.uploaded_dir.display()
            )));
        }
        std::fs::create_dir_all(self.uploaded_dir.join("audio")).map_err(|e| {
            Error::string(&format!(
                "cannot create audio directory in {}. {e}",
                self.uploaded_dir.display()
            ))
        })?;
        Ok(())
    }

    pub fn request_limit(&self) -> usize {
        self.upload_file_limit + 1_048_576
    }
}
