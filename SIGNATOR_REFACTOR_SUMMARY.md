# Signator 重构总结

## 更改概述

将 Signator 中间件的 `key_loader`、`backdoor` 和 `redis_url` 从 `Signator` 结构体移动到 `SignatorConfig` 配置结构体中，并添加了配置验证机制，实现更好的配置管理和关注点分离。

## 主要更改

### 1. SignatorConfig 结构体更新

**之前:**
```rust
pub struct SignatorConfig {
    pub priority: i32,
    pub apply: Option<Arc<dyn Fn(&Parts) -> bool + Send + Sync>>,
    pub methods: Option<Arc<Vec<ApplyKind<HttpMethod>>>>,
    pub patterns: Option<Arc<Vec<ApplyKind<Pattern>>>>,
    pub nonce_lifetime: i64,
}
```

**之后:**
```rust
pub struct SignatorConfig {
    pub priority: i32,
    pub apply: Option<Arc<dyn Fn(&Parts) -> bool + Send + Sync>>,
    pub methods: Option<Arc<Vec<ApplyKind<HttpMethod>>>>,
    pub patterns: Option<Arc<Vec<ApplyKind<Pattern>>>>,
    pub nonce_lifetime: i64,
    pub key_loader: KeyLoader,        // 必填字段
    pub backdoor: Option<String>,
    pub redis_url: String,            // 必填字段
}
```

### 2. 构造函数和配置方法更新

更新了 `SignatorConfig` 的构造函数和配置方法：

```rust
impl SignatorConfig {
    /// 创建新的配置，需要提供必填的 key_loader 和 redis_url
    pub fn new(key_loader: KeyLoader, redis_url: String) -> Self {
        // 初始化配置
    }

    /// 设置密钥加载器
    pub fn key_loader(mut self, key_loader: KeyLoader) -> Self {
        self.key_loader = key_loader;
        self
    }

    /// 设置后门
    pub fn backdoor(mut self, backdoor: String) -> Self {
        self.backdoor = Some(backdoor);
        self
    }

    /// 设置 Redis 连接 URL
    pub fn redis_url(mut self, redis_url: String) -> Self {
        self.redis_url = redis_url;
        self
    }

    /// 验证配置是否完整和有效
    pub fn validate(&self) -> Result<(), Error> {
        // 验证配置的合理性（不再需要检查必填项）
    }
}
```

### 3. Signator 结构体简化

**之前:**
```rust
pub struct Signator {
    backdoor: String,
    config: SignatorConfig,
    key_loader: KeyLoader,
    redis_client: redis::Client,
}
```

**之后:**
```rust
pub struct Signator {
    config: SignatorConfig,
    redis_client: redis::Client,
}
```

### 4. 错误类型扩展

添加了 `ConfigError` 来处理配置相关的错误：

```rust
#[derive(Debug)]
pub enum Error {
    /// 配置错误
    ConfigError(String),
    // ... 其他错误类型
}
```

### 5. 构造函数重构

**新的主要构造函数:**
```rust
impl Signator {
    pub fn new(config: SignatorConfig) -> Result<Self, Error>
    pub fn from_config(config: SignatorConfig) -> Result<Self, Error>
}
```

**向后兼容的构造函数:**
```rust
impl Signator {
    pub fn with_key_loader(redis_url: &str, key_loader: KeyLoader) -> Result<Self, Error>
    pub fn with_backdoor(redis_url: &str, key_loader: KeyLoader, backdoor: String) -> Result<Self, Error>
    pub fn with_config(redis_url: &str, config: SignatorConfig) -> Result<Self, Error>
}
```

### 6. 执行逻辑更新

在 `exec` 方法中，现在从配置中获取 `key_loader` 和 `backdoor`：

```rust
// 从配置中获取 key_loader
let key_loader = self.config.key_loader.as_ref()
    .ok_or_else(|| Error::ConfigError("Key loader not configured".to_string()))?;

// 从配置中获取 backdoor
let empty_string = String::new();
let backdoor = self.config.backdoor.as_ref().unwrap_or(&empty_string);
```

Redis 客户端现在从配置中的 `redis_url` 初始化，并在构造时进行验证。

## 使用方式更新

### 新的推荐用法

```rust
use rings::web::middleware::signator::{Signator, SignatorConfig};

let key_loader = Arc::new(|user_id: String| -> Pin<Box<dyn Future<Output = Result<String, Erx>> + Send>> {
    Box::pin(async move {
        Ok(format!("secret_key_for_{}", user_id))
    })
});

let config = SignatorConfig::new(key_loader, "redis://localhost:6379".to_string())
    .priority(100)
    .nonce_lifetime(600)
    .backdoor("dev_token".to_string());

let signator = Signator::new(config).expect("Failed to create Signator");
```

### 向后兼容

现有代码仍然可以工作，但建议迁移到新的 API：

```rust
// 仍然支持，但不推荐
let signator = Signator::with_backdoor("redis://localhost:6379", key_loader, "backdoor".to_string())
    .expect("Failed to create Signator");

// 推荐的新方式
let config = SignatorConfig::new(key_loader, "redis://localhost:6379".to_string())
    .backdoor("backdoor".to_string());
let signator = Signator::new(config).expect("Failed to create Signator");
```

## 更新的文件

1. `src/web/middleware/signator.rs` - 主要重构文件
2. `examples/signator_config.rs` - 更新示例代码
3. `rexamples/web/src/middleware/api/signator.rs` - 更新使用方式
4. `docs/signator_configuration.md` - 更新文档

## 优势

1. **更好的配置管理**: 所有配置项现在都在 `SignatorConfig` 中统一管理
2. **关注点分离**: `Signator` 专注于执行逻辑，配置逻辑分离到 `SignatorConfig`
3. **更灵活的配置**: 可以更容易地组合不同的配置选项
4. **向后兼容**: 保留了旧的构造函数，现有代码无需立即修改
5. **更清晰的 API**: 新的 API 更加直观和一致
6. **配置验证**: 自动验证配置的完整性和有效性，提前发现配置错误
7. **错误处理**: 明确的错误类型，更好的错误信息
8. **Redis 连接管理**: Redis 连接配置统一管理，支持更多连接选项
9. **类型安全**: 必填字段在编译时强制要求，减少运行时错误

## 测试验证

通过创建临时测试文件验证了所有新的 API 都能正常工作，包括：
- 基本配置
- 带后门的配置  
- 向后兼容的构造函数
- 链式配置调用
- 配置验证逻辑
- 错误处理机制

添加了单元测试来验证：
- 配置验证的各种场景
- 错误类型的正确性
- 必需配置项的检查
- Redis URL 格式验证

所有测试都通过，确保重构没有破坏现有功能。