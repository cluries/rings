use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prompt {
    system: Option<String>,
    assistant: Option<String>,
    user: String,
}

pub struct PromptBuilder {
    skills: Option<Skills>,
    actions: Option<Actions>,
    constrains: Option<Constrains>,
    output_format: Option<OutFormat>,
}

/// A structured prompt component.
trait PromptComponent {
    fn render(&self, w: &mut fmt::Formatter<'_>) -> fmt::Result;
}

pub struct CommonComponent {
    kind: String,
    value: Vec<String>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ValueKind {
    String,
    Integer,
    Float,
    Object,
    Array,
    Boolean,
}

pub struct OutFormatJson {
    kind: ValueKind,
    field: String,
    comment: String,
    nest_object: Option<Vec<Box<OutFormatJson>>>,
    nest_array: Option<Box<OutFormatJson>>,
}

pub struct Skills(Vec<String>);
pub struct Actions(Vec<String>);
pub struct Constrains(Vec<String>);
pub struct OutFormat(OutFormatJson);

pub struct Input(String);

impl Prompt {
    pub fn system(&self) -> &str {
        self.system.as_deref().unwrap_or("")
    }
    pub fn assistant(&self) -> &str {
        self.assistant.as_deref().unwrap_or("")
    }
    pub fn user(&self) -> &str {
        &self.user
    }
}

impl PromptBuilder {
    pub fn new() -> PromptBuilder {
        PromptBuilder { skills: None, actions: None, constrains: None, output_format: None }
    }

    pub fn add_skill(mut self, skill: &str) -> Self {
        match self.skills.as_mut() {
            None => {
                self.skills = Some(Skills(vec![String::from(skill)]));
            },
            Some(skills) => {
                skills.0.push(String::from(skill));
            },
        }
        self
    }

    pub fn add_action(mut self, action: &str) -> Self {
        match self.actions.as_mut() {
            None => {
                self.actions = Some(Actions(vec![String::from(action)]));
            },
            Some(actions) => {
                actions.0.push(String::from(action));
            },
        }
        self
    }

    pub fn add_constraint(mut self, constraint: &str) -> Self {
        match self.constrains.as_mut() {
            None => {
                self.constrains = Some(Constrains(vec![String::from(constraint)]));
            },
            Some(constrains) => {
                constrains.0.push(String::from(constraint));
            },
        }
        self
    }

    // pub fn set_output_format(mut self, output: OutFormat) -> Self {
    //     match self.output_format.as_mut() {
    //         None => {
    //             self.output_format = Some(OutFormat(output.to_string()));
    //         },
    //         Some(format) => {
    //             format.0 = output.to_string();
    //         },
    //     }
    //     self
    // }

    pub fn build(self) -> String {
        let mut prompt: Vec<String> = Vec::new();
        if let Some(skills) = self.skills {
            let c: CommonComponent = skills.into();
            prompt.push(c.to_string());
        }
        if let Some(actions) = self.actions {
            let c: CommonComponent = actions.into();
            prompt.push(c.to_string());
        }
        if let Some(constraints) = self.constrains {
            let c: CommonComponent = constraints.into();
            prompt.push(c.to_string());
        }

        prompt.join("\n")
    }
}

impl fmt::Display for CommonComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.render(f)
    }
}

impl PromptComponent for CommonComponent {
    fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.value.iter().map(|x| format!("- {}", x)).collect::<Vec<String>>().join("\n");
        writeln!(f, "## {}\n{}", self.kind, value)
    }
}

impl Into<CommonComponent> for Skills {
    fn into(self) -> CommonComponent {
        CommonComponent { kind: "Skills".to_string(), value: self.0 }
    }
}

impl Into<CommonComponent> for Actions {
    fn into(self) -> CommonComponent {
        CommonComponent { kind: "Actions".to_string(), value: self.0 }
    }
}

impl Into<CommonComponent> for Constrains {
    fn into(self) -> CommonComponent {
        CommonComponent { kind: "Constrains".to_string(), value: self.0 }
    }
}

// impl Into<CommonComponent> for OutFormat {
//     fn into(self) -> CommonComponent {
//         CommonComponent { kind: "Format".to_string(), value: vec![self.0] }
//     }
// }

impl Into<CommonComponent> for Input {
    fn into(self) -> CommonComponent {
        CommonComponent { kind: "Input".to_string(), value: vec![self.0] }
    }
}

impl OutFormatJson {
    pub fn prefix() -> &'static str {
        "Return JSON in this exact structure"
    }

    pub fn new_str(field: &str, comment: &str) -> OutFormatJson {
        OutFormatJson {
            kind: ValueKind::String,
            field: field.to_string(),
            comment: comment.to_string(),
            nest_array: None,
            nest_object: None,
        }
    }

    pub fn new_int(field: &str, comment: &str) -> OutFormatJson {
        OutFormatJson {
            kind: ValueKind::Integer,
            field: field.to_string(),
            comment: comment.to_string(),
            nest_array: None,
            nest_object: None,
        }
    }

    pub fn new_float(field: &str, comment: &str) -> OutFormatJson {
        OutFormatJson {
            kind: ValueKind::Float,
            field: field.to_string(),
            comment: comment.to_string(),
            nest_array: None,
            nest_object: None,
        }
    }

    pub fn new_bool(field: &str, comment: &str) -> OutFormatJson {
        OutFormatJson {
            kind: ValueKind::Boolean,
            field: field.to_string(),
            comment: comment.to_string(),
            nest_array: None,
            nest_object: None,
        }
    }

    pub fn new_array(field: &str, comment: &str, nest_array: OutFormatJson) -> OutFormatJson {
        OutFormatJson {
            kind: ValueKind::Array,
            field: field.to_string(),
            comment: comment.to_string(),
            nest_array: Some(Box::new(nest_array)),
            nest_object: None,
        }
    }

    pub fn new_object(field: &str, comment: &str, nest_object: Vec<Box<OutFormatJson>>) -> OutFormatJson {
        OutFormatJson {
            kind: ValueKind::Array,
            field: field.to_string(),
            comment: comment.to_string(),
            nest_array: None,
            nest_object: Some(nest_object),
        }
    }

    pub fn description(&self) -> String {
        let kind = self.kind.clone();
        match kind {
            ValueKind::String => {
                format!("\"{}\": \"string, {}\"", self.field, self.comment)
            },
            ValueKind::Integer => {
                format!("\"{}\": \"integer, {}\"", self.field, self.comment)
            },
            ValueKind::Float => {
                format!("\"{}\": \"float, {}\"", self.field, self.comment)
            },
            ValueKind::Object => {
                let n = self.nest_object.as_ref().unwrap();
                let n = n.iter().map(|x| x.description()).collect::<Vec<String>>().join(",\n ");
                format!("\"{}\": {{ {} }}", self.field, n)
            },
            ValueKind::Array => {
                let n = self.nest_array.as_ref().unwrap().description();
                format!("\"{}\": [ {} ] ", self.field, n)
            },
            ValueKind::Boolean => {
                format!("\"{}\": \"boolean, {}\"", self.field, self.comment)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let s = PromptBuilder::new()
            .add_action("action")
            .add_constraint("dont_care")
            .add_constraint("use chinese")
            .add_skill("convert json to xml")
            .add_skill("convert xml to json");
        println!("{}", s.build());
    }

    #[test]
    fn test_json_output_format() {
        let k = OutFormatJson::new_object(
            "foo",
            "bar",
            vec![Box::new(OutFormatJson::new_int("foo", "bar")), Box::new(OutFormatJson::new_float("foo", "bar"))],
        );
        println!("{}", k.description());
    }
}
