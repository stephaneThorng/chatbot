#[derive(Debug, Clone, PartialEq)]
pub struct InferenceContext {
    pub lang: String,
    pub domain: String,
    pub task: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaggedInput {
    pub text: String,
    pub prefix_length: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NluIntent {
    pub name: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NluIntentCandidate {
    pub name: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NluEntity {
    pub entity_type: String,
    pub value: String,
    pub raw_value: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NerTokenLabel {
    pub token: String,
    pub label: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NluAnalysis {
    pub tagged_text: String,
    pub intent: NluIntent,
    pub intents: Vec<NluIntentCandidate>,
    pub entities: Vec<NluEntity>,
    pub ner_labels: Vec<NerTokenLabel>,
}

impl InferenceContext {
    pub fn new(lang: impl Into<String>, domain: impl Into<String>, task: Option<String>) -> Self {
        Self {
            lang: lang.into(),
            domain: domain.into(),
            task,
        }
    }
}

pub fn build_tagged_input(text: &str, context: &InferenceContext) -> TaggedInput {
    let mut tags = Vec::new();
    if let Some(task) = context.task.as_ref() {
        tags.push(format!("[TASK={task}]"));
    }
    tags.push(format!("[LANG={}]", context.lang));
    tags.push(format!("[DOMAIN={}]", context.domain));
    let prefix = tags.join(" ");
    TaggedInput {
        text: format!("{prefix} {text}"),
        prefix_length: prefix.len() + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tagged_input_omits_task_when_missing() {
        let context = InferenceContext::new("en", "restaurant", None);
        let tagged = build_tagged_input("Hello", &context);
        assert_eq!(tagged.text, "[LANG=en] [DOMAIN=restaurant] Hello");
    }

    #[test]
    fn tagged_input_includes_task_when_present() {
        let context = InferenceContext::new("id", "restaurant", Some("WF_BOOK".to_string()));
        let tagged = build_tagged_input("empat orang", &context);
        assert_eq!(
            tagged.text,
            "[TASK=WF_BOOK] [LANG=id] [DOMAIN=restaurant] empat orang"
        );
    }
}
