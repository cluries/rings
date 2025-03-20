use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};
use crate::web::url::url_encode;

pub trait Connectable {
    fn connection_string(&self, conf: ConnectBasic, options: Option<HashMap<String, String>>) -> String;
}

pub trait ConnectProtocol {
    fn protocol(&self) -> &'static str;
}


/// 数据库连接信息
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConnectBasic {
    alias: String, // 别名, 用于连接时的标识
    host: String, // 主机地址
    port: u16, // 端口
    name: String, // 数据库名称
    user: String, // 用户名
    pass: String, // 密码
    charset: Option<String>, // 字符集
    options: HashMap<String, String>, // 其他选项
}

/// 数据库管理系统类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum DBMS {
    /// 关系型数据库
    Relational(Relational),
    /// 时序数据库
    TimeSeries(TimeSeries),
    /// 文档数据库
    Document(Document),
    /// 键值数据库
    KeyValue(KeyValue),
    /// 图数据库
    Graph(Graph),
    /// 列式数据库
    Column(Column),
    /// 搜索引擎
    Search(Search),
    /// 多模型数据库
    /// - 文档+图+键值: MongoDB
    /// - 图+文档: Neo4j
    /// - 键值+列族: Redis
    MultiModel(MultiModel),
}


/// 关系型数据库
#[derive(Debug, Clone, PartialEq)]
pub enum Relational {
    Oracle,
    Postgres,
    MySQL,
    SQLite,
    SQLServer,
}


/// 时序数据库
#[derive(Debug, Clone, PartialEq)]
pub enum TimeSeries {
    InfluxDB,
    Prometheus,
    Graphite,
    OpenTSDB,
}

/// 文档数据库
#[derive(Debug, Clone, PartialEq)]
pub enum Document {
    MongoDB,
    CouchDB,
    Elasticsearch,
    RethinkDB,
}


/// 键值数据库
#[derive(Debug, Clone, PartialEq)]
pub enum KeyValue {
    Redis,
    Memcached,
    HBase,
    Cassandra,
}


/// 图数据库
#[derive(Debug, Clone, PartialEq)]
pub enum Graph {
    Neo4j,
    Dgraph,
    Titan,
    ArangoDB,
}


/// 列式数据库
#[derive(Debug, Clone, PartialEq)]
pub enum Column {
    ClickHouse,
    Druid,
    Kudu,
    Hive,
}

/// 搜索引擎
#[derive(Debug, Clone, PartialEq)]
pub enum Search {
    Elasticsearch,
    Solr,
    Lucene,
    Xapian,
}


/// 多模型数据库
#[derive(Debug, Clone, PartialEq)]
pub enum MultiModel {
    MongoDB,
    Neo4j,
    Redis,
}

impl ConnectProtocol for DBMS {
    fn protocol(&self) -> &'static str {
        match self {
            DBMS::Relational(r) => match r {
                Relational::Oracle => "oracle",
                Relational::Postgres => "postgres",
                Relational::MySQL => "mysql",
                Relational::SQLite => "sqlite",
                Relational::SQLServer => "sqlserver",
            },
            DBMS::TimeSeries(ts) => match ts {
                TimeSeries::InfluxDB => "influxdb",
                TimeSeries::Prometheus => "prometheus",
                TimeSeries::Graphite => "graphite",
                TimeSeries::OpenTSDB => "opentsdb",
            },
            DBMS::Document(doc) => match doc {
                Document::MongoDB => "mongodb",
                Document::CouchDB => "couchdb",
                Document::Elasticsearch => "elasticsearch",
                Document::RethinkDB => "rethinkdb",
            },
            DBMS::KeyValue(kv) => match kv {
                KeyValue::Redis => "redis",
                KeyValue::Memcached => "memcached",
                KeyValue::HBase => "hbase",
                KeyValue::Cassandra => "cassandra",
            },
            DBMS::Graph(g) => match g {
                Graph::Neo4j => "neo4j",
                Graph::Dgraph => "dgraph",
                Graph::Titan => "titan",
                Graph::ArangoDB => "arangodb",
            },
            DBMS::Column(c) => match c {
                Column::ClickHouse => "clickhouse",
                Column::Druid => "druid",
                Column::Kudu => "kudu",
                Column::Hive => "hive",
            },
            DBMS::Search(s) => match s {
                Search::Elasticsearch => "elasticsearch",
                Search::Solr => "solr",
                Search::Lucene => "lucene",
                Search::Xapian => "xapian",
            },
            DBMS::MultiModel(mm) => match mm {
                MultiModel::MongoDB => "mongodb",
                MultiModel::Neo4j => "neo4j",
                MultiModel::Redis => "redis",
            },
        }
    }
}

impl Default for ConnectBasic {
    fn default() -> ConnectBasic {
        ConnectBasic {
            alias: Default::default(),
            host: Default::default(),
            port: 0,
            name: Default::default(),
            user: Default::default(),
            pass: Default::default(),
            charset: None,
            options: Default::default(),
        }
    }
}

impl ConnectBasic {
    pub fn new() -> ConnectBasic {
        ConnectBasic::default()
    }

    pub fn set_alias(&mut self, alias: String) -> &mut Self {
        self.alias = alias;
        self
    }

    pub fn set_host(&mut self, host: String) -> &mut Self {
        self.host = host;
        self
    }

    pub fn set_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_user(&mut self, user: String) -> &mut Self {
        self.user = user;
        self
    }

    pub fn set_pass(&mut self, pass: String) -> &mut Self {
        self.pass = pass;
        self
    }

    pub fn set_charset(&mut self, charset: String) -> &mut Self {
        self.charset = Some(charset);
        self
    }

    pub fn set_options(&mut self, options: HashMap<String, String>) -> &mut Self {
        self.options = options;
        self
    }

    pub fn add_option(&mut self, key: String, value: String) -> &mut Self {
        self.options.insert(key, value);
        self
    }

    pub fn remove_option(&mut self, key: &str) -> &mut Self {
        self.options.remove(key);
        self
    }

    /// 获取别名
    pub fn alias(&self) -> &str {
        &self.alias
    }

    /// 获取主机地址
    pub fn host(&self) -> &str {
        &self.host
    }

    /// 获取端口
    pub fn port(&self) -> u16 {
        self.port
    }

    /// 获取数据库名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取用户名
    pub fn user(&self) -> &str {
        &self.user
    }

    /// 获取密码
    pub fn pass(&self) -> &str {
        &self.pass
    }

    /// 获取字符集
    pub fn charset(&self) -> Option<&String> {
        self.charset.as_ref()
    }

    /// 获取其他选项
    pub fn options(&self) -> &HashMap<String, String> {
        &self.options
    }
    
    pub fn basic_connection_string(&self, protocol: &str, options: Option<HashMap<String, String>>) -> String {
        let mut c = format!("{}://{}:{}/{}?", protocol, self.host, self.port, self.name);
        // 添加用户名和密码
        c = format!("{}user={}&password={}", c, url_encode(&self.user), url_encode(&self.pass));

        // 如果有字符集设置，添加到连接字符串中
        if let Some(charset) = &self.charset {
            c = format!("{}&charset={}", c, url_encode(charset));
        }

        let mut opts = self.options.clone();
        if let Some(options) = options {
            opts.extend(options);
        }

        for (k, v) in opts.iter() {
            c = format!("{}&{}={}", c, url_encode(k), url_encode(v));
        }

        c
    }
}


#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;

    fn basic() -> ConnectBasic {
        ConnectBasic {
            alias: "test_db".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            name: "test_database".to_string(),
            user: "test_user".to_string(),
            pass: "test_password".to_string(),
            charset: Some("utf8".to_string()),
            options: {
                let mut map = HashMap::new();
                map.insert("sslmode".to_string(), "disable".to_string());
                map.insert("connect_timeout".to_string(), "10".to_string());
                map
            },
        }
    }
}