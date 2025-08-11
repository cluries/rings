/// 快速创建 String 对象的宏
///
/// # 示例
/// ```
/// let text = s!("hello world");
/// // 等价于 String::from("hello world")
/// ```
#[macro_export]
macro_rules! s {
    ($s:expr) => {
        String::from($s)
    };
}

/// 将标识符转换为字符串字面量的宏
///
/// # 示例
/// ```
/// let name = ms!(variable_name);
/// // 返回 "variable_name"
/// ```
#[macro_export]
macro_rules! ms {
    ($x:ident) => {
        stringify!($x)
    };
}

/// 将标识符转换为 String 对象的宏
///
/// # 示例
/// ```
/// let name = mst!(variable_name);
/// // 返回 String::from("variable_name")
/// ```
#[macro_export]
macro_rules! mst {
    ($x:ident) => {
        String::from(stringify!($x))
    };
}

/// 将表达式转换为可选字符串 (to optional string)
///
/// # 示例
/// ```
/// let opt_str = tos!(42);
/// // 返回 Some("42".to_string())
///
/// let combined = tos!(1, 2, 3);
/// // 返回 Some("123")
/// ```
#[macro_export]
macro_rules! tos {
    ($e:expr) => {
        Some($e.to_string())
    };
    ($($e:expr),+) => {
        Some(format!("{}", ts_helper!($($e),+)))
    };
}

/// 将表达式转换为字符串 (to string)
///
/// # 示例
/// ```
/// let str_val = ts!(42);
/// // 返回 "42"
///
/// let combined = ts!(1, 2, 3);
/// // 返回 "123"
/// ```
#[macro_export]
macro_rules! ts {
    ($e:expr) => {
        $e.to_string()
    };
    ($($e:expr),+) => {
        format!("{}", ts_helper!($($e),+))
    };
}

/// ts! 宏的辅助宏，用于处理多个参数的递归连接
#[macro_export]
#[allow(unused_macros)]
macro_rules! ts_helper {
    ($e:expr) => {
        $e.to_string()
    };
    ($e:expr, $($rest:expr),+) => {
        format!("{}{}", $e, ts_helper!($($rest),+))
    };
}

/// 检查所有 Option 值是否都为 None
///
/// # 示例
/// ```
/// let opt1: Option<i32> = None;
/// let opt2: Option<i32> = Some(42);
///
/// let result = all_none!(opt1);        // true
/// let result = all_none!(opt1, opt2);  // true (任一为 None 即返回 true)
/// ```
#[macro_export]
macro_rules! all_none {
    ($e:expr) => {
        $e.is_none()
    };
    ($e:expr, $($rest:expr),+) => {
        $e.is_none() || all_none!($($rest),+)
    };
}

/// 将 Result<T, E> 转换为 Result<T, String>
///
/// # 示例
/// ```
/// let result: Result<i32, std::io::Error> = Err(std::io::Error::new(std::io::ErrorKind::Other, "error"));
/// let string_result = result_message!(result);
/// // 返回 Result<i32, String>
/// ```
#[macro_export]
macro_rules! result_message {
    ($s:expr) => {
        match $s {
            Ok(r) => Ok(r),
            Err(err) => Err(err.to_string()),
        }
    };
}

/// 三元运算符宏，类似于其他语言的 condition ? true_value : false_value
///
/// # 示例
/// ```
/// let result = ternary!(x > 0, "positive", "non-positive");
/// // 等价于 if x > 0 { "positive" } else { "non-positive" }
/// ```
#[macro_export]
macro_rules! ternary {
    ($condition:expr, $true_value:expr, $false_value:expr) => {
        if $condition {
            $true_value
        } else {
            $false_value
        }
    };
}

/// 尝试执行表达式，如果失败则记录错误并返回
///
/// # 示例
/// ```
/// fn some_function() {
///     let value = try_or_return!(risky_operation());
///     // 如果 risky_operation() 失败，会记录错误并从函数返回
///     // 否则继续执行
/// }
/// ```
#[macro_export]
macro_rules! try_or_return {
    ($expr:expr) => {
        match $expr {
            Ok(value) => value,
            Err(e) => {
                error!("{}", e);
                return;
            },
        }
    };
}

/// 定义服务函数的宏，用于创建异步服务入口点
///
/// # 示例
/// ```
/// // 创建默认服务函数
/// its_service!();
///
/// // 创建带名称的服务函数
/// its_service!(my_service);
/// ```
#[macro_export]
macro_rules! its_service {
    () => {
        pub async fn its_service() {
            ringm::serviced!();
        }
    };
    ($e:expr) => {
        pub async fn its_service() {
            ringm::serviced!(stringify!($e));
        }
    };
}

/// 调用服务的宏，用于异步调用指定模块的服务函数
///
/// # 示例
/// ```
/// // 调用 UserService 模块的服务函数
/// hey_service!(UserService);
/// // 等价于 UserService::its_service().await
/// ```
#[macro_export]
macro_rules! hey_service {
    ($e:ident) => {{
        $e::its_service().await
    }};
}

/// 导入 SeaORM 常用类型和 trait 的宏
///
/// # 示例
/// ```
/// use_seaorm_min!();
/// // 导入 ActiveModelTrait, ActiveValue, ColumnTrait, Condition,
/// // EntityTrait, IntoActiveModel, QueryFilter, QueryOrder
/// ```
#[macro_export]
macro_rules! use_seaorm_min {
    () => {
        #[allow(unused_imports)]
        use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder};
    };
}
