use crate::erx::{smp, ResultE};
use serde::{de::DeserializeOwned, Serialize};
use serde_json;


/// Encoder
pub struct Enc;

///Decoder
pub struct Dec;

///Describer
pub struct Describe;

impl Enc {
    pub fn en<T: Serialize>(obj: &T) -> ResultE<String> {
        serde_json::to_string(obj).map_err(smp)
    }

    pub fn ens<T: Serialize>(obj: &T) -> String {
        serde_json::to_string(obj).unwrap_or(Default::default())
    }

    pub fn pretty<T: Serialize>(obj: &T) -> ResultE<String> {
        serde_json::to_string_pretty(obj).map_err(smp)
    }
}

impl Dec {
    pub fn de<T: DeserializeOwned>(json: &str) -> ResultE<T> {
        serde_json::from_str(json).map_err(smp)
    }

    pub async fn file<T: DeserializeOwned>(filename: &str) -> ResultE<T> {
        let fc = crate::tools::fs::Content(filename.to_string());
        fc.json().await
    }

    pub fn is_valid(s: &str) -> bool {
        serde_json::from_str::<serde_json::Value>(s).is_ok()
    }
}

impl Describe {
    pub fn describe<T: Serialize>(object: &T, cribe: std::collections::HashMap<String, String>) -> ResultE<String> {
        let mut value = serde_json::to_value(object).map_err(smp)?;

        fn recurse(current: &mut serde_json::Value, path: &str, descriptions: &std::collections::HashMap<String, String>) {
            match current {
                serde_json::Value::Object(map) => {
                    for (key, val) in map.iter_mut() {
                        let path = if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };
                        recurse(val, &path, descriptions);
                    }
                },
                serde_json::Value::Array(arr) => {
                    // ### FixIt ###
                    // 这里如果是一个空的数组, 那无法获取到对应项类型，下面逻辑也执行不到 FixIt
                    for item in arr.iter_mut() {
                        recurse(item, path, descriptions);
                    }
                },
                _ => {
                    if path.is_empty() {
                        return;
                    }

                    let describe = match descriptions.get(path) {
                        Some(description) => description.to_string(),
                        None => path.to_string(),
                    };
                    *current = serde_json::Value::String(describe);
                },
            }
        }

        recurse(&mut value, "", &cribe);

        serde_json::to_string(&value).map_err(smp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Serialize, PartialEq, Debug, Default)]
    struct Tag {
        id: i64,
        name: String,
    }

    #[derive(Deserialize, Serialize, PartialEq, Debug, Default)]
    struct Book {
        name: String,
        pages: i32,
        price: f32,
        tags: Vec<Tag>,
    }

    #[derive(Deserialize, Serialize, PartialEq, Debug, Default)]
    struct Info {
        birthday: String,
        description: String,
        mark: String,
        address: String,
        mobile: String,
        education: String,
        email: String,
    }

    #[derive(Deserialize, Serialize, PartialEq, Debug, Default)]
    struct Person {
        name: String,
        age: i32,
        likes: Vec<Book>,
        info: Info,
    }

    #[test]
    fn json_deser() {
        let mut person = Person::default();
        let mut book: Book = Book::default();
        book.tags.push(Default::default());
        person.likes.push(book);

        let mut describe: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        describe.insert(String::from("name"), String::from("姓名"));
        describe.insert(String::from("likes.name"), String::from("书名"));
        describe.insert(String::from("likes.tags.name"), String::from("标签"));

        println!("{}", Desc::describe(&person, describe).unwrap());
    }
}
