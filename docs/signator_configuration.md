# Signator 中间件配置指南

Signator 中间件现在支持灵活的配置选项，允许您精确控制中间件的行为，包括优先级、应用条件、HTTP 方法过滤和路径匹配模式。

## 配置选项

### 1. 创建配置

创建 SignatorConfig 需要提供必需的 key_loader 和 redis_url：

```rust
let key_loader = Arc::new(|user_id: String| -> Pin<Box<dyn Future<Output = Result<String, Erx>> + Send>> {
    Box::pin(async move {
        // 从数据库或其他存储中加载用户的签名密钥
        Ok(format!("secret_key_for_{}", user_id))
    })
});

let config = SignatorConfig::new(key_loader, "redis://localhost:6379".to_string());
```

### 2. 优先级 (Priority)

设置中间件的执行优先级，数值越大优先级越高。

```rust
let config = SignatorConfig::new(key_loader, redis_url)
    .priority(100);
```

### 3. 自定义应用逻辑 (Apply)

提供自定义函数来决定中间件是否应该处理特定请求。这是最高优先级的过滤条件。

```rust
let config = SignatorConfig::new(key_loader, redis_url)
    .apply(|parts| {
        // 只对包含特定头部的请求应用签名验证
        parts.headers.contains_key("x-require-signature")
    });
```

### 4. HTTP 方法过滤 (Methods)

指定中间件应该处理或忽略的 HTTP 方法。

```rust
let config = SignatorConfig::new(key_loader, redis_url)
    .include_method(HttpMethod::POST)
    .include_method(HttpMethod::PUT)
    .exclude_method(HttpMethod::GET);
```

### 5. 路径匹配模式 (Patterns)

使用各种模式匹配来过滤请求路径。

#### 前缀匹配
```rust
let config = SignatorConfig::new(key_loader, redis_url)
    .include_prefix("/api/", true)  // 区分大小写
    .exclude_prefix("/public/", false); // 不区分大小写
```

#### 后缀匹配
```rust
let config = SignatorConfig::new(key_loader, redis_url)
    .include_suffix(".json", true)
    .exclude_suffix(".html", false);
```

#### 包含匹配
```rust
let config = SignatorConfig::new(key_loader, redis_url)
    .include_contains("admin", true)
    .exclude_contains("public", false);
```

#### 正则表达式匹配
```rust
let config = SignatorConfig::new(key_loader, redis_url)
    .include_regex(r"^/api/v\d+/.*$")
    .exclude_regex(r".*\.(css|js|png|jpg)$");
```

### 6. 随机数生命周期 (Nonce Lifetime)

设置随机数的有效期（秒）。

```rust
let config = SignatorConfig::new(key_loader, redis_url)
    .nonce_lifetime(600); // 10分钟
```

### 7. 后门 (Backdoor)

设置开发时使用的后门令牌，用于跳过签名验证。

```rust
let config = SignatorConfig::new(key_loader, redis_url)
    .backdoor("dev_skip_token".to_string());
```

## 过滤优先级

中间件的过滤逻辑按以下优先级执行：

1. **apply** - 如果提供了自定义应用逻辑，直接使用其返回值
2. **methods** - HTTP 方法过滤
3. **patterns** - 路径匹配模式

只有当前一级过滤条件通过（或未设置）时，才会检查下一级条件。

## 使用示例

### 基本配置

```rust
use rings::web::middleware::signator::{Signator, SignatorConfig};

let config = SignatorConfig::new(key_loader, "redis://localhost:6379".to_string())
    .priority(100)
    .nonce_lifetime(300);

let signator = Signator::new(config).expect("Failed to create Signator");
```

### API 端点保护

```rust
let config = SignatorConfig::new(key_loader, "redis://localhost:6379".to_string())
    .priority(200)
    .include_method(HttpMethod::POST)
    .include_method(HttpMethod::PUT)
    .include_method(HttpMethod::DELETE)
    .include_prefix("/api/", true)
    .exclude_contains("public", false);

let signator = Signator::new(config).expect("Failed to create Signator");
```

### 复杂的自定义逻辑

```rust
let config = SignatorConfig::new(key_loader, "redis://localhost:6379".to_string())
    .apply(|parts| {
        let path = parts.uri.path();
        let method = parts.method.as_str();
        
        // 管理员路径总是需要签名
        if path.starts_with("/admin/") {
            return true;
        }
        
        // GET 请求不需要签名
        if method == "GET" {
            return false;
        }
        
        // API 路径需要签名，但公共路径除外
        path.starts_with("/api/") && !path.contains("/public/")
    });

let signator = Signator::new(config).expect("Failed to create Signator");
```

### 带后门的配置

```rust
let config = SignatorConfig::new(key_loader, "redis://localhost:6379".to_string())
    .priority(50)
    .include_method(HttpMethod::POST)
    .backdoor("dev_skip_token".to_string());

let signator = Signator::new(config).expect("Failed to create Signator");
```

## 构建器模式

所有配置方法都返回 `Self`，支持链式调用：

```rust
let config = SignatorConfig::new(key_loader, "redis://localhost:6379".to_string())
    .priority(100)
    .nonce_lifetime(600)
    .include_method(HttpMethod::POST)
    .include_prefix("/api/", true)
    .exclude_suffix(".html", false)
    .backdoor("dev_token".to_string())
    .apply(|parts| {
        // 自定义逻辑
        true
    });

let signator = Signator::new(config).expect("Failed to create Signator");
```

## 配置验证

Signator 会在创建时自动验证配置的完整性和有效性：

```rust
let config = SignatorConfig::new(key_loader, "redis://localhost:6379".to_string());

// 配置验证会在 new() 时自动执行
match Signator::new(config) {
    Ok(signator) => {
        // 配置有效，可以使用
    },
    Err(error) => {
        // 配置错误，查看错误信息
        eprintln!("Configuration error: {}", error);
    }
}
```

### 必需的配置项

- `key_loader`: 密钥加载器函数（构造函数参数）
- `redis_url`: Redis 连接 URL（构造函数参数）

### 验证规则

- `nonce_lifetime` 必须为正数且不超过 24 小时
- `redis_url` 必须以 `redis://` 或 `rediss://` 开头
- Redis 连接必须能够成功建立

## 错误处理

配置错误会返回 `ConfigError` 类型的错误：

```rust
use rings::web::middleware::signator::Error;

match Signator::new(config) {
    Ok(signator) => { /* 使用 signator */ },
    Err(Error::ConfigError(msg)) => {
        eprintln!("Configuration error: {}", msg);
    },
    Err(other_error) => {
        eprintln!("Other error: {}", other_error);
    }
}
```

## 注意事项

1. **性能考虑**：复杂的自定义应用逻辑可能影响性能，建议保持逻辑简单高效。

2. **正则表达式缓存**：正则表达式模式会被自动缓存，重复使用相同模式不会有性能损失。

3. **配置验证**：配置会在创建 Signator 时自动验证，确保所有必需项都已设置。

4. **调试**：可以使用 `{:?}` 格式化打印配置来调试匹配逻辑。

5. **向后兼容**：仍然支持旧的构造函数，但推荐使用新的配置方式。