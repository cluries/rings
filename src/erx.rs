/// Layouted: 预设好的一些Layout快速方法
/// ResultE<T> = Result<T, Erx>;
/// ResultEX = ResultE<()>;
/// fn smp<T: ToString>(error: T) -> Erx
/// fn amp<T: ToString>(additional: &str) -> impl Fn(T) -> Erx
use crate::conf;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Display;


lazy_static! {
    static ref APP_SHORT: String = conf::rebit().read().expect("failed read rebit object").short.clone();
}


/// Zero
pub static LAYOUTED_C_ZERO: &'static str = "0000";

/// ResultE<T> = Result<T, Erx>;
pub type ResultE<T> = Result<T, Erx>;

/// ResultEX = ResultE<()>;
pub type ResultEX = ResultE<()>;



/// Layouted: Some predefined Layouted methods
pub struct Layouted;


pub fn describe_error(e: &dyn std::error::Error) -> String {
    let mut description = e.to_string();
    let mut current = e.source();
    while let Some(source) = current {
        description.push_str(&format!("\nCaused by: {}", source));
        current = source.source();
    }
    description
}


/// emp
/// emp: error message processor - 将标准错误类型转换为Erx错误类型
/// emp函数的作用是将任何实现了std::error::Error trait的错误转换为Erx错误类型：
/// - 接受一个实现了std::error::Error trait的错误参数
/// - 使用describe_error函数获取完整的错误链描述，并将其作为额外信息存储在extra字段中
/// - 将错误的主要消息（error.to_string()）作为message字段
/// - 使用默认的错误代码（LayoutedC::default()）
/// - 在extra字段中添加"ORIGIN"键，值为完整的错误链描述
/// 适用于需要保留原始错误完整信息的场景，特别是当错误具有复杂的因果链时
/// 与smp函数的区别在于：emp专门处理Error类型并保留错误链信息，而smp只是简单的字符串转换
/// 
/// # 示例
/// ```
/// let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
/// let erx = emp(io_error);
/// // erx.extra 中会包含 ("ORIGIN", "完整的错误描述包括错误链")
/// ```
pub fn emp<T: std::error::Error>(error: T) -> Erx {
    let extra = vec![(String::from("ORIGIN"), describe_error(&error))];
    let message = error.to_string();
    Erx { code: Default::default(), message, extra }
}

/// smp: simple convert T: ToString to Erx
/// smp函数的作用是将任何实现了ToString trait的类型简单转换为Erx错误类型
/// 这是一个便捷函数，用于快速创建基本的错误对象：
/// - 使用默认的错误代码（LayoutedC::default()）
/// - 将输入参数转换为字符串作为错误消息
/// - 不包含任何额外信息（extra字段为空）
/// 适用于需要快速创建简单错误的场景，不需要复杂的错误分类或额外上下文信息
pub fn smp<T: ToString>(error: T) -> Erx {
    Erx { code: Default::default(), message: error.to_string(), extra: Vec::new() }
}

/// amp: return a function that convert T: ToString to Erx
/// amp: 返回一个函数，该函数将T: ToString转换为Erx，并在错误消息前添加额外的上下文信息
/// amp函数的作用是创建一个错误转换闭包，用于为错误消息添加上下文前缀：
/// - 接受一个additional参数作为错误消息的前缀
/// - 返回一个闭包，该闭包可以将任何实现了ToString trait的类型转换为Erx
/// - 生成的错误消息格式为: "{additional} : {原始错误消息}"
/// - 使用默认的错误代码（LayoutedC::default()）
/// - 不包含任何额外信息（extra字段为空）
/// 适用于需要为一系列相关错误添加统一上下文信息的场景，比如在特定模块或函数中批量处理错误时
/// 
/// # 示例
/// ```
/// let db_error_converter = amp("Database operation failed");
/// let error = db_error_converter("Connection timeout");
/// // 生成的错误消息为: "Database operation failed : Connection timeout"
/// ```
pub fn amp<T: ToString>(additional: &str) -> impl Fn(T) -> Erx {
    let additional = additional.to_string();
    move |err: T| Erx { code: Default::default(), message: format!("{} : {}", additional, err.to_string()), extra: Vec::new() }
}

/// Predefined Layouted Code with length 4
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PreL4 {
    /// Fuzz: 模糊错误
    FUZZ,
    /// Common: 通用错误
    COMM,
    /// Middleware: 中间件错误
    MIDL,
    /// Service: 服务错误
    SERV,
    /// Model: 模型错误
    MODE,
    /// Action: Action错误
    ACTN,
    /// Undefined: 未定义错误
    UNDF,
    /// Task: Task错误
    TASK,
    /// Cron: Cron错误
    CRON,
    ///
    OTHE,
}

impl PreL4 {
    pub fn four(&self) -> &'static str {
        match self {
            PreL4::FUZZ => "FUZZ",
            PreL4::COMM => "COMM",
            PreL4::MIDL => "MIDL",
            PreL4::SERV => "SERV",
            PreL4::MODE => "MODE",
            PreL4::ACTN => "ACTN",
            PreL4::UNDF => "UNDF",
            PreL4::TASK => "TASK",
            PreL4::CRON => "CRON",
            PreL4::OTHE => "OTHE",
        }
    }

    pub fn from_str(s: &str) -> Option<PreL4> {
        match s.to_uppercase().as_str() {
            "FUZZ" => Some(PreL4::FUZZ),
            "COMM" => Some(PreL4::COMM),
            "MIDL" => Some(PreL4::MIDL),
            "SERV" => Some(PreL4::SERV),
            "MODE" => Some(PreL4::MODE),
            "ACTN" => Some(PreL4::ACTN),
            "UNDF" => Some(PreL4::UNDF),
            "TASK" => Some(PreL4::TASK),
            "CRON" => Some(PreL4::CRON),
            "OTHE" => Some(PreL4::OTHE),
            _ => None,
        }
    }

    pub fn layoutc(&self, category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(self.four(), category, detail)
    }
}

impl Display for PreL4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.four())
    }
}

impl From<&str> for PreL4 {
    fn from(s: &str) -> Self {
        PreL4::from_str(s).unwrap_or(PreL4::OTHE)
    }
}
    
impl From<PreL4> for String {
    fn from(value: PreL4) -> Self {
        value.four().to_string()
    }
}


 

impl Layouted {
    /// fuzz_udf: 模糊未定义错误
    pub fn fuzz_udf(detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::FUZZ.four(), PreL4::UNDF.four(), detail)
    }

    /// fuzz: 模糊错误
    pub fn fuzz(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::FUZZ.four(), category, detail)
    }

    /// common: 通用错误
    pub fn common(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::COMM.four(), category, detail)
    }

    /// middleware: 中间件错误
    pub fn middleware(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::MIDL.four(), category, detail)
    }

    /// service: 服务错误
    pub fn service(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::SERV.four(), category, detail)
    }

    /// model: 模型错误
    pub fn model(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::MODE.four(), category, detail)
    }

    /// action: Action错误
    pub fn action(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::ACTN.four(), category, detail)
    }

    /// task: Task错误
    pub fn task(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::TASK.four(), category, detail)
    }

    pub fn cron(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::CRON.four(), category, detail)
    }
}

/// Code code format
/// aaaa-xxxx-yyyy-zzzz
///
///    aaaa : 应用标示，建议4位长度
///    xxxx : 单词字母，建议4位长度，用于区分大类（功能域）
///    yyyy : 字母或者数字，建议4位长度，用于区分子类
///    zzzz : 字母或者数字，建议4位长度，具体错误
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LayoutedC {
    pub application: String,
    pub domain: String,
    pub category: String,
    pub detail: String,
}


impl LayoutedC {
    pub fn okay() -> LayoutedC {
        LayoutedC {
            application: APP_SHORT.clone(),
            domain: LAYOUTED_C_ZERO.into(),
            category: LAYOUTED_C_ZERO.into(),
            detail: LAYOUTED_C_ZERO.into(),
        }
    }

    pub fn new(domain: &str, category: &str, detail: &str) -> LayoutedC {
        LayoutedC { application: APP_SHORT.clone(), domain: domain.into(), category: category.into(), detail: detail.into() }
    }

    pub fn is_okc(&self) -> bool {
        self.domain.replace("0", "").len() == 0 && self.category.replace("0", "").len() == 0 && self.detail.replace("0", "").len() == 0
    }

    pub fn layout_string(&self) -> String {
        format!("{}-{}-{}-{}", self.application, self.domain, self.category, self.detail)
    }

    pub fn get_app(&self) -> &str {
        &self.application
    }
    pub fn get_domain(&self) -> &str {
        &self.domain
    }

    pub fn get_category(&self) -> &str {
        &self.category
    }

    pub fn get_detail(&self) -> &str {
        &self.detail
    }
}
 
impl Default for LayoutedC {
    fn default() -> Self {
        LayoutedC { application: APP_SHORT.clone(), domain: PreL4::UNDF.into(), category: PreL4::UNDF.into(), detail: PreL4::UNDF.into() }
    }
}

 

impl From<LayoutedC> for String {
    fn from(value: LayoutedC) -> Self {
        value.layout_string()
    }
}

impl From<LayoutedC> for bool {
    fn from(value: LayoutedC) -> Self {
        value.is_okc()
    }
}
 

impl From<String> for LayoutedC {
    fn from(value: String) -> Self {
        let mut c = LayoutedC::default();
        let parts: Vec<&str> = value.split("-").collect();
        if let Some(application) = parts.get(0) {
            c.application = application.to_string();
        }
        if let Some(domain) = parts.get(1) {
            c.domain = domain.to_string();
        }
        if let Some(category) = parts.get(2) {
            c.category = category.to_string();
        }
        if let Some(detail) = parts.get(3) {
            c.detail = detail.to_string();
        }
        c
    }
}

impl From<(String, String, String, String)> for LayoutedC {
    fn from(value: (String, String, String, String)) -> Self {
        LayoutedC { application: value.0, domain: value.1, category: value.2, detail: value.3 }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Erx {
    code: LayoutedC,
    message: String,
    extra: Vec<(String, String)>,
}

impl Erx {
    pub fn new(message: &str) -> Erx {
        Erx { code: Default::default(), message: message.to_string(), extra: Vec::new() }
    }

    pub fn code(&self) -> LayoutedC {
        self.code.clone()
    }

    pub fn code_mut(&mut self) -> &mut LayoutedC {
        &mut self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn message_string(&self) -> String {
        self.message.clone()
    }

    pub fn message_mut(&mut self) -> &mut String {
        &mut self.message
    }

    pub fn description(&self) -> String {
        let mut description = self.code.layout_string();
        description.push_str(" ");
        description.push_str(&self.message);
        if self.extra.is_empty() {
            return description;
        }

        description.push_str(" { ");

        self.extra.iter().for_each(|x| {
            description.push_str(&format!("{}={} ,", x.0, x.1));
        });

        description.remove(description.len() - 1);
        description.push_str(" }");

        description
    }

    /// get extra
    pub fn extra(&self) -> &Vec<(String, String)> {
        &self.extra
    }

    /// get extra value, if not exists, return None
    pub fn extra_val(&self, key: &str) -> Option<String> {
        if self.extra.len() < 1 {
            return None;
        }

        self.extra.iter().find(|e| e.0.eq(key)).and_then(|e| Some(e.1.clone()))
    }

    /// get extra value, if not exists, return defaults
    pub fn extra_val_d(&self, key: &str, defaults: String) -> String {
        self.extra_val(key).unwrap_or(defaults)
    }

    pub fn extra_mut(&mut self) -> &mut Vec<(String, String)> {
        &mut self.extra
    }

    /// add extra
    /// if key exists, replace value
    pub fn add_extra(&mut self, key: &str, value: &str) -> &mut Self {
        for (k, v) in self.extra.iter_mut() {
            if *k == key {
                *v = value.to_string();
                return self;
            }
        }

        self.extra.push((key.to_string(), value.to_string()));
        self
    }

    /// get extra and convert to HashMap
    /// if have same key, the last value will be used
    pub fn extra_map(&self) -> HashMap<String, String> {
        let m: HashMap<String, String> = HashMap::from_iter(self.extra.clone());
        m
    }
}

impl Display for Erx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap_or_default())
    }
}

impl Default for Erx {
    fn default() -> Self {
        Erx { code: Default::default(), message: Default::default(), extra: Default::default() }
    }
}
 
impl<T> Into<Result<T, Erx>> for Erx {
    fn into(self) -> Result<T, Erx> {
        Err(self)
    }
}

 

impl From<Infallible> for Erx {
    fn from(_: Infallible) -> Self {
        Erx::default()
    }
}
 
impl From<&str> for Erx {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

impl From<String> for Erx {
    fn from(str: String) -> Erx {
        if str.is_empty() {
            return Erx::default();
        }

        serde_json::from_str(&str).unwrap_or_else(|_| Erx::new(&str))
    }
}

impl From<Box<dyn std::error::Error>> for Erx {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Erx::new(&value.to_string())
    }
}

impl From<(&str, &str)> for Erx {
    fn from((code, message): (&str, &str)) -> Self {
        (code.to_string(), message.to_string()).into()
    }
}

impl From<(String, String)> for Erx {
    fn from((code, message): (String, String)) -> Erx {
        let code: LayoutedC = code.into();
        Erx { code, message, extra: Default::default() }
    }
}

impl<T: ToString + Default> From<Vec<T>> for Erx {
    fn from(value: Vec<T>) -> Self {
        let len = value.len();
        if len == 0 {
            Erx::default()
        } else if len == 1 {
            value[0].to_string().into()
        } else if len == 2 {
            (value[0].to_string(), value[1].to_string()).into()
        } else {
            let code = value[0].to_string();
            let message = value[1].to_string();

            let mut iter = value.into_iter();
            let mut extra: Vec<(String, String)> = Vec::new();
            while let Some(first) = iter.next() {
                let second = iter.next().unwrap_or_default();
                extra.push((first.to_string(), second.to_string()));
            }

            Erx { code: code.into(), message, extra }
        }
    }
}
