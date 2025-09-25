/// 文件系统操作工具模块
/*
## 1. Is 结构体 - 路径存在性检查
exists() - 检查路径是否存在
dir() - 检查是否为目录
file() - 检查是否为文件
symlink() - 检查是否为符号链接

## 2. Directory 结构体 - 目录操作
files() - 获取目录下的所有文件
dirs() - 获取目录下的所有子目录
symlinks() - 获取目录下的所有符号链接

## 3. Content 结构体 - 文件内容操作
读取操作：
len() - 获取文件大小
head() / head_string() / head_lines() - 读取文件头部
tail() / tail_string() / tail_lines() - 读取文件尾部
vec8() - 读取整个文件为字节向量
utf8_string() - 读取整个文件为字符串
lines() - 读取所有行

写入操作：
write() - 覆盖写入内容
append() - 追加内容到文件末尾
truncate() - 截断文件到指定大小
clear() - 清空文件内容
JSON 操作：

json<T>() - 读取并解析 JSON 文件
write_json<T>() - 序列化对象并写入 JSON 文件

## 4. 工具函数
normalize_path() - 规范化路径
join_path() - 拼接多个路径段
working_dir() - 获取当前工作目录

*/
use serde::de::DeserializeOwned;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::erx::{smp_boxed, ResultBoxedE};
use std::env;
use std::path::{Path, PathBuf};

/// 目录操作结构体
///
/// 用于封装目录路径并提供目录相关的操作方法
///
/// # 使用示例
/// ```rust
/// let dir = Directory("/path/to/directory".to_string());
/// let files = dir.files().await?; // 获取目录下的所有文件
/// ```
#[derive(Debug, Clone)]
pub struct Directory(pub String);

/// 文件内容操作结构体
///
/// 用于封装文件路径并提供文件内容相关的操作方法
///
/// # 使用示例
/// ```rust
/// let content = Content("/path/to/file.txt".to_string());
/// let text = content.utf8_string().await?; // 读取文件内容为字符串
/// ```
#[derive(Debug, Clone)]
pub struct Content(pub String);

/// 文件/目录存在性检查结构体
///
/// 用于检查文件或目录的存在性和类型
///
/// # 使用示例
/// ```rust
/// let is = Is("/path/to/item".to_string());
/// if is.exists().await {
///     if is.file().await {
///         println!("这是一个文件");
///     } else if is.dir().await {
///         println!("这是一个目录");
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Is(pub String);

/// 规范化路径，处理相对路径符号
///
/// 该函数会处理路径中的 `.` 和 `..` 符号，返回规范化后的绝对路径。
///
/// # 功能
/// - 移除路径中的 `.` （当前目录）
/// - 处理 `..` （上级目录）符号
/// - 确保路径的一致性和正确性
///
/// # 参数
/// - `path`: 需要规范化的路径引用
///
/// # 返回值
/// 返回规范化后的 `PathBuf`
///
/// # 使用示例
/// ```rust
/// use std::path::Path;
///
/// let path = Path::new("/home/user/../documents/./file.txt");
/// let normalized = normalize_path(path);
/// // 结果: /home/documents/file.txt
/// ```
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

/// 拼接多个路径段并规范化
///
/// 将多个路径段拼接成一个完整的路径，并自动进行规范化处理。
///
/// # 功能
/// - 将多个路径段按顺序拼接
/// - 自动处理路径分隔符
/// - 调用 `normalize_path` 进行路径规范化
///
/// # 参数
/// - `paths`: 路径段的字符串切片向量
///
/// # 返回值
/// 返回拼接并规范化后的路径字符串
///
/// # 使用示例
/// ```rust
/// let path = join_path(vec!["/home", "user", "../documents", "file.txt"]);
/// // 结果: "/home/documents/file.txt"
/// ```
pub fn join_path(paths: Vec<&str>) -> String {
    let mut merged_path = PathBuf::new();
    for segment in paths {
        merged_path.push(segment);
    }
    normalize_path(&merged_path).to_string_lossy().to_string()
}

/// 获取当前工作目录
///
/// 返回当前进程的工作目录路径。
///
/// # 功能
/// - 获取当前工作目录的绝对路径
/// - 处理可能的错误情况
///
/// # 返回值
/// - `Some(PathBuf)`: 成功获取到工作目录路径
/// - `None`: 获取失败（权限不足或目录不存在等）
///
/// # 使用示例
/// ```rust
/// if let Some(cwd) = working_dir() {
///     println!("当前工作目录: {:?}", cwd);
/// } else {
///     println!("无法获取当前工作目录");
/// }
/// ```
pub fn working_dir() -> Option<PathBuf> {
    env::current_dir().ok()
}

impl Is {
    /// 检查文件或目录是否存在
    ///
    /// # 功能
    /// - 异步检查指定路径的文件或目录是否存在
    /// - 不区分文件类型，只要路径存在就返回 true
    ///
    /// # 返回值
    /// - `true`: 路径存在
    /// - `false`: 路径不存在或检查失败
    ///
    /// # 使用示例
    /// ```rust
    /// let is = Is("/path/to/file".to_string());
    /// if is.exists().await {
    ///     println!("路径存在");
    /// }
    /// ```
    pub async fn exists(&self) -> bool {
        tokio::fs::try_exists(&self.0).await.ok().map_or(false, |b| b)
    }

    /// 检查路径是否为目录
    ///
    /// # 功能
    /// - 异步检查指定路径是否为目录
    /// - 如果路径不存在或不是目录，返回 false
    ///
    /// # 返回值
    /// - `true`: 路径存在且为目录
    /// - `false`: 路径不存在、不是目录或检查失败
    ///
    /// # 使用示例
    /// ```rust
    /// let is = Is("/path/to/directory".to_string());
    /// if is.dir().await {
    ///     println!("这是一个目录");
    /// }
    /// ```
    pub async fn dir(&self) -> bool {
        tokio::fs::metadata(&self.0).await.ok().is_some_and(|m| m.is_dir())
    }

    /// 检查路径是否为文件
    ///
    /// # 功能
    /// - 异步检查指定路径是否为普通文件
    /// - 如果路径不存在或不是文件，返回 false
    ///
    /// # 返回值
    /// - `true`: 路径存在且为文件
    /// - `false`: 路径不存在、不是文件或检查失败
    ///
    /// # 使用示例
    /// ```rust
    /// let is = Is("/path/to/file.txt".to_string());
    /// if is.file().await {
    ///     println!("这是一个文件");
    /// }
    /// ```
    pub async fn file(&self) -> bool {
        tokio::fs::metadata(&self.0).await.ok().is_some_and(|m| m.is_file())
    }

    /// 检查路径是否为符号链接
    ///
    /// # 功能
    /// - 异步检查指定路径是否为符号链接
    /// - 如果路径不存在或不是符号链接，返回 false
    ///
    /// # 返回值
    /// - `true`: 路径存在且为符号链接
    /// - `false`: 路径不存在、不是符号链接或检查失败
    ///
    /// # 使用示例
    /// ```rust
    /// let is = Is("/path/to/symlink".to_string());
    /// if is.symlink().await {
    ///     println!("这是一个符号链接");
    /// }
    /// ```
    pub async fn symlink(&self) -> bool {
        tokio::fs::metadata(&self.0).await.ok().is_some_and(|m| m.is_symlink())
    }
}

impl Directory {
    /// 文件类型位标识常量
    const BIT_FILE: i32 = 0; // 普通文件
    const BIT_DIR: i32 = 1; // 目录
    const BIT_SYMLINK: i32 = 2; // 符号链接

    /// 获取目录下的所有文件名
    ///
    /// # 功能
    /// - 异步遍历目录，返回所有普通文件的名称
    /// - 不包含子目录和符号链接
    /// - 只返回文件名，不包含完整路径
    ///
    /// # 返回值
    /// - `Ok(Vec<String>)`: 文件名列表
    /// - `Err(erx::Erx)`: 目录不存在、权限不足或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let dir = Directory("/path/to/directory".to_string());
    /// match dir.files().await {
    ///     Ok(files) => {
    ///         for file in files {
    ///             println!("文件: {}", file);
    ///         }
    ///     }
    ///     Err(e) => println!("读取目录失败: {:?}", e),
    /// }
    /// ```
    pub async fn files(&self) -> ResultBoxedE<Vec<String>> {
        self.all(1 << Self::BIT_FILE).await
    }

    /// 获取目录下的所有子目录名
    ///
    /// # 功能
    /// - 异步遍历目录，返回所有子目录的名称
    /// - 不包含普通文件和符号链接
    /// - 只返回目录名，不包含完整路径
    ///
    /// # 返回值
    /// - `Ok(Vec<String>)`: 子目录名列表
    /// - `Err(erx::Erx)`: 目录不存在、权限不足或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let dir = Directory("/path/to/directory".to_string());
    /// match dir.dirs().await {
    ///     Ok(dirs) => {
    ///         for subdir in dirs {
    ///             println!("子目录: {}", subdir);
    ///         }
    ///     }
    ///     Err(e) => println!("读取目录失败: {:?}", e),
    /// }
    /// ```
    pub async fn dirs(&self) -> ResultBoxedE<Vec<String>> {
        self.all(1 << Self::BIT_DIR).await
    }

    /// 获取目录下的所有符号链接名
    ///
    /// # 功能
    /// - 异步遍历目录，返回所有符号链接的名称
    /// - 不包含普通文件和子目录
    /// - 只返回符号链接名，不包含完整路径
    ///
    /// # 返回值
    /// - `Ok(Vec<String>)`: 符号链接名列表
    /// - `Err(erx::Erx)`: 目录不存在、权限不足或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let dir = Directory("/path/to/directory".to_string());
    /// match dir.symlinks().await {
    ///     Ok(links) => {
    ///         for link in links {
    ///             println!("符号链接: {}", link);
    ///         }
    ///     }
    ///     Err(e) => println!("读取目录失败: {:?}", e),
    /// }
    /// ```
    pub async fn symlinks(&self) -> ResultBoxedE<Vec<String>> {
        self.all(1 << Self::BIT_SYMLINK).await
    }

    /// 内部方法：根据类型掩码获取目录条目
    ///
    /// # 功能
    /// - 使用位掩码过滤不同类型的文件系统条目
    /// - 支持组合多种类型（如文件+目录）
    ///
    /// # 参数
    /// - `focus`: 类型位掩码，用于指定要获取的条目类型
    ///
    /// # 返回值
    /// - `Ok(Vec<String>)`: 符合条件的条目名列表
    /// - `Err(erx::Erx)`: I/O 错误
    async fn all(&self, focus: i32) -> ResultBoxedE<Vec<String>> {
        let mut dir = tokio::fs::read_dir(&self.0).await.map_err(smp_boxed)?;
        let mut results: Vec<String> = Vec::new();

        while let Some(entry) = dir.next_entry().await.map_err(smp_boxed)? {
            let ft = entry.file_type().await.map_err(smp_boxed)?;
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
    /// 获取文件大小（字节数）
    ///
    /// # 功能
    /// - 异步获取文件的大小信息
    /// - 返回文件的字节数
    ///
    /// # 返回值
    /// - `Ok(u64)`: 文件大小（字节）
    /// - `Err(erx::Erx)`: 文件不存在、权限不足或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let content = Content("/path/to/file.txt".to_string());
    /// match content.len().await {
    ///     Ok(size) => println!("文件大小: {} 字节", size),
    ///     Err(e) => println!("获取文件大小失败: {:?}", e),
    /// }
    /// ```
    pub async fn len(&self) -> ResultBoxedE<u64> {
        Ok(tokio::fs::metadata(&self.0).await.map_err(smp_boxed)?.len())
    }

    /// 读取文件头部指定字节数的内容
    ///
    /// # 功能
    /// - 异步读取文件开头指定字节数的原始数据
    /// - 适用于二进制文件或需要精确字节控制的场景
    ///
    /// # 参数
    /// - `size`: 要读取的字节数
    ///
    /// # 返回值
    /// - `Ok(Vec<u8>)`: 读取到的字节数据
    /// - `Err(erx::Erx)`: 文件不存在、读取失败或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let content = Content("/path/to/file.bin".to_string());
    /// match content.head(1024).await {
    ///     Ok(bytes) => println!("读取了 {} 字节", bytes.len()),
    ///     Err(e) => println!("读取失败: {:?}", e),
    /// }
    /// ```
    pub async fn head(&self, size: usize) -> ResultBoxedE<Vec<u8>> {
        let mut fd = tokio::fs::File::open(&self.0).await.map_err(smp_boxed)?;
        let mut buffer = vec![0; size];
        fd.read_exact(&mut buffer).await.map_err(smp_boxed)?;
        Ok(buffer)
    }

    /// 读取文件头部指定行数的内容
    ///
    /// # 功能
    /// - 异步读取文件开头指定行数的文本内容
    /// - 自动处理不同的换行符格式
    /// - 移除每行末尾的换行符
    ///
    /// # 参数
    /// - `lines`: 要读取的行数
    ///
    /// # 返回值
    /// - `Ok(Vec<String>)`: 读取到的行内容列表
    /// - `Err(erx::Erx)`: 文件不存在、读取失败或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let content = Content("/path/to/file.txt".to_string());
    /// match content.head_lines(10).await {
    ///     Ok(lines) => {
    ///         for (i, line) in lines.iter().enumerate() {
    ///             println!("第 {} 行: {}", i + 1, line);
    ///         }
    ///     }
    ///     Err(e) => println!("读取失败: {:?}", e),
    /// }
    /// ```
    pub async fn head_lines(&self, lines: usize) -> ResultBoxedE<Vec<String>> {
        let fd = tokio::fs::File::open(&self.0).await.map_err(smp_boxed)?;
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
                Err(e) => return Err(smp_boxed(e)),
            }
        }

        Ok(result)
    }

    /// 读取文件头部指定字节数并转换为字符串
    ///
    /// # 功能
    /// - 异步读取文件开头指定字节数的内容
    /// - 自动将字节数据转换为 UTF-8 字符串
    /// - 对于无效的 UTF-8 序列使用替换字符
    ///
    /// # 参数
    /// - `size`: 要读取的字节数
    ///
    /// # 返回值
    /// - `Ok(String)`: 转换后的字符串内容
    /// - `Err(erx::Erx)`: 文件不存在、读取失败或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let content = Content("/path/to/file.txt".to_string());
    /// match content.head_string(500).await {
    ///     Ok(text) => println!("文件开头内容:\n{}", text),
    ///     Err(e) => println!("读取失败: {:?}", e),
    /// }
    /// ```
    pub async fn head_string(&self, size: usize) -> ResultBoxedE<String> {
        let v8 = self.head(size).await?;
        Ok(String::from_utf8_lossy(&v8).into_owned())
    }

    /// 读取文件尾部指定字节数的内容
    ///
    /// # 功能
    /// - 异步读取文件末尾指定字节数的原始数据
    /// - 如果请求的字节数大于文件大小，则读取整个文件
    /// - 适用于日志文件分析或大文件的尾部内容查看
    ///
    /// # 参数
    /// - `size`: 要读取的字节数
    ///
    /// # 返回值
    /// - `Ok(Vec<u8>)`: 读取到的字节数据
    /// - `Err(erx::Erx)`: 文件不存在、读取失败或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let content = Content("/var/log/app.log".to_string());
    /// match content.tail(1024).await {
    ///     Ok(bytes) => {
    ///         let text = String::from_utf8_lossy(&bytes);
    ///         println!("日志尾部内容:\n{}", text);
    ///     }
    ///     Err(e) => println!("读取失败: {:?}", e),
    /// }
    /// ```
    pub async fn tail(&self, size: usize) -> ResultBoxedE<Vec<u8>> {
        let mut fd = tokio::fs::File::open(&self.0).await.map_err(smp_boxed)?;
        let metadata = fd.metadata().await.map_err(smp_boxed)?;
        let file_size = metadata.len();

        if size as u64 > file_size {
            // If requested size is larger than file size, read entire file
            let mut buffer = Vec::new();
            fd.read_to_end(&mut buffer).await.map_err(smp_boxed)?;
            return Ok(buffer);
        }

        // Seek to position where we should start reading
        fd.seek(std::io::SeekFrom::End(-(size as i64))).await.map_err(smp_boxed)?;

        let mut buffer = vec![0; size];
        fd.read_exact(&mut buffer).await.map_err(smp_boxed)?;
        Ok(buffer)
    }

    /// 读取文件尾部指定行数的内容
    ///
    /// # 功能
    /// - 异步读取文件末尾指定行数的文本内容
    /// - 使用高效的分块读取算法，适合处理大文件
    /// - 自动处理不同的换行符格式
    /// - 保持行的正确顺序（最后的行在列表末尾）
    ///
    /// # 参数
    /// - `lines`: 要读取的行数
    ///
    /// # 返回值
    /// - `Ok(Vec<String>)`: 读取到的行内容列表（按文件中的顺序）
    /// - `Err(erx::Erx)`: 文件不存在、读取失败或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let content = Content("/var/log/app.log".to_string());
    /// match content.tail_lines(50).await {
    ///     Ok(lines) => {
    ///         println!("最后 50 行日志:");
    ///         for line in lines {
    ///             println!("{}", line);
    ///         }
    ///     }
    ///     Err(e) => println!("读取失败: {:?}", e),
    /// }
    /// ```
    pub async fn tail_lines(&self, lines: usize) -> ResultBoxedE<Vec<String>> {
        let fd = tokio::fs::File::open(&self.0).await.map_err(smp_boxed)?;
        let file_size = fd.metadata().await.map_err(smp_boxed)?.len();
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
            reader.seek(std::io::SeekFrom::Start(position)).await.map_err(smp_boxed)?;
            let bytes_read = reader.read_exact(&mut buffer[..read_size]).await.map_err(smp_boxed)?;

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

    /// 读取文件尾部指定字节数并转换为字符串
    ///
    /// # 功能
    /// - 异步读取文件末尾指定字节数的内容
    /// - 自动将字节数据转换为 UTF-8 字符串
    /// - 对于无效的 UTF-8 序列使用替换字符
    ///
    /// # 参数
    /// - `size`: 要读取的字节数
    ///
    /// # 返回值
    /// - `Ok(String)`: 转换后的字符串内容
    /// - `Err(erx::Erx)`: 文件不存在、读取失败或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let content = Content("/var/log/app.log".to_string());
    /// match content.tail_string(2048).await {
    ///     Ok(text) => println!("日志尾部内容:\n{}", text),
    ///     Err(e) => println!("读取失败: {:?}", e),
    /// }
    /// ```
    pub async fn tail_string(&self, size: usize) -> ResultBoxedE<String> {
        let v8 = self.tail(size).await?;
        Ok(String::from_utf8_lossy(&v8).into_owned())
    }

    /// 读取整个文件内容为字节向量
    ///
    /// # 功能
    /// - 异步读取文件的全部内容到内存中
    /// - 返回原始字节数据，适用于二进制文件
    /// - 对于大文件要谨慎使用，可能消耗大量内存
    ///
    /// # 返回值
    /// - `Ok(Vec<u8>)`: 文件的完整字节内容
    /// - `Err(erx::Erx)`: 文件不存在、读取失败或内存不足
    ///
    /// # 使用示例
    /// ```rust
    /// let content = Content("/path/to/image.png".to_string());
    /// match content.vec8().await {
    ///     Ok(bytes) => {
    ///         println!("读取了 {} 字节的二进制数据", bytes.len());
    ///         // 可以进一步处理二进制数据
    ///     }
    ///     Err(e) => println!("读取失败: {:?}", e),
    /// }
    /// ```
    pub async fn vec8(&self) -> ResultBoxedE<Vec<u8>> {
        let mut fd = tokio::fs::File::open(&self.0).await.map_err(smp_boxed)?;
        let mut buffer = Vec::new();
        fd.read_to_end(&mut buffer).await.map_err(smp_boxed)?;
        Ok(buffer)
    }

    /// 读取文件所有行内容
    ///
    /// # 功能
    /// - 异步读取文件的全部内容并按行分割
    /// - 自动处理不同的换行符格式（\n, \r\n, \r）
    /// - 移除每行末尾的换行符
    /// - 适用于文本文件的逐行处理
    ///
    /// # 返回值
    /// - `Ok(Vec<String>)`: 文件的所有行内容
    /// - `Err(erx::Erx)`: 文件不存在、读取失败或编码错误
    ///
    /// # 使用示例
    /// ```rust
    /// let content = Content("/path/to/config.txt".to_string());
    /// match content.lines().await {
    ///     Ok(lines) => {
    ///         for (i, line) in lines.iter().enumerate() {
    ///             println!("第 {} 行: {}", i + 1, line);
    ///         }
    ///     }
    ///     Err(e) => println!("读取失败: {:?}", e),
    /// }
    /// ```
    pub async fn lines(&self) -> ResultBoxedE<Vec<String>> {
        let mut fd = tokio::fs::File::open(&self.0).await.map_err(smp_boxed)?;
        let mut buffer = String::new();
        fd.read_to_string(&mut buffer).await.map_err(smp_boxed)?;
        Ok(buffer.lines().map(|s| s.to_string()).collect())
    }

    /// 读取整个文件内容为 UTF-8 字符串
    ///
    /// # 功能
    /// - 异步读取文件的全部内容并转换为字符串
    /// - 自动处理 UTF-8 编码，对无效序列使用替换字符
    /// - 适用于文本文件的完整内容读取
    ///
    /// # 返回值
    /// - `Ok(String)`: 文件的完整文本内容
    /// - `Err(erx::Erx)`: 文件不存在、读取失败或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let content = Content("/path/to/document.txt".to_string());
    /// match content.utf8_string().await {
    ///     Ok(text) => {
    ///         println!("文件内容:\n{}", text);
    ///         println!("字符数: {}", text.chars().count());
    ///     }
    ///     Err(e) => println!("读取失败: {:?}", e),
    /// }
    /// ```
    pub async fn utf8_string(&self) -> ResultBoxedE<String> {
        let v8 = self.vec8().await?;
        Ok(String::from_utf8_lossy(&v8).into_owned())
    }

    /// 截断文件到指定大小
    ///
    /// # 功能
    /// - 异步将文件截断到指定的字节大小
    /// - 如果指定大小小于当前文件大小，多余部分将被删除
    /// - 如果指定大小大于当前文件大小，文件将被扩展（用零填充）
    ///
    /// # 参数
    /// - `size`: 目标文件大小（字节）
    ///
    /// # 返回值
    /// - `Ok(())`: 截断操作成功
    /// - `Err(erx::Erx)`: 文件不存在、权限不足或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let content = Content("/path/to/file.txt".to_string());
    /// // 截断文件到 1KB
    /// match content.truncate(1024).await {
    ///     Ok(()) => println!("文件已截断到 1024 字节"),
    ///     Err(e) => println!("截断失败: {:?}", e),
    /// }
    /// ```
    pub async fn truncate(&self, size: u64) -> ResultBoxedE<()> {
        let fd = tokio::fs::File::open(&self.0).await.map_err(smp_boxed)?;
        fd.set_len(size).await.map_err(smp_boxed)
    }

    /// 写入内容到文件（覆盖模式）
    ///
    /// # 功能
    /// - 异步将字符串内容写入文件
    /// - 如果文件不存在，将创建新文件
    /// - 如果文件已存在，将完全覆盖原有内容
    /// - 自动将字符串转换为 UTF-8 字节并写入
    ///
    /// # 参数
    /// - `contents`: 要写入的字符串内容
    ///
    /// # 返回值
    /// - `Ok(())`: 写入操作成功
    /// - `Err(erx::Erx)`: 权限不足、磁盘空间不足或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let content = Content("/path/to/output.txt".to_string());
    /// let text = "Hello, World!\n这是一些文本内容。";
    /// match content.write(text).await {
    ///     Ok(()) => println!("文件写入成功"),
    ///     Err(e) => println!("写入失败: {:?}", e),
    /// }
    /// ```
    pub async fn write(&self, contents: &str) -> ResultBoxedE<()> {
        let mut fd = tokio::fs::File::create(&self.0).await.map_err(smp_boxed)?;
        fd.write_all(contents.as_bytes()).await.map_err(smp_boxed)?;
        fd.flush().await.map_err(smp_boxed)
    }

    /// 追加内容到文件末尾
    ///
    /// # 功能
    /// - 异步将字符串内容追加到文件末尾
    /// - 如果文件不存在，操作将失败
    /// - 不会覆盖原有内容，只在末尾添加新内容
    /// - 适用于日志文件或需要保留历史内容的场景
    ///
    /// # 参数
    /// - `contents`: 要追加的字符串内容
    ///
    /// # 返回值
    /// - `Ok(())`: 追加操作成功
    /// - `Err(erx::Erx)`: 文件不存在、权限不足或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let content = Content("/var/log/app.log".to_string());
    /// let log_entry = format!("[{}] 应用启动\n", chrono::Utc::now());
    /// match content.append(&log_entry).await {
    ///     Ok(()) => println!("日志追加成功"),
    ///     Err(e) => println!("追加失败: {:?}", e),
    /// }
    /// ```
    pub async fn append(&self, contents: &str) -> ResultBoxedE<()> {
        let mut fd = tokio::fs::OpenOptions::new().append(true).open(&self.0).await.map_err(smp_boxed)?;
        fd.write_all(contents.as_bytes()).await.map_err(smp_boxed)?;
        fd.flush().await.map_err(smp_boxed)
    }

    /// 清空文件内容
    ///
    /// # 功能
    /// - 异步清空文件的所有内容
    /// - 文件大小将变为 0 字节
    /// - 文件本身不会被删除，只是内容被清空
    /// - 等同于调用 `truncate(0)`
    ///
    /// # 返回值
    /// - `Ok(())`: 清空操作成功
    /// - `Err(erx::Erx)`: 文件不存在、权限不足或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// let content = Content("/tmp/temp_file.txt".to_string());
    /// match content.clear().await {
    ///     Ok(()) => println!("文件已清空"),
    ///     Err(e) => println!("清空失败: {:?}", e),
    /// }
    /// ```
    pub async fn clear(&self) -> ResultBoxedE<()> {
        self.truncate(0).await
    }

    /// 读取 JSON 文件并反序列化为指定类型
    ///
    /// # 功能
    /// - 异步读取文件内容并解析为 JSON 对象
    /// - 支持任何实现了 `DeserializeOwned` trait 的类型
    /// - 自动处理 UTF-8 编码和 JSON 解析
    ///
    /// # 类型参数
    /// - `T`: 目标反序列化类型，必须实现 `DeserializeOwned`
    ///
    /// # 返回值
    /// - `Ok(T)`: 成功解析的对象
    /// - `Err(erx::Erx)`: 文件不存在、读取失败或 JSON 解析错误
    ///
    /// # 使用示例
    /// ```rust
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Config {
    ///     name: String,
    ///     port: u16,
    ///     debug: bool,
    /// }
    ///
    /// let content = Content("/path/to/config.json".to_string());
    /// match content.json::<Config>().await {
    ///     Ok(config) => {
    ///         println!("配置名称: {}", config.name);
    ///         println!("端口: {}", config.port);
    ///     }
    ///     Err(e) => println!("读取配置失败: {:?}", e),
    /// }
    /// ```
    pub async fn json<T: DeserializeOwned>(&self) -> ResultBoxedE<T> {
        let json = self.utf8_string().await?;
        serde_json::from_str(&json).map_err(smp_boxed)
    }

    /// 将对象序列化为 JSON 并写入文件
    ///
    /// # 功能
    /// - 将任何可序列化的对象转换为 JSON 格式
    /// - 异步写入到指定文件（覆盖模式）
    /// - 自动处理 JSON 序列化和文件写入
    ///
    /// # 类型参数
    /// - `T`: 源对象类型，必须实现 `serde::Serialize`
    ///
    /// # 参数
    /// - `obj`: 要序列化的对象引用
    ///
    /// # 返回值
    /// - `Ok(())`: 序列化和写入成功
    /// - `Err(erx::Erx)`: 序列化失败、权限不足或其他 I/O 错误
    ///
    /// # 使用示例
    /// ```rust
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Config {
    ///     name: String,
    ///     port: u16,
    ///     debug: bool,
    /// }
    ///
    /// let config = Config {
    ///     name: "MyApp".to_string(),
    ///     port: 8080,
    ///     debug: true,
    /// };
    ///
    /// let content = Content("/path/to/config.json".to_string());
    /// match content.write_json(&config).await {
    ///     Ok(()) => println!("配置文件保存成功"),
    ///     Err(e) => println!("保存失败: {:?}", e),
    /// }
    /// ```
    pub async fn write_json<T: serde::Serialize>(&self, obj: &T) -> ResultBoxedE<()> {
        let json = serde_json::to_string(obj).map_err(smp_boxed)?;
        self.write(&json).await
    }
}

impl From<Is> for Directory {
    fn from(is: Is) -> Self {
        Directory(is.0)
    }
}

/// 类型转换实现：Directory -> Is
///
/// 允许将 `Directory` 结构体转换为 `Is` 结构体，
/// 用于在目录操作后检查路径属性。
///
/// # 使用示例
/// ```rust
/// let dir = Directory("/path/to/dir".to_string());
/// let is: Is = dir.into();
/// if is.exists().await {
///     println!("目录存在");
/// }
/// ```
impl From<Directory> for Is {
    fn from(dir: Directory) -> Self {
        Is(dir.0)
    }
}

/// 类型转换实现：Is -> Content
///
/// 允许将 `Is` 结构体转换为 `Content` 结构体，
/// 用于在检查路径存在性后进行文件内容操作。
///
/// # 使用示例
/// ```rust
/// let is = Is("/path/to/file.txt".to_string());
/// if is.file().await {
///     let content: Content = is.into();
///     let text = content.utf8_string().await?;
/// }
/// ```
impl Into<Content> for Is {
    fn into(self) -> Content {
        Content(self.0)
    }
}

/// 类型转换实现：Content -> Is
///
/// 允许将 `Content` 结构体转换为 `Is` 结构体，
/// 用于在文件操作后检查路径属性。
///
/// # 使用示例
/// ```rust
/// let content = Content("/path/to/file.txt".to_string());
/// let is: Is = content.into();
/// if is.exists().await {
///     println!("文件存在");
/// }
/// ```
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
