# Signator 中间件配置指南

Signator 中间件现在支持灵活的配置选项，允许您精确控制中间件的行为，包括优先级、应用条件、HTTP 方法过滤和路径匹配模式。

## 配置选项

### 1. 优先级 (Priority)

设置中间件的执行优先级，数值越大优先级越高。

```rust
let config = SignatorConfig::new()
    .priority(100);
```

### 2. 自定义应用逻辑 (Apply)

提供自定义函数来决定中间件是否应该处理特定请求。这是最高优先级的过滤条件。

```rust
let config = SignatorConfig::new()
    .apply(|parts| {
        // 只对包含特定头部的请求应用签名验证
        parts.headers.contains_key("x-require-signature")
    });
```

### 3. HTTP 方法过滤 (Methods)

指定中间件应该处理或忽略的 HTTP 方法。

```rust
let config = SignatorConfig::new()
    .include_method(HttpMethod::POST)
    .include_method(HttpMethod::PUT)
    .exclude_method(HttpMethod::GET);
```

### 4. 路径匹配模式 (Patterns)

使用各种模式匹配来过滤请求路径。

#### 前缀匹配
```rust
let config = SignatorConfig::new()
    .include_prefix("/api/", true)  // 区分大小写
    .exclude_prefix("/public/", false); // 不区分大小写
```

#### 后缀匹配
```rust
let config = SignatorConfig::new()
    .include_suffix(".json", true)
    .exclude_suffix(".html", false);
```

#### 包含匹配
```rust
let config = SignatorConfig::new()
    .include_contains("admin", true)
    .exclude_contains("public", false);
```

#### 正则表达式匹配
```rust
let config = SignatorConfig::new()
    .include_regex(r"^/api/v\d+/.*$")
    .exclude_regex(r".*\.(css|js|png|jpg)$");
```

### 5. 随机数生命周期 (Nonce Lifetime)

设置随机数的有效期（秒）。

```rust
let config = SignatorConfig::new()
    .nonce_lifetime(600); // 10分钟
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

let config = SignatorConfig::new()
    .priority(100)
    .nonce_lifetime(300);

let signator = Signator::with_config(
    "redis://localhost:6379",
    key_loader,
    config
);
```

### API 端点保护

```rust
let config = SignatorConfig::new()
    .priority(200)
    .include_method(HttpMethod::POST)
    .include_method(HttpMethod::PUT)
    .include_method(HttpMethod::DELETE)
    .include_prefix("/api/", true)
    .exclude_contains("public", false);

let signator = Signator::with_config(
    "redis://localhost:6379",
    key_loader,
    config
);
```

### 复杂的自定义逻辑

```rust
let config = SignatorConfig::new()
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

let signator = Signator::with_config(
    "redis://localhost:6379",
    key_loader,
    config
);
```

### 带后门的配置

```rust
let config = SignatorConfig::new()
    .priority(50)
    .include_method(HttpMethod::POST);

let signator = Signator::with_config_and_backdoor(
    "redis://localhost:6379",
    key_loader,
    config,
    "dev_skip_token".to_string()
);
```

## 构建器模式

所有配置方法都返回 `Self`，支持链式调用：

```rust
let config = SignatorConfig::new()
    .priority(100)
    .nonce_lifetime(600)
    .include_method(HttpMethod::POST)
    .include_prefix("/api/", true)
    .exclude_suffix(".html", false)
    .apply(|parts| {
        // 自定义逻辑
        true
    });
```

## 注意事项

1. **性能考虑**：复杂的自定义应用逻辑可能影响性能，建议保持逻辑简单高效。

2. **正则表达式缓存**：正则表达式模式会被自动缓存，重复使用相同模式不会有性能损失。

3. **配置验证**：确保配置的逻辑是合理的，避免创建永远不会匹配的条件。

4. **调试**：可以使用 `{:?}` 格式化打印配置来调试匹配逻辑。