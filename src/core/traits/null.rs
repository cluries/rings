//! Null Trait Implementation
//!
//! 这个模块提供了一个用于表示"空"或"零值"概念的标记 trait。
//!
//! This module provides a marker trait for representing "null" or "zero-value" concepts.

use std::fmt::Debug;
use std::hash::Hash;

/// A trait for types that can represent a null or zero state.
///
/// This trait is used to mark types that have a meaningful null/zero representation
/// and can be used in contexts where null-like behavior is required.
///
/// 这是一个用于表示具有 null 或零状态的类型的 trait。
/// 用于标记那些具有有意义的 null/zero 表示的类型，
/// 并且可以在需要类似 null 行为的上下文中使用。
///
/// # Safety
///
/// Implementors must ensure that the null value is a valid state
/// that doesn't violate type invariants.
///
/// 实现者必须确保 null 值是一个不会违反类型不变式的有效状态。
pub trait NullTrait:
    Clone +
    Debug +
    Send +
    Sync +
    'static
{
    /// Returns the null/zero value for this type.
    ///
    /// This should return a value that represents the concept of "nothing"
    /// or "zero" for the implementing type.
    ///
    /// 返回此类型的 null/zero 值。
    /// 应该返回一个表示"无"或"零"概念的值。
    ///
    /// # Examples
    ///
    /// ```
    /// use rings::core::traits::NullTrait;
    ///
    /// let null_option: Option<i32> = Option::null();
    /// assert_eq!(null_option, None);
    ///
    /// let null_string: String = String::null();
    /// assert_eq!(null_string, "");
    /// ```
    fn null() -> Self;

    /// Checks if this value is considered null.
    ///
    /// Returns true if the value represents a null state, false otherwise.
    ///
    /// 检查该值是否被视为 null。
    /// 如果值表示 null 状态则返回 true，否则返回 false。
    ///
    /// # Examples
    ///
    /// ```
    /// use rings::core::traits::NullTrait;
    ///
    /// let some_value = Some(42);
    /// assert!(!some_value.is_null());
    ///
    /// let none_value: Option<i32> = None;
    /// assert!(none_value.is_null());
    /// ```
    fn is_null(&self) -> bool;

    /// Converts this value to its null representation.
    ///
    /// This method modifies the value in-place to its null state.
    ///
    /// 将此值转换为其 null 表示形式。
    /// 此方法原地修改值为其 null 状态。
    ///
    /// # Examples
    ///
    /// ```
    /// use rings::core::traits::NullTrait;
    ///
    /// let mut value = "hello".to_string();
    /// value.make_null();
    /// assert_eq!(value, "");
    /// ```
    fn make_null(&mut self);

    /// Returns a string representation of the null state.
    ///
    /// This provides a human-readable description of what "null" means
    /// for this type.
    ///
    /// 返回 null 状态的字符串表示形式。
    /// 提供对这种类型而言"null"含义的人类可读描述。
    fn null_description() -> &'static str;

    /// Attempts to convert from a null value of another type.
    ///
    /// This allows safe conversion between different null representations.
    /// Returns None if the conversion is not meaningful.
    ///
    /// 尝试从另一个类型的 null 值转换。
    /// 允许在不同 null 表示之间安全转换。
    /// 如果转换没有意义则返回 None。
    fn from_null<T: NullTrait>(_other: T) -> Option<Self> {
        None
    }
}

/// A trait for types that can be coalesced from null values.
///
/// This trait provides the ability to provide fallback values when
/// dealing with potentially null values.
///
/// 这是一个用于可以从 null 值合并的类型的 trait。
/// 提供在处理可能为 null 的值时提供回退值的能力。
pub trait NullCoalesce: NullTrait {
    /// Coalesces this value to a non-null default if it is null.
    ///
    /// If this value is null, returns the provided default. Otherwise,
    /// returns the value itself.
    ///
    /// 如果此值为 null，则合并到非 null 默认值，否则返回值本身。
    fn coalesce(self, default: Self) -> Self;

    /// Coalesces this value using a closure for the default.
    ///
    /// This is useful when the default value is expensive to compute
    /// and should only be evaluated when needed.
    ///
    /// 使用闭包为默认值进行合并。
    /// 当默认值计算成本高且仅在需要时才计算时很有用。
    fn coalesce_with<F: FnOnce() -> Self>(self, default_fn: F) -> Self;
}

/// Extension trait for Option types to work with NullTrait.
///
/// 为 Option 类型提供与 NullTrait 配合工作的扩展 trait。
pub trait NullExt: NullTrait {
    /// Returns the contained value or the null value if None.
    ///
    /// 如果包含 Some 值则返回，否则返回 null 值。
    fn unwrap_or_null(self) -> Self;

    /// Returns the contained value or computes it from a closure if None.
    ///
    /// 如果包含 Some 值则返回，否则从闭包计算值。
    fn unwrap_or_else_null<F: FnOnce() -> Self>(self, f: F) -> Self;
}

// Implementations for common types

impl<T> NullTrait for Option<T>
where
    T: Clone + Debug + Send + Sync + 'static
{
    fn null() -> Self {
        None
    }

    fn is_null(&self) -> bool {
        self.is_none()
    }

    fn make_null(&mut self) {
        *self = None;
    }

    fn null_description() -> &'static str {
        "No value present"
    }

    fn from_null<U: NullTrait>(_other: U) -> Option<Self> {
        Some(None)
    }
}

impl<T: NullTrait> NullCoalesce for Option<T> {
    fn coalesce(self, default: Self) -> Self {
        self.or(default)
    }

    fn coalesce_with<F: FnOnce() -> Self>(self, default_fn: F) -> Self {
        self.or_else(default_fn)
    }
}

impl<T: NullTrait> NullExt for Option<T> {
    fn unwrap_or_null(self) -> Self {
        self.or_else(|| Some(T::null()))
    }

    fn unwrap_or_else_null<F: FnOnce() -> Self>(self, f: F) -> Self {
        self.or_else(f)
    }
}

impl NullTrait for String {
    fn null() -> Self {
        String::new()
    }

    fn is_null(&self) -> bool {
        self.is_empty()
    }

    fn make_null(&mut self) {
        self.clear();
    }

    fn null_description() -> &'static str {
        "Empty string"
    }
}

impl NullCoalesce for String {
    fn coalesce(self, default: Self) -> Self {
        if self.is_empty() { default } else { self }
    }

    fn coalesce_with<F: FnOnce() -> Self>(self, default_fn: F) -> Self {
        if self.is_empty() { default_fn() } else { self }
    }
}


impl<T> NullTrait for Vec<T>
where
    T: Clone + Debug + Send + Sync + 'static
{
    fn null() -> Self {
        Vec::new()
    }

    fn is_null(&self) -> bool {
        self.is_empty()
    }

    fn make_null(&mut self) {
        self.clear();
    }

    fn null_description() -> &'static str {
        "Empty vector"
    }
}

impl<T: NullTrait> NullCoalesce for Vec<T> {
    fn coalesce(self, default: Self) -> Self {
        if self.is_empty() { default } else { self }
    }

    fn coalesce_with<F: FnOnce() -> Self>(self, default_fn: F) -> Self {
        if self.is_empty() { default_fn() } else { self }
    }
}

// Numeric types implementations

macro_rules! impl_null_trait_for_numeric {
    ($($type:ty),*) => {
        $(impl NullTrait for $type {
            fn null() -> Self {
                0 as $type
            }

            fn is_null(&self) -> bool {
                *self == 0 as $type
            }

            fn make_null(&mut self) {
                *self = 0 as $type;
            }

            fn null_description() -> &'static str {
                "Zero value"
            }

            fn from_null<U: NullTrait>(other: U) -> Option<Self> {
                if other.is_null() {
                    Some(0 as $type)
                } else {
                    None
                }
            }
        }

        impl NullCoalesce for $type {
            fn coalesce(self, default: Self) -> Self {
                if self == 0 as $type { default } else { self }
            }

            fn coalesce_with<F: FnOnce() -> Self>(self, default_fn: F) -> Self {
                if self == 0 as $type { default_fn() } else { self }
            }
        })*
    };
}

impl_null_trait_for_numeric!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

// Float types implementation (without Hash/Eq)
macro_rules! impl_null_trait_for_float {
    ($($type:ty),*) => {
        $(impl NullTrait for $type {
            fn null() -> Self {
                0.0 as $type
            }

            fn is_null(&self) -> bool {
                *self == 0.0 as $type
            }

            fn make_null(&mut self) {
                *self = 0.0 as $type;
            }

            fn null_description() -> &'static str {
                "Zero value"
            }

            fn from_null<U: NullTrait>(other: U) -> Option<Self> {
                if other.is_null() {
                    Some(0.0 as $type)
                } else {
                    None
                }
            }
        }

        impl NullCoalesce for $type {
            fn coalesce(self, default: Self) -> Self {
                if self == 0.0 as $type { default } else { self }
            }

            fn coalesce_with<F: FnOnce() -> Self>(self, default_fn: F) -> Self {
                if self == 0.0 as $type { default_fn() } else { self }
            }
        })*
    };
}

impl_null_trait_for_float!(f32, f64);

// Boolean implementation

impl NullTrait for bool {
    fn null() -> Self {
        false
    }

    fn is_null(&self) -> bool {
        !*self
    }

    fn make_null(&mut self) {
        *self = false;
    }

    fn null_description() -> &'static str {
        "False value"
    }
}

impl NullCoalesce for bool {
    fn coalesce(self, default: Self) -> Self {
        self || default
    }

    fn coalesce_with<F: FnOnce() -> Self>(self, default_fn: F) -> Self {
        self || default_fn()
    }
}

/// A wrapper type that ensures non-null values.
///
/// This type provides compile-time guarantees that the contained value
/// is never null. It's useful for APIs where null values should be
/// explicitly forbidden.
///
/// 一个确保非 null 值的包装类型。
/// 提供编译时保证，确保包含的值永远不会为 null。
/// 对于明确禁止 null 值的 API 很有用。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonNull<T: NullTrait> {
    inner: T,
}

impl<T: NullTrait> NonNull<T> {
    /// Creates a new NonNull value, failing if the value is null.
    ///
    /// 创建一个新的 NonNull 值，如果值为 null 则失败。
    pub fn new(value: T) -> Result<Self, T> {
        if value.is_null() {
            Err(value)
        } else {
            Ok(Self { inner: value })
        }
    }

    /// Creates a NonNull value without checking for null.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the value is not null.
    ///
    /// 不检查 null 创建 NonNull 值。
    /// 调用者必须确保值不为 null。
    pub unsafe fn new_unchecked(value: T) -> Self {
        Self { inner: value }
    }

    /// Gets the contained value.
    ///
    /// 获取包含的值。
    pub fn get(&self) -> &T {
        &self.inner
    }

    /// Gets the contained value as mutable.
    ///
    /// 获取包含的可变值。
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Consumes the NonNull and returns the inner value.
    ///
    /// 消耗 NonNull 并返回内部值。
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Maps the contained value using a function.
    ///
    /// 使用函数映射包含的值。
    pub fn map<U: NullTrait, F: FnOnce(T) -> U>(self, f: F) -> NonNull<U> {
        NonNull { inner: f(self.inner) }
    }
}

impl<T: NullTrait + Default> Default for NonNull<T> {
    fn default() -> Self {
        // Safety: Default for non-null should be non-null
        unsafe { Self::new_unchecked(T::default()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_null_trait() {
        let opt_none: Option<i32> = Option::null();
        assert!(opt_none.is_null());
        assert_eq!(opt_none, None);

        let opt_some = Some(42);
        assert!(!opt_some.is_null());

        let mut opt = Some(42);
        opt.make_null();
        assert!(opt.is_null());
    }

    #[test]
    fn test_string_null_trait() {
        let empty = String::null();
        assert!(empty.is_null());
        assert_eq!(empty, "");

        let non_empty = "hello".to_string();
        assert!(!non_empty.is_null());

        let mut s = "hello".to_string();
        s.make_null();
        assert!(s.is_null());
    }

    #[test]
    fn test_numeric_null_trait() {
        let zero = i32::null();
        assert!(zero.is_null());
        assert_eq!(zero, 0);

        let non_zero = 42;
        assert!(!non_zero.is_null());

        let mut n = 42;
        n.make_null();
        assert!(n.is_null());
    }

    #[test]
    fn test_coalesce() {
        let null_str = String::null();
        let default = "default".to_string();
        let result = null_str.coalesce(default.clone());
        assert_eq!(result, default);

        let non_null = "hello".to_string();
        let result = non_null.coalesce(default.clone());
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_coalesce_with() {
        let null_str = String::null();
        let result = null_str.coalesce_with(|| "computed".to_string());
        assert_eq!(result, "computed");

        let non_null = "hello".to_string();
        let result = non_null.coalesce_with(|| "computed".to_string());
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_non_null() {
        let valid = NonNull::new("hello".to_string()).unwrap();
        assert_eq!(valid.get(), "hello");

        let invalid = NonNull::new(String::null());
        assert!(invalid.is_err());

        let mapped = valid.map(|s| s.len() as i32);
        assert_eq!(mapped.get(), &5);
    }

    #[test]
    fn test_null_ext() {
        let some_val: Option<i32> = Some(42);
        let none_val: Option<i32> = None;

        assert_eq!(some_val.unwrap_or_null(), some_val);
        assert_eq!(none_val.unwrap_or_null(), Some(i32::null()));

        let computed = none_val.unwrap_or_else_null(|| Some(100));
        assert_eq!(computed, Some(100));
    }

    #[test]
    fn test_from_null_conversion() {
        let null_opt: Option<i32> = None;
        let converted = i32::from_null(null_opt);
        assert_eq!(converted, Some(0));

        let non_null_opt = Some(42);
        let converted = i32::from_null(non_null_opt);
        assert_eq!(converted, None);
    }
}
