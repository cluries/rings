use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prompt {
    system: Option<String>,
    assistant: Option<String>,
    user: String,
}

pub struct PromptBuilder {
    role: Option<String>,
    skills: Option<Vec<String>>,
    task: Option<String>,
    constraints: Option<std::collections::HashMap<String, Vec<String>>>,
    fewshots: Option<Vec<Fewshot>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fewshot {
    input: String,
    output: String,
}

// pub type Fewshots = Vec<Fewshot>;

pub const CONSTRAINT_GROUP_GLOBAL: &'static str = "Global";
pub const CONSTRAINT_GROUP_OUTPUT: &'static str = "Output Format";

pub const CONSTRAINT_ONLY_RESULTS_NO_FORMATS: [&'static str; 2] = [
    "Strictly output only the result, without any prefixes, suffixes, explanations, or formatting. Provide the answer directly.",
    "严格只输出结果，无需任何前缀、后缀、解释或格式化,直接给出答案即可。",
];

/// Return JSON in this exact structure
pub const CONSTRAINT_ONLY_RESULTS_USE_JSON_PREFIX: [&'static str; 2] = [
    "Strictly return the result in JSON format only, without any extra explanations, notes, or text. Output valid JSON exclusively, like",
    "请严格以JSON格式返回结果，不要包含任何额外的解释、说明或文本。仅输出有效的JSON内容，格式如下",
];

pub const CCONSTRAINT_ACCURACY_OVER_CREATIVITY: [&'static str; 2] =
    ["Prioritize accuracy over creativity in all responses", "所有回答保持中立，不表达主观观点"];

pub fn constraint_json(structure: &str) -> String {
    let s = structure.split("\n").into_iter().map(|x| x.trim().to_string()).collect::<Vec<String>>();
    format!("{}: {}", CONSTRAINT_ONLY_RESULTS_USE_JSON_PREFIX[0], s.join(""))
}

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
    pub fn new(role: &str, task: &str) -> PromptBuilder {
        PromptBuilder { role: Some(role.to_string()), skills: None, task: Some(task.to_string()), constraints: None, fewshots: None }
    }

    pub fn set_role(mut self, role: &str) -> Self {
        self.role = Some(role.to_string());
        self
    }

    pub fn add_skill(mut self, skill: &str) -> Self {
        match self.skills.as_mut() {
            None => {
                self.skills = Some(vec![String::from(skill)]);
            },
            Some(skills) => {
                skills.push(String::from(skill));
            },
        }
        self
    }

    pub fn set_task(mut self, task: &str) -> Self {
        self.task = Some(String::from(task));
        self
    }

    pub fn add_constraint(mut self, group: &str, constraint: &str) -> Self {
        if let None = self.constraints {
            let constrains = std::collections::HashMap::<String, Vec<String>>::new();
            self.constraints = Some(constrains);
        }

        let constrains = self.constraints.as_mut().unwrap();
        match constrains.get_mut(group) {
            None => {
                constrains.insert(String::from(group), vec![String::from(constraint)]);
            },
            Some(consts) => {
                consts.push(String::from(constraint));
            },
        }

        self
    }

    pub fn add_constraint_global(self, constraint: &str) -> Self {
        self.add_constraint(CONSTRAINT_GROUP_GLOBAL, constraint)
    }

    pub fn add_constraint_output(self, constraint: &str) -> Self {
        self.add_constraint(CONSTRAINT_GROUP_OUTPUT, constraint)
    }

    pub fn add_fewshot(mut self, fewshot: Fewshot) -> Self {
        match self.fewshots.as_mut() {
            None => self.fewshots = Some(vec![fewshot]),
            Some(fewshots) => fewshots.push(fewshot),
        };
        self
    }

    pub fn add_fewshot_directly(self, input: &str, output: &str) -> Self {
        self.add_fewshot(Fewshot { input: input.into(), output: output.into() })
    }

    pub fn build(self) -> String {
        let mut prompt: Vec<String> = Vec::new();
        if let Some(role) = self.role {
            prompt.push(format!("## Role\n- {}", role));
        }

        if let Some(skills) = self.skills {
            let mut vec: Vec<String> = Vec::new();
            vec.push("## Skills".to_string());
            for skill in skills {
                vec.push(format!("- {}", skill));
            }
            prompt.push(vec.join("\n"));
        }
        if let Some(task) = self.task {
            prompt.push(format!("## Task\n- {}", task));
        }

        if let Some(constraints) = self.constraints {
            let mut vec: Vec<String> = Vec::new();
            vec.push("## Constraints".to_string());
            let mut keys: Vec<String> = constraints.keys().map(|x| x.to_string()).collect();
            keys.sort();

            for (i, group) in keys.iter().enumerate() {
                vec.push(format!("### {}", group));
                let constraints = constraints.get(group).unwrap();
                for constraint in constraints {
                    vec.push(format!("- {}", constraint));
                }
                if i == 0 {
                    vec.push(" ".to_string());
                }
            }

            prompt.push(vec.join("\n"));
        }

        if let Some(fewshots) = self.fewshots {
            let mut vec: Vec<String> = Vec::new();
            vec.push("## Examples".to_string());
            for fewshot in fewshots {
                vec.push(format!("- Input: {}", fewshot.input));
                vec.push(format!("  Output: {}", fewshot.output));
            }

            prompt.push(vec.join("\n"));
        }

        prompt.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        struct Person {
            name: String,
            age: i32,
        }

        struct Record {
            time: String,
            unit: String,
            amount: i32,
        }

        struct MoneyMaker {
            id: i64,
            person: Person,
            total: i32,
            records: Vec<Record>,
        }

        let s = PromptBuilder::new("monery maker", "make money for people")
            .add_constraint_global("money is good")
            .add_constraint_global("use chinese")
            .add_skill("generate money")
            .add_skill("generate person")
            .add_constraint_output(&constraint_json(
                r#"{
                    "name": "string, 名字",
                    "age":  "number, 年龄",
                    "girls": [
                        "name": "string, 名字",
                         "age":  "number, 年龄"
                    ]
                }"#,
            ))
            .add_fewshot_directly("古大", "天神")
            .add_fewshot_directly("古大大", "天神");

        println!("{}", s.build());
    }

    #[test]
    fn test_json_output_format() {}
}
