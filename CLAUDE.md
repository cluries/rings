# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

**Rings** is a comprehensive Rust backend application framework built on Axum, SeaORM, and Redis. It provides a "batteries-included" foundation for building modern web services with a clean, modular architecture.

## Project Structure

This is a Cargo workspace with four main crates:

- **`rings`** (main): Core framework containing web, service, model, and configuration layers
- **`ringm`** (macros): Procedural macros for reducing boilerplate code
- **`ringb`** (build): CLI utility for project maintenance and dependency management
- **`rexamples`**: Example applications and testing code

## Common Commands

### Building and Testing
```bash
# Build all workspace members
cargo build

# Build with release mode
cargo build --release

# Run tests
cargo test

# Run tests for specific crate
cargo test -p rings
cargo test -p ringm

# Check code
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Running Applications
```bash
# Run main rings application
cargo run

# Run ringb build tool
cargo run -p ringb

# Run examples
cargo run -p rexamples
```

## Architecture

### Core Framework (rings)

The main framework follows a layered architecture:

**Configuration System** (`src/conf.rs`):
- Uses `config` crate with layered configuration (config.yml → {mode}.yml → local.yml → environment variables)
- Supports multiple environments via `REBT_RUN_MODE` (development/production/testing)
- Configuration accessed via `GetDefault::`, `GetOption::`, and `Has::` static methods
- Main config structure: `Rebit` containing web, model, log, and extends sections

**Web Layer** (`src/web/`):
- Built on Axum with middleware support
- `Web` struct manages HTTP servers with graceful shutdown
- Route registration via `web_route_merge!` macro
- Built-in middleware manager for JWT, rate limiting, etc.
- Lua action support for dynamic route handling

**Service Layer** (`src/service.rs`):
- Sophisticated service manager with dependency injection
- Services implement `ServiceTrait` with lifecycle methods
- Thread-safe service access via `Arc<RwLock<Box<dyn ServiceTrait>>>`
- Macros `with_service_read!` and `with_service_write!` for safe service access
- Global shared service manager via `ServiceManager::shared()`

**Model Layer** (`src/model/`):
- Database abstraction supporting PostgreSQL and Redis
- SeaORM integration with connection pooling
- Shared database connections via `shared_must()` and `shared()`
- Migration support with automatic schema management

**Application Builder** (`src/app.rs`):
- `AppBuilder` pattern for constructing applications
- Fluent API: `new() → use_model() → use_web() → use_scheduler() → build()`
- Automatic module registration and lifecycle management

### Macros (ringm)

Provides procedural macros to reduce boilerplate:

- `#[service]`: Marks structs as services with automatic trait implementations
- `seaorm_mo!`: Generates SeaORM entity definitions
- `migrate_*!`: Database migration macros
- `#[default_any]`: Automatic `AnyTrait` implementations

### Key Framework Features

**State Management**:
- `RingsApplication` manages registered modules with lifecycle states
- `RingState` enum tracks module states (Initializing → Ready → Terminating → Terminated)
- Thread-safe state transitions via `Arc<RwLock<RingState>>`

**Error Handling**:
- Custom `Erx` error type with source tracking
- Consistent error propagation using `?` operator
- Error conversion utilities in `erx` module

**Utilities** (`src/tools/`):
- Comprehensive utility modules: AI, audio, captcha, crypto, file I/O, HTTP client, image processing, JSON, Lua scripting, random, validation, etc.
- Lua scripting integration with `mlua` crate

**Scheduler** (`src/scheduler.rs`):
- Cron-based job scheduling with `tokio-cron-scheduler`
- Integration with service layer for scheduled tasks

## Configuration

The framework uses YAML configuration files:

1. `config/config.yml` - Base configuration
2. `config/{mode}.yml` - Environment-specific overrides
3. `config/local.yml` - Local development overrides
4. Environment variables with `REBT_` prefix

Key configuration sections:
- `web`: HTTP server settings (port, bind, middleware)
- `model`: Database backends (PostgreSQL, Redis)
- `log`: Logging configuration
- `extends`: Custom extension values

## Development Patterns

### Service Implementation
```rust
#[derive(Default)]
struct MyService {}

impl ServiceTrait for MyService {
    fn name(&self) -> &'static str {
        "my_service"
    }
    
    fn initialize(&mut self) {
        // Service initialization
    }
    
    fn release(&mut self) {
        // Cleanup resources
    }
    
    fn ready(&self) -> bool {
        true
    }
    
    fn schedules(&self) -> Vec<Job> {
        vec![] // Return scheduled jobs
    }
}
```

### Application Setup
```rust
let app = AppBuilder::new("my_app")
    .use_model().await
    .use_web(vec![
        web_reconfig_simple("api", || { vec![my_routes()] })
    ]).await
    .use_scheduler().await
    .build();
```

### Service Access
```rust
let shared = ServiceManager::shared().await;
let result = with_service_read!(shared, MyService, service, {
    service.do_something()
});
```

## Testing

The framework includes comprehensive test support:
- Test configuration loading via `tests_load.yml`
- Integration tests in `tests/` directory
- Service manager testing utilities
- Mock configuration for test environments

## Build System

The workspace uses shared dependencies in `Cargo.toml` workspace.dependencies section. The `ringb` utility helps manage workspace dependencies and can consolidate dependencies from member crates into the shared workspace configuration.