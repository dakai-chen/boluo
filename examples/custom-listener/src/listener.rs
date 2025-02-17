use std::path::{Path, PathBuf};

use boluo::BoxError;
use boluo::listener::Listener;
use tokio::fs::{File, ReadDir};

pub struct FileListener {
    idir: ReadDir,
    odir: PathBuf,
}

impl FileListener {
    pub async fn new(idir: impl AsRef<Path>, odir: impl Into<PathBuf>) -> Result<Self, BoxError> {
        Ok(Self {
            idir: tokio::fs::read_dir(idir).await?,
            odir: odir.into(),
        })
    }

    async fn next_file_path(&mut self) -> Result<Option<PathBuf>, BoxError> {
        while let Some(entry) = self.idir.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }
}

impl Listener for FileListener {
    type IO = File;
    type Addr = ();
    type Error = BoxError;

    async fn accept(&mut self) -> Result<(Self::IO, Self::Addr), Self::Error> {
        let Some(path) = self.next_file_path().await? else {
            std::future::pending().await
        };

        let mut temp = self.odir.clone();
        temp.push(path.file_name().unwrap());

        tokio::fs::create_dir_all(&self.odir).await?;
        tokio::fs::copy(path, &temp).await?;

        Ok((File::options().read(true).write(true).open(temp).await?, ()))
    }
}
