/// Input command for the NLU analysis use case.
#[derive(Debug, Clone)]
pub struct AnalyzeTextCommand {
    pub text: String,
    pub lang: String,
    pub domain: String,
    pub task: Option<String>,
    pub slot: Option<String>,
}
