// 基础类型枚举
#[derive(Debug, Clone)]
pub enum PrimitiveType {
    // 布尔
    Boolean(bool),
    
    // 整数
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    
    // 浮点
    F32(f32),
    F64(f64),
    
    // 字符和字符串
    Char(char),
    String(String),
    
    // 复合类型
    Array(Vec<PrimitiveType>),
    Tuple(Vec<PrimitiveType>),
    
    // 特殊类型
    Unit,
    None,
}

impl PrimitiveType {
    // 创建新实例
    pub fn new_bool(v: bool) -> Self {
        Self::Boolean(v)
    }
    
    pub fn new_i32(v: i32) -> Self {
        Self::I32(v)
    }
    
    pub fn new_string<S: Into<String>>(v: S) -> Self {
        Self::String(v.into())
    }
    
    // 类型判断
    pub fn is_numeric(&self) -> bool {
        matches!(self,
            Self::I8(_) | Self::I16(_) | Self::I32(_) | Self::I64(_) | Self::I128(_) |
            Self::U8(_) | Self::U16(_) | Self::U32(_) | Self::U64(_) | Self::U128(_) |
            Self::F32(_) | Self::F64(_)
        )
    }
    
    pub fn is_integer(&self) -> bool {
        matches!(self,
            Self::I8(_) | Self::I16(_) | Self::I32(_) | Self::I64(_) | Self::I128(_) |
            Self::U8(_) | Self::U16(_) | Self::U32(_) | Self::U64(_) | Self::U128(_)
        )
    }
    
    pub fn is_float(&self) -> bool {
        matches!(self, Self::F32(_) | Self::F64(_))
    }
    
    // 转换方法
    pub fn to_string(&self) -> String {
        match self {
            Self::Boolean(b) => b.to_string(),
            Self::I32(n) => n.to_string(),
            Self::String(s) => s.clone(),
            Self::Char(c) => c.to_string(),
            Self::None => "None".to_string(),
            Self::Unit => "()".to_string(),
            _ => format!("{:?}", self)
        }
    }
    
    // Option-like 方法
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
    
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
    
    // 数组操作
    pub fn push_to_array(&mut self, value: PrimitiveType) -> Result<(), &'static str> {
        match self {
            Self::Array(vec) => {
                vec.push(value);
                Ok(())
            }
            _ => Err("Not an array type")
        }
    }
    
    pub fn array_len(&self) -> Result<usize, &'static str> {
        match self {
            Self::Array(vec) => Ok(vec.len()),
            _ => Err("Not an array type")
        }
    }
}

// 实现基本运算符
impl std::ops::Add for PrimitiveType {
    type Output = Result<Self, &'static str>;
    
    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::I32(a), Self::I32(b)) => Ok(Self::I32(a + b)),
            (Self::F64(a), Self::F64(b)) => Ok(Self::F64(a + b)),
            (Self::String(a), Self::String(b)) => Ok(Self::String(a + &b)),
            _ => Err("Unsupported operation")
        }
    }
}
