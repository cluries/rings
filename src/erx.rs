/// Layouted: 预设好的一些Layout快速方法
/// ResultE<T> = Result<T, Erx>;
/// ResultEX = ResultE<()>;
/// fn smp<T: ToString>(error: T) -> Erx
/// fn amp<T: ToString>(additional: &str) -> impl Fn(T) -> Erx
use crate::conf;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;


lazy_static! {
    static ref APP_SHORT: String = conf::rebit().read().expect("failed read rebit object").short.clone();
}

/// ResultE<T> = Result<T, Erx>;
pub type ResultE<T> = Result<T, Erx>;

/// ResultEX = ResultE<()>;
pub type ResultEX = ResultE<()>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Erx {
    code: LayoutedC,
    message: String,
    extra: Vec<(String, String)>,
}

/// Layouted: Some predefined Layouted methods
pub struct Layouted;

/// Zero
pub static LAYOUTED_C_ZERO: &'static str = "0000";

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
pub fn emp<T: std::error::Error>(error: T) -> Erx {
    let extra = vec![(String::from("ORIGIN"), describe_error(&error))];
    let message = error.to_string();
    Erx { code: Default::default(), message, extra }
}

/// smp: simple convert T: ToString to Erx
pub fn smp<T: ToString>(error: T) -> Erx {
    Erx { code: Default::default(), message: error.to_string(), extra: Vec::new() }
}

/// amp: return a function that convert T: ToString to Erx
pub fn amp<T: ToString>(additional: &str) -> impl Fn(T) -> Erx {
    let additional = additional.to_string();
    move |err: T| Erx { code: Default::default(), message: format!("{} : {}", additional, err.to_string()), extra: Vec::new() }
}

/// Predefined Layouted Code with length 4
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

impl Into<&'static str> for PreL4 {
    fn into(self) -> &'static str {
        self.four()
    }
}

impl Into<String> for PreL4 {
    fn into(self) -> String {
        self.four().to_string()
    }
}

impl Layouted {
    /// fuzz_udf: 模糊未定义错误
    pub fn fuzz_udf(detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::FUZZ.into(), PreL4::UNDF.into(), detail)
    }

    /// fuzz: 模糊错误
    pub fn fuzz(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::FUZZ.into(), category, detail)
    }

    /// common: 通用错误
    pub fn common(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::COMM.into(), category, detail)
    }

    /// middleware: 中间件错误
    pub fn middleware(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::MIDL.into(), category, detail)
    }

    /// service: 服务错误
    pub fn service(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::SERV.into(), category, detail)
    }

    /// model: 模型错误
    pub fn model(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::MODE.into(), category, detail)
    }

    /// action: Action错误
    pub fn action(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::ACTN.into(), category, detail)
    }

    /// task: Task错误
    pub fn task(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::TASK.into(), category, detail)
    }

    pub fn cron(category: &str, detail: &str) -> LayoutedC {
        LayoutedC::new(PreL4::CRON.into(), category, detail)
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
    application: String,
    domain: String,
    category: String,
    detail: String,
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
    pub fn extral_val_d(&self, key: &str, defaults: String) -> String {
        self.extra_val(key).unwrap_or(defaults)
    }

    pub fn extra_mut(&mut self) -> &mut Vec<(String, String)> {
        &mut self.extra
    }

    /// add extra
    /// if key exists, replace value
    pub fn add_extra(&mut self, key: &str, value: &str) {
        for (k, v) in self.extra.iter_mut() {
            if *k == key {
                *v = value.to_string();
            }
        }

        self.extra.push((key.to_string(), value.to_string()));
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

impl Into<String> for Erx {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

impl Into<(String, String)> for Erx {
    fn into(self) -> (String, String) {
        (self.code.into(), self.message)
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

impl Into<String> for LayoutedC {
    fn into(self) -> String {
        self.layout_string()
    }
}

impl Into<bool> for LayoutedC {
    fn into(self) -> bool {
        self.is_okc()
    }
}

impl Into<(String, String, String, String)> for LayoutedC {
    fn into(self) -> (String, String, String, String) {
        (self.application, self.domain, self.category, self.detail)
    }
}

impl Default for LayoutedC {
    fn default() -> Self {
        LayoutedC { application: APP_SHORT.clone(), domain: PreL4::UNDF.into(), category: PreL4::UNDF.into(), detail: PreL4::UNDF.into() }
    }
}

impl From<&str> for LayoutedC {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}

impl From<(&str, &str, &str, &str)> for LayoutedC {
    fn from(value: (&str, &str, &str, &str)) -> Self {
        LayoutedC {
            application: value.0.to_string(),
            domain: value.1.to_string(),
            category: value.2.to_string(),
            detail: value.3.to_string(),
        }
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
