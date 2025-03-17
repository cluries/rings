/// JSON Query

pub trait JsonQueryable {
    /// Extract a JSON field value
    fn extract(&self, field: &str) -> String;

    /// Extract a JSON field value as text
    fn extract_text(&self, field: &str) -> String;

    /// Extract a JSON field value as integer
    fn extract_int(&self, field: &str) -> String;

    /// Extract a JSON field value at specified path
    fn extract_path(&self, path: &str) -> String;

    /// Check if a JSON field exists
    fn exists(&self, field: &str) -> String;

    /// Check if a JSON path exists
    fn exists_path(&self, path: &str) -> String;

    /// Get JSON object keys
    fn keys(&self) -> String;

    /// Get JSON array length
    fn array_length(&self) -> String;

    /// Create a new JSON object
    fn build_object(&self, pairs: Vec<(&str, &str)>) -> String;

    /// Create a new JSON array
    fn build_array(&self, elements: Vec<&str>) -> String;
}


pub trait JsonOperationable {
    /// Set a JSON field value
    fn set(&self, field: &str, value: &str) -> String;

    /// Set a JSON field value at specified path
    fn set_path(&self, path: &str, value: &str) -> String;

    /// Delete a JSON field
    fn delete(&self, field: &str) -> String;

    /// Delete a JSON field at specified path
    fn delete_path(&self, path: &str) -> String;

    /// Merge two JSON objects
    fn merge(&self, other: &str) -> String;

    /// Append element to JSON array
    fn append(&self, element: &str) -> String;

    /// Remove element from JSON array
    fn remove(&self, index: i32) -> String;

    /// Update element in JSON array
    fn update(&self, index: i32, value: &str) -> String;
}


pub enum DBMS {
    Postgres,
    MySQL,
}

impl JsonQueryable for DBMS {
    fn extract(&self, field: &str) -> String {
        match self {
            DBMS::Postgres => format!("->> '{}'", field),
            DBMS::MySQL => format!("->>'$.{}'", field),
        }
    }

    fn extract_text(&self, field: &str) -> String {
        match self {
            DBMS::Postgres => format!("->>'{}' ", field),
            DBMS::MySQL => format!("->>'$.{}'", field),
        }
    }

    fn extract_int(&self, field: &str) -> String {
        match self {
            DBMS::Postgres => format!("->>'{}' ", field),
            DBMS::MySQL => format!("->>'$.{}'", field),
        }
    }

    fn extract_path(&self, path: &str) -> String {
        match self {
            DBMS::Postgres => format!("#>'{}'", path),
            DBMS::MySQL => format!("->>'$.{}'", path),
        }
    }

    fn exists(&self, field: &str) -> String {
        match self {
            DBMS::Postgres => format!("? '{}'", field),
            DBMS::MySQL => format!("JSON_CONTAINS_PATH(data, 'one', '$.{}')", field),
        }
    }

    fn exists_path(&self, path: &str) -> String {
        match self {
            DBMS::Postgres => format!("?& array[{}]", path),
            DBMS::MySQL => format!("JSON_CONTAINS_PATH(data, 'one', '$.{}')", path),
        }
    }

    fn keys(&self) -> String {
        match self {
            DBMS::Postgres => String::from("json_object_keys"),
            DBMS::MySQL => String::from("JSON_KEYS"),
        }
    }

    fn array_length(&self) -> String {
        match self {
            DBMS::Postgres => String::from("json_array_length"),
            DBMS::MySQL => String::from("JSON_LENGTH"),
        }
    }

    fn build_object(&self, pairs: Vec<(&str, &str)>) -> String {
        match self {
            DBMS::Postgres => {
                let pairs_str: Vec<String> = pairs.iter().map(|(k, v)| format!("'{}', {}", k, v)).collect();
                format!("json_build_object({})", pairs_str.join(", "))
            }
            DBMS::MySQL => {
                let pairs_str: Vec<String> = pairs.iter().map(|(k, v)| format!("'{}', {}", k, v)).collect();
                format!("JSON_OBJECT({})", pairs_str.join(", "))
            }
        }
    }

    fn build_array(&self, elements: Vec<&str>) -> String {
        match self {
            DBMS::Postgres => format!("json_build_array({})", elements.join(", ")),
            DBMS::MySQL => format!("JSON_ARRAY({})", elements.join(", ")),
        }
    }
}

impl JsonOperationable for DBMS {
    fn set(&self, field: &str, value: &str) -> String {
        match self {
            DBMS::Postgres => format!("jsonb_set(data, '{{{}}}', '{}')", field, value),
            DBMS::MySQL => format!("JSON_SET(data, '$.{}', '{}')", field, value),
        }
    }

    fn set_path(&self, path: &str, value: &str) -> String {
        match self {
            DBMS::Postgres => format!("jsonb_set(data, '{{{}}}', '{}')", path, value),
            DBMS::MySQL => format!("JSON_SET(data, '$.{}', '{}')", path, value),
        }
    }

    fn delete(&self, field: &str) -> String {
        match self {
            DBMS::Postgres => format!("data - '{}'", field),
            DBMS::MySQL => format!("JSON_REMOVE(data, '$.{}')", field),
        }
    }

    fn delete_path(&self, path: &str) -> String {
        match self {
            DBMS::Postgres => format!("data #- '{{{}}}'", path),
            DBMS::MySQL => format!("JSON_REMOVE(data, '$.{}')", path),
        }
    }

    fn merge(&self, other: &str) -> String {
        match self {
            DBMS::Postgres => format!("data || '{}'", other),
            DBMS::MySQL => format!("JSON_MERGE_PATCH(data, '{}')", other),
        }
    }

    fn append(&self, element: &str) -> String {
        match self {
            DBMS::Postgres => format!("jsonb_insert(data, '{{-1}}', '{}')", element),
            DBMS::MySQL => format!("JSON_ARRAY_APPEND(data, '$', '{}')", element),
        }
    }

    fn remove(&self, index: i32) -> String {
        match self {
            DBMS::Postgres => format!("data - '{}'", index),
            DBMS::MySQL => format!("JSON_REMOVE(data, '$[{}]')", index),
        }
    }

    fn update(&self, index: i32, value: &str) -> String {
        match self {
            DBMS::Postgres => format!("jsonb_set(data, '{{{}}}'::text[], '{}')", index, value),
            DBMS::MySQL => format!("JSON_SET(data, '$[{}]', '{}')", index, value),
        }
    }
}


/// Add tests for JSON operations
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_json_operations() {
        let db = DBMS::Postgres;

        // Test basic operations
        assert_eq!(db.extract("name"), "->> 'name'");
        assert_eq!(db.extract_path("info,address"), "#>'info,address'");

        // Test array/object building
        let pairs = vec![("name", "value"), ("age", "25")];
        assert_eq!(
            db.build_object(pairs),
            "json_build_object('name', value, 'age', 25)"
        );

        let elements = vec!["1", "2", "3"];
        assert_eq!(
            db.build_array(elements),
            "json_build_array(1, 2, 3)"
        );
    }

    #[test]
    fn test_mysql_json_operations() {
        let db = DBMS::MySQL;

        // Test basic operations
        assert_eq!(db.extract("name"), "->>'$.name'");
        assert_eq!(db.exists("age"), "JSON_CONTAINS_PATH(data, 'one', '$.age')");

        // Test modifications
        assert_eq!(
            db.set("name", "John"),
            "JSON_SET(data, '$.name', 'John')"
        );
        assert_eq!(
            db.append("value"),
            "JSON_ARRAY_APPEND(data, '$', 'value')"
        );
    }
}


