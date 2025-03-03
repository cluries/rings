use serde::de::DeserializeOwned;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::erx;

#[derive(Debug, Clone)]
pub struct File(pub String);

impl File {
    pub async fn exists(&self) -> bool {
        tokio::fs::try_exists(&self.0).await.ok().map_or(false, |b| b)
    }

    pub async fn is_directory(&self) -> bool {
        tokio::fs::metadata(&self.0).await.ok().map_or(false, |m| m.is_dir())
    }

    pub async fn is_file(&self) -> bool {
        tokio::fs::metadata(&self.0).await.ok().map_or(false, |m| m.is_file())
    }

    pub async fn is_symlink(&self) -> bool {
        tokio::fs::metadata(&self.0).await.ok().map_or(false, |m| m.is_symlink())
    }

    pub async fn len(&self) -> Result<u64, erx::Erx> {
        Ok(tokio::fs::metadata(&self.0).await.map_err(erx::smp)?.len())
    }
}

pub struct Dir(pub String);

impl Dir {
    const BIT_FILE: i32 = 0;
    const BIT_DIR: i32 = 1;
    const BIT_SYMLINK: i32 = 2;

    pub async fn files(&self) -> Result<Vec<String>, erx::Erx> {
        self.dir_contents(1 << Self::BIT_FILE).await
    }

    pub async fn dirs(&self) -> Result<Vec<String>, erx::Erx> {
        self.dir_contents(1 << Self::BIT_DIR).await
    }

    pub async fn symlinks(&self) -> Result<Vec<String>, erx::Erx> {
        self.dir_contents(1 << Self::BIT_SYMLINK).await
    }

    async fn dir_contents(&self, focus: i32) -> Result<Vec<String>, erx::Erx> {
        let mut dir = tokio::fs::read_dir(&self.0).await.map_err(erx::smp)?;
        let mut results: Vec<String> = Vec::new();

        while let Some(entry) = dir.next_entry().await.map_err(erx::smp)? {
            let ft = entry.file_type().await.map_err(erx::smp)?;

            if 1 << Self::BIT_FILE & focus != 0 && ft.is_file() {
                results.push(entry.file_name().to_string_lossy().into_owned());
                continue;
            }

            if 1 << Self::BIT_DIR & focus != 0 && ft.is_dir() {
                results.push(entry.file_name().to_string_lossy().into_owned());
                continue;
            }

            if 1 << Self::BIT_SYMLINK & focus != 0 && ft.is_symlink() {
                results.push(entry.file_name().to_string_lossy().into_owned());
                continue;
            }
        }

        Ok(results)
    }
}


#[derive(Debug, Clone)]
pub struct FileContent(pub String);


impl FileContent {
    pub async fn vec8(&self) -> Result<Vec<u8>, erx::Erx> {
        let mut fd = tokio::fs::File::open(&self.0).await.map_err(erx::smp)?;
        let mut buffer = Vec::new();
        fd.read_to_end(&mut buffer).await.map_err(erx::smp)?;
        Ok(buffer)
    }

    pub async fn utf8_string(&self) -> Result<String, erx::Erx> {
        let v8 = self.vec8().await?;
        Ok(String::from_utf8_lossy(&v8).into_owned())
    }

    pub async fn lines(&self) -> Result<Vec<String>, erx::Erx> {
        let mut fd = tokio::fs::File::open(&self.0).await.map_err(erx::smp)?;
        let mut buffer = String::new();
        fd.read_to_string(&mut buffer).await.map_err(erx::smp)?;
        Ok(buffer.lines().map(|s| s.to_string()).collect())
    }

    pub async fn truncate(&self, size: u64) -> Result<(), erx::Erx> {
        let fd = tokio::fs::File::open(&self.0).await.map_err(erx::smp)?;
        fd.set_len(size).await.map_err(erx::smp)
    }

    pub async fn append(&self, contents: &str) -> Result<(), erx::Erx> {
        let mut fd = tokio::fs::OpenOptions::new().append(true).open(&self.0).await.map_err(erx::smp)?;
        fd.write_all(contents.as_bytes()).await.map_err(erx::smp)?;
        fd.flush().await.map_err(erx::smp)
    }

    pub async fn clear(&self) -> Result<(), erx::Erx> {
        self.truncate(0).await
    }


    pub async fn json<T: DeserializeOwned>(&self) -> Result<T, erx::Erx> {
        let json = self.utf8_string().await?;
        serde_json::from_str(&json).map_err(erx::smp)
    }

    pub async fn write_json<T: serde::Serialize>(&self, obj: &T) -> Result<(), erx::Erx> {
        let json = serde_json::to_string(obj).map_err(erx::smp)?;
        self.write(&json).await
    }

    pub async fn write(&self, contents: &str) -> Result<(), erx::Erx> {
        let mut fd = tokio::fs::File::create(&self.0).await.map_err(erx::smp)?;
        fd.write_all(contents.as_bytes()).await.map_err(erx::smp)?;
        fd.flush().await.map_err(erx::smp)
    }
}
