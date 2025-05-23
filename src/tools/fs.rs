use serde::de::DeserializeOwned;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::erx;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Directory(pub String);

#[derive(Debug, Clone)]
pub struct Content(pub String);

#[derive(Debug, Clone)]
pub struct Is(pub String);

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut stack = Vec::new();
    // 分解路径组件，处理 `..` 和 `.`
    for component in path.components() {
        match component {
            // 根目录：清空栈并保留根目录
            std::path::Component::RootDir => {
                stack.clear();
                stack.push(component);
            },
            // 当前目录：忽略
            std::path::Component::CurDir => {},
            // 上级目录：弹出栈顶元素（如果可能）
            std::path::Component::ParentDir => {
                if let Some(std::path::Component::RootDir) = stack.last() {
                    // 根目录的父目录仍是根目录（Unix 规则）
                } else if !stack.is_empty() {
                    stack.pop();
                }
            },
            // 普通路径组件：直接入栈
            _ => stack.push(component),
        }
    }
    // 将组件重新组合为 PathBuf
    let mut normalized = PathBuf::new();
    for component in stack {
        normalized.push(component.as_os_str());
    }
    normalized
}

pub fn join_path(paths: Vec<&str>) -> String {
    let mut merged_path = PathBuf::new();
    for segment in paths {
        merged_path.push(segment);
    }
    normalize_path(&merged_path).to_string_lossy().to_string()
}

pub fn working_dir() -> Option<PathBuf> {
    env::current_dir().ok()
}

impl Is {
    pub async fn exists(&self) -> bool {
        tokio::fs::try_exists(&self.0).await.ok().map_or(false, |b| b)
    }

    pub async fn dir(&self) -> bool {
        tokio::fs::metadata(&self.0).await.ok().map_or(false, |m| m.is_dir())
    }

    pub async fn file(&self) -> bool {
        tokio::fs::metadata(&self.0).await.ok().map_or(false, |m| m.is_file())
    }

    pub async fn symlink(&self) -> bool {
        tokio::fs::metadata(&self.0).await.ok().map_or(false, |m| m.is_symlink())
    }
}

impl Directory {
    const BIT_FILE: i32 = 0;
    const BIT_DIR: i32 = 1;
    const BIT_SYMLINK: i32 = 2;

    pub async fn files(&self) -> Result<Vec<String>, erx::Erx> {
        self.all(1 << Self::BIT_FILE).await
    }

    pub async fn dirs(&self) -> Result<Vec<String>, erx::Erx> {
        self.all(1 << Self::BIT_DIR).await
    }

    pub async fn symlinks(&self) -> Result<Vec<String>, erx::Erx> {
        self.all(1 << Self::BIT_SYMLINK).await
    }

    async fn all(&self, focus: i32) -> Result<Vec<String>, erx::Erx> {
        let mut dir = tokio::fs::read_dir(&self.0).await.map_err(erx::smp)?;
        let mut results: Vec<String> = Vec::new();

        while let Some(entry) = dir.next_entry().await.map_err(erx::smp)? {
            let ft = entry.file_type().await.map_err(erx::smp)?;
            if (((1 << Self::BIT_FILE) & focus) != 0 && ft.is_file())
                || (((1 << Self::BIT_DIR) & focus) != 0 && ft.is_dir())
                || (((1 << Self::BIT_SYMLINK) & focus) != 0 && ft.is_symlink())
            {
                results.push(entry.file_name().to_string_lossy().into_owned());
            }
        }

        Ok(results)
    }
}

impl Content {
    pub async fn len(&self) -> Result<u64, erx::Erx> {
        Ok(tokio::fs::metadata(&self.0).await.map_err(erx::smp)?.len())
    }

    pub async fn head(&self, size: usize) -> Result<Vec<u8>, erx::Erx> {
        let mut fd = tokio::fs::File::open(&self.0).await.map_err(erx::smp)?;
        let mut buffer = vec![0; size];
        fd.read_exact(&mut buffer).await.map_err(erx::smp)?;
        Ok(buffer)
    }

    pub async fn head_lines(&self, lines: usize) -> Result<Vec<String>, erx::Erx> {
        let fd = tokio::fs::File::open(&self.0).await.map_err(erx::smp)?;
        let mut reader = tokio::io::BufReader::new(fd);
        let mut line = String::new();
        let mut result = Vec::new();
        let mut count = 0;

        while count < lines {
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF reached
                Ok(_) => {
                    result.push(line.trim_end().to_string());
                    count += 1;
                    line.clear();
                },
                Err(e) => return Err(erx::smp(e)),
            }
        }

        Ok(result)
    }

    pub async fn head_string(&self, size: usize) -> Result<String, erx::Erx> {
        let v8 = self.head(size).await?;
        Ok(String::from_utf8_lossy(&v8).into_owned())
    }

    pub async fn tail(&self, size: usize) -> Result<Vec<u8>, erx::Erx> {
        let mut fd = tokio::fs::File::open(&self.0).await.map_err(erx::smp)?;
        let metadata = fd.metadata().await.map_err(erx::smp)?;
        let file_size = metadata.len();

        if size as u64 > file_size {
            // If requested size is larger than file size, read entire file
            let mut buffer = Vec::new();
            fd.read_to_end(&mut buffer).await.map_err(erx::smp)?;
            return Ok(buffer);
        }

        // Seek to position where we should start reading
        fd.seek(std::io::SeekFrom::End(-(size as i64))).await.map_err(erx::smp)?;

        let mut buffer = vec![0; size];
        fd.read_exact(&mut buffer).await.map_err(erx::smp)?;
        Ok(buffer)
    }

    pub async fn tail_lines(&self, lines: usize) -> Result<Vec<String>, erx::Erx> {
        let fd = tokio::fs::File::open(&self.0).await.map_err(erx::smp)?;
        let file_size = fd.metadata().await.map_err(erx::smp)?.len();
        let mut reader = tokio::io::BufReader::new(fd);

        // Use a circular buffer to store the last N lines
        let mut line_buffer = Vec::with_capacity(lines);

        // For very large files, read in chunks from the end
        let chunk_size: usize = (lines / 32).clamp(2, 16) * 1024;

        let mut buffer = vec![0; chunk_size];
        let mut position = file_size;
        let mut found_lines = 0;

        while position > 0 && found_lines < lines {
            let read_size = std::cmp::min(chunk_size, position as usize);
            position = position.saturating_sub(read_size as u64);

            // Seek to the current position
            reader.seek(std::io::SeekFrom::Start(position)).await.map_err(erx::smp)?;
            let bytes_read = reader.read_exact(&mut buffer[..read_size]).await.map_err(erx::smp)?;

            // Convert chunk to string and process lines in reverse
            let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);
            let mut chunk_lines: Vec<&str> = chunk.lines().collect();
            chunk_lines.reverse();

            for line in chunk_lines {
                if found_lines >= lines {
                    break;
                }
                line_buffer.push(line.to_string());
                found_lines += 1;
            }
        }

        // Reverse the lines to maintain correct order
        line_buffer.reverse();
        Ok(line_buffer)
    }

    pub async fn tail_string(&self, size: usize) -> Result<String, erx::Erx> {
        let v8 = self.tail(size).await?;
        Ok(String::from_utf8_lossy(&v8).into_owned())
    }

    pub async fn vec8(&self) -> Result<Vec<u8>, erx::Erx> {
        let mut fd = tokio::fs::File::open(&self.0).await.map_err(erx::smp)?;
        let mut buffer = Vec::new();
        fd.read_to_end(&mut buffer).await.map_err(erx::smp)?;
        Ok(buffer)
    }

    pub async fn lines(&self) -> Result<Vec<String>, erx::Erx> {
        let mut fd = tokio::fs::File::open(&self.0).await.map_err(erx::smp)?;
        let mut buffer = String::new();
        fd.read_to_string(&mut buffer).await.map_err(erx::smp)?;
        Ok(buffer.lines().map(|s| s.to_string()).collect())
    }

    pub async fn utf8_string(&self) -> Result<String, erx::Erx> {
        let v8 = self.vec8().await?;
        Ok(String::from_utf8_lossy(&v8).into_owned())
    }

    pub async fn truncate(&self, size: u64) -> Result<(), erx::Erx> {
        let fd = tokio::fs::File::open(&self.0).await.map_err(erx::smp)?;
        fd.set_len(size).await.map_err(erx::smp)
    }

    pub async fn write(&self, contents: &str) -> Result<(), erx::Erx> {
        let mut fd = tokio::fs::File::create(&self.0).await.map_err(erx::smp)?;
        fd.write_all(contents.as_bytes()).await.map_err(erx::smp)?;
        fd.flush().await.map_err(erx::smp)
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
}

impl Into<Directory> for Is {
    fn into(self) -> Directory {
        Directory(self.0)
    }
}

impl From<Directory> for Is {
    fn from(dir: Directory) -> Self {
        Is(dir.0)
    }
}

impl Into<Content> for Is {
    fn into(self) -> Content {
        Content(self.0)
    }
}

impl From<Content> for Is {
    fn from(content: Content) -> Self {
        Is(content.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::tests::tools as ts;

    #[test]
    fn test_join_path() {
        let c = join_path(vec!["/root/user/work", "../notin", "name/d"]);

        println!("{:?}", c);
    }

    #[tokio::test]
    async fn test_tail_string() {
        let cargo = ts::project_dir().join("Cargo.toml").to_str().unwrap_or_default().to_string();

        println!("{:?}", Content(cargo).tail_lines(2).await.unwrap_or_default());
        // println!("{}", Content(cargo).tail_string(120).await.unwrap());
    }
}
