# ringm

## 宏定义列表

本 crate 中定义的 proc_macro 宏如下：

| 宏名称                | 类型                                            | 说明                            |
| --------------------- | ----------------------------------------------- | ------------------------------- |
| migrate_using_macros  | #[proc_macro]                                   | 迁移相关宏，见 migrate.rs       |
| migrate_make_migrator | #[proc_macro]                                   | 迁移器生成宏，见 migrate.rs     |
| service               | #[proc_macro_attribute]                         | 服务标记宏，见 service.rs       |
| serviced              | #[proc_macro]                                   | 服务扩展宏，见 service.rs       |
| service_resolve       | #[proc_macro] (feature: serivce_macro_use_func) | 服务解析宏，见 service.rs       |
| default_any           | #[proc_macro_attribute]                         | 默认 Any 宏，见 any.rs          |
| seaorm_mo             | #[proc_macro]                                   | SeaORM 实体相关宏，见 seaorm.rs |

> 具体用法和参数请参考各模块源码。
