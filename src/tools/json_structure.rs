//! A module for defining and describing the structure of a JSON object.
//!
//! This is useful for generating documentation, validating schemas, or providing
//! clear instructions to an LLM about the expected format of a JSON object.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Creates a `JsonStructure` using a concise, JSON-like syntax.
///
/// This macro provides a highly ergonomic way to define the structure of a JSON object,
/// including nested objects and arrays.
///
/// # Example
///
/// ```
/// # use rings::tools::json_structure::{json_struct, JsonStructure, JsonType};
/// let structure = json_struct!({
///     "name": (string, "The full name of the user."),
///     "age": (number, "The age of the user."),
///     "is_active": (boolean, "Whether the user account is active."),
///     "address": ({
///         "street": (string, "Street name and number."),
///         "city": (string, "City name.")
///     }, "The user's primary address."),
///     "tags": ([string], "A list of user-specific tags."),
///     "history": ([{
///         "event_type": (string, "The type of event."),
///         "timestamp": (number, "When the event occurred.")
///     }], "A list of historical events.")
/// });
/// ```
#[macro_export]
macro_rules! json_struct {
    // Main entry point for an object
    ({ $($tt:tt)* }) => {
        {
            let mut structure = $crate::tools::json_structure::JsonStructure::new();
            json_struct_inner!(structure, { $($tt)* });
            structure
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! json_struct_inner {
    // Base case: All tokens have been processed.
    ($s:ident, {}) => {};

    // --- Rules for fields followed by a comma ---

    // Primitive field with a comma
    ($s:ident, { $name:literal: ($type:ident, $desc:literal), $($rest:tt)* }) => {
        $s = $s.add_field($name, $crate::tools::json_structure::JsonType::$type, $desc);
        json_struct_inner!($s, { $($rest)* });
    };

    // Nested object field with a comma
    ($s:ident, { $name:literal: ({ $($inner:tt)* }, $desc:literal), $($rest:tt)* }) => {
        $s = $s.add_object($name, json_struct!({ $($inner)* }), $desc);
        json_struct_inner!($s, { $($rest)* });
    };

    // Array of primitives with a comma
    ($s:ident, { $name:literal: ([$type:ident], $desc:literal), $($rest:tt)* }) => {
        $s = $s.add_array($name, $crate::tools::json_structure::JsonType::$type, $desc);
        json_struct_inner!($s, { $($rest)* });
    };

    // Array of objects with a comma
    ($s:ident, { $name:literal: ([{ $($inner:tt)* }], $desc:literal), $($rest:tt)* }) => {
        $s = $s.add_array($name, $crate::tools::json_structure::JsonType::Object(json_struct!({ $($inner)* })), $desc);
        json_struct_inner!($s, { $($rest)* });
    };

    // --- Rules for the final field in the list (no trailing comma) ---

    // Final primitive field
    ($s:ident, { $name:literal: ($type:ident, $desc:literal) }) => {
        $s = $s.add_field($name, $crate::tools::json_structure::JsonType::$type, $desc);
    };

    // Final nested object field
    ($s:ident, { $name:literal: ({ $($inner:tt)* }, $desc:literal) }) => {
        $s = $s.add_object($name, json_struct!({ $($inner)* }), $desc);
    };

    // Final array of primitives
    ($s:ident, { $name:literal: ([$type:ident], $desc:literal) }) => {
        $s = $s.add_array($name, $crate::tools::json_structure::JsonType::$type, $desc);
    };

    // Final array of objects
    ($s:ident, { $name:literal: ([{ $($inner:tt)* }], $desc:literal) }) => {
        $s = $s.add_array($name, $crate::tools::json_structure::JsonType::Object(json_struct!({ $($inner)* })), $desc);
    };
}


/// Represents the type of a field in a JSON object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JsonType {
    /// A string value.
    String,
    /// A numeric value (integer or float).
    Number,
    /// A boolean value (`true` or `false`).
    Boolean,
    /// A JSON object, defined by its own set of fields.
    Object(JsonStructure),
    /// An array of values of a specific type.
    Array(Box<JsonType>),
    /// A null value.
    Null,
}

/// Describes a single field within a JSON object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonField {
    /// The data type of the field.
    #[serde(rename = "type")]
    pub field_type: JsonType,
    /// A comment or description explaining the purpose of the field.
    pub description: String,
}

/// Represents the complete structure of a JSON object.
///
/// It is essentially a map where keys are field names and values
/// describe the type and purpose of each field.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsonStructure {
    /// A map from field names to their corresponding `JsonField` definitions.
    fields: HashMap<String, JsonField>,
}

impl JsonStructure {
    /// Creates a new, empty `JsonStructure`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or updates a field in the JSON structure.
    ///
    /// # Arguments
    /// * `name` - The name of the field (the key in the JSON object).
    /// * `field_type` - The `JsonType` of the field.
    /// * `description` - A comment explaining the field's purpose.
    pub fn add_field(mut self, name: &str, field_type: JsonType, description: &str) -> Self {
        self.fields.insert(
            name.to_string(),
            JsonField {
                field_type,
                description: description.to_string(),
            },
        );
        self
    }

    /// Returns a reference to the fields map.
    pub fn fields(&self) -> &HashMap<String, JsonField> {
        &self.fields
    }

    /// Convenience method to add a string field.
    pub fn add_string(self, name: &str, description: &str) -> Self {
        self.add_field(name, JsonType::String, description)
    }

    /// Convenience method to add a number field.
    pub fn add_number(self, name: &str, description: &str) -> Self {
        self.add_field(name, JsonType::Number, description)
    }

    /// Convenience method to add a boolean field.
    pub fn add_boolean(self, name: &str, description: &str) -> Self {
        self.add_field(name, JsonType::Boolean, description)
    }

    /// Convenience method to add a null field.
    pub fn add_null(self, name: &str, description: &str) -> Self {
        self.add_field(name, JsonType::Null, description)
    }

    /// Convenience method to add a nested object field.
    pub fn add_object(self, name: &str, structure: JsonStructure, description: &str) -> Self {
        self.add_field(name, JsonType::Object(structure), description)
    }

    /// Convenience method to add an array field.
    pub fn add_array(self, name: &str, element_type: JsonType, description: &str) -> Self {
        self.add_field(name, JsonType::Array(Box::new(element_type)), description)
    }

    /// Generates a compact, example-like JSON string representing the structure.
    ///
    /// This format is often more intuitive for LLMs and humans than a full JSON schema.
    ///
    /// # Returns
    /// A pretty-printed JSON string describing the structure in a compact format.
    pub fn to_compact_json_string(&self) -> String {
        let value = self.to_compact_json_value();
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
    }

    /// Recursively builds a `serde_json::Value` from the `JsonStructure`.
    fn to_compact_json_value(&self) -> Value {
        let mut map = serde_json::Map::new();
        for (name, field) in &self.fields {
            map.insert(
                name.clone(),
                Self::type_to_compact_value(&field.field_type, &field.description),
            );
        }
        Value::Object(map)
    }

    /// Converts a `JsonType` into the compact `serde_json::Value` representation.
    fn type_to_compact_value(field_type: &JsonType, description: &str) -> Value {
        match field_type {
            JsonType::String => Value::String(format!("string, {}", description)),
            JsonType::Number => Value::String(format!("number, {}", description)),
            JsonType::Boolean => Value::String(format!("boolean, {}", description)),
            JsonType::Null => Value::String(format!("null, {}", description)),
            JsonType::Object(inner_structure) => inner_structure.to_compact_json_value(),
            JsonType::Array(inner_type) => {
                // When the array contains objects, we recursively render the object structure.
                // When it contains primitives, we use the array's description to describe the elements.
                let element_representation = match &**inner_type {
                    JsonType::Object(s) => s.to_compact_json_value(),
                    JsonType::String => Value::String(format!("string, {}", description)),
                    JsonType::Number => Value::String(format!("number, {}", description)),
                    JsonType::Boolean => Value::String(format!("boolean, {}", description)),
                    JsonType::Null => Value::String(format!("null, {}", description)),
                    JsonType::Array(_) => Self::type_to_compact_value(inner_type, description),
                };
                Value::Array(vec![element_representation])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_json_structure_with_convenience_methods() {
        let user_profile_structure = JsonStructure::new()
            .add_number("userId", "The unique identifier for the user.")
            .add_string("username", "The user's public display name.")
            .add_boolean("isActive", "Indicates if the user's account is active.")
            .add_array(
                "tags",
                JsonType::String,
                "A list of tags associated with the user.",
            );

        let address_structure = JsonStructure::new()
            .add_string("street", "The street name and number.")
            .add_string("city", "The city of the address.");

        let final_structure = user_profile_structure
            .add_object("address", address_structure, "The user's primary address.");

        // Verify a top-level field
        let username_field = final_structure.fields().get("username").unwrap();
        assert_eq!(username_field.description, "The user's public display name.");
        assert!(matches!(username_field.field_type, JsonType::String));

        // Verify a nested object field
        let address_field = final_structure.fields().get("address").unwrap();
        if let JsonType::Object(address_struct) = &address_field.field_type {
            let city_field = address_struct.fields().get("city").unwrap();
            assert_eq!(city_field.description, "The city of the address.");
        } else {
            panic!("Address field was not an object.");
        }

        // Verify an array field
        let tags_field = final_structure.fields().get("tags").unwrap();
        if let JsonType::Array(inner_type) = &tags_field.field_type {
            assert!(matches!(**inner_type, JsonType::String));
        } else {
            panic!("Tags field was not an array.");
        }
    }

    #[test]
    fn test_serialization() {
        let structure = JsonStructure::new()
            .add_field("name", JsonType::String, "The name of the item.")
            .add_field("count", JsonType::Number, "How many items are in stock.");

        let serialized = serde_json::to_string_pretty(&structure).unwrap();
        println!("{}", serialized);

        let expected = r#"{
  "fields": {
    "count": {
      "type": "number",
      "description": "How many items are in stock."
    },
    "name": {
      "type": "string",
      "description": "The name of the item."
    }
  }
}"#;
        // Comparing JSON strings is tricky due to key order, so we deserialize both to compare.
        let expected_value: serde_json::Value = serde_json::from_str(expected).unwrap();
        let actual_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn test_to_compact_json_string() {
        let nested_object = JsonStructure::new()
            .add_field("nest_key", JsonType::String, "comment2");

        let structure = JsonStructure::new()
            .add_field("key", JsonType::Number, "comment")
            .add_field(
                "key2",
                JsonType::Array(Box::new(JsonType::Object(nested_object))),
                "list of nested objects",
            )
            .add_field(
                "key3",
                JsonType::Array(Box::new(JsonType::String)),
                "list of strings",
            );

        let compact_string = structure.to_compact_json_string();
        println!("{}", compact_string);

        let expected = r#"{
  "key": "number, comment",
  "key2": [
    {
      "nest_key": "string, comment2"
    }
  ],
  "key3": [
    "string, list of strings"
  ]
}"#;

        // Compare by deserializing to Value to ignore formatting differences
        let actual_value: Value = serde_json::from_str(&compact_string).unwrap();
        let expected_value: Value = serde_json::from_str(expected).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn test_json_struct_macro() {
        let structure = json_struct!({
            "key": (Number, "comment"),
            "key2": ([{
                "nest_key": (String, "comment2")
            }], "list of nested objects"),
            "key3": ([String], "list of strings")
        });

        let compact_string = structure.to_compact_json_string();
        println!("{}", compact_string);

        let expected_json = r#"{
          "key": "number, comment",
          "key2": [
            {
              "nest_key": "string, comment2"
            }
          ],
          "key3": [
            "string, list of strings"
          ]
        }"#;

        let actual_value: Value = serde_json::from_str(&compact_string).unwrap();
        let expected_value: Value = serde_json::from_str(expected_json).unwrap();

        assert_eq!(actual_value, expected_value);

        // Also test against the manual builder to ensure consistency
        let manual_nested = JsonStructure::new().add_string("nest_key", "comment2");
        let manual_structure = JsonStructure::new()
            .add_number("key", "comment")
            .add_array(
                "key2",
                JsonType::Object(manual_nested),
                "list of nested objects",
            )
            .add_array("key3", JsonType::String, "list of strings");

        assert_eq!(
            structure.to_compact_json_string(),
            manual_structure.to_compact_json_string()
        );
    }
}
