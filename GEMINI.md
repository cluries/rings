# rings (Core Framework)

`rings` is a highly integrated and well-designed Rust backend application framework. It provides a robust, "batteries-included" foundation for building modern, reliable, and scalable web services.

- **Core Technologies**: Built on top of popular libraries like `Axum` for web services, `SeaORM` for database interaction (PostgreSQL), and `Redis` for caching/messaging.
- **Architecture**: Features a clean, modular, and layered architecture:
    - **Configuration**: A powerful, layered configuration system (`conf.rs`).
    - **Web Layer**: Manages `Axum` routes, middleware, and requests/responses (`web/`).
    - **Service Layer**: A sophisticated service manager (`service.rs`) for handling business logic in a decoupled, service-oriented manner.
    - **Model Layer**: Centralized management of database connections (`model/`).
- **Features**: Includes out-of-the-box support for logging, configuration, database migrations, and scheduled tasks.

# ringm (Productivity Macros)

`ringm` is a companion procedural macro crate for `rings`. Its primary purpose is to significantly reduce boilerplate code and accelerate development.

- **Code Generation**: Provides a rich set of macros to automate common tasks.
- **Key Macros**:
    - `#[service]`: Simplifies the creation of services compatible with the core service manager.
    - `seaorm_*`: A comprehensive suite of macros that streamline the definition of `SeaORM` database entities, relations, and active models.
    - `migrate_*`: Macros to help create and manage database migrations.
    - Utility macros for generating getters, setters, and other boilerplate code.

# ringb (Build & Maintenance Tool)

`ringb` is a small command-line utility designed for project maintenance and build-related tasks.

- **Functionality**: Its main function is to help manage dependencies within the Cargo workspace, for example, by consolidating dependencies from the main `Cargo.toml` into the workspace's shared dependency list.

# rexamples (Examples & Testing)

`rexamples` is a crate that contains practical examples and test cases for the `rings` framework.

- **Purpose**: It serves as a live demonstration of how to use the framework's features, including defining services with `ringm` macros, creating web routes, and integrating different components. It is used during development for testing and serves as a valuable learning resource for new users.
