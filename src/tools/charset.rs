/// 字符集枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    /// UTF-8 字符集 
    UTF8,
    /// GB2312 中文字符集
    GB2312,
    /// GBK 中文字符集
    GBK,
    /// GB18030 中文字符集
    GB18030,
    /// Big5 繁体中文字符集
    BIG5,
    /// Unicode 字符集
    UNICODE,
    /// ISO-8859-1 西欧字符集
    ISO8859_1,
}

impl Charset {
    /// 获取字符集名称
    pub fn name(&self) -> &str {
        match self {
            Charset::UTF8 => "UTF-8",
            Charset::GB2312 => "GB2312",
            Charset::GBK => "GBK",
            Charset::GB18030 => "GB18030",
            Charset::BIG5 => "Big5",
            Charset::UNICODE => "Unicode",
            Charset::ISO8859_1 => "ISO-8859-1",
        }
    }


}

impl std::fmt::Display for Charset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}


