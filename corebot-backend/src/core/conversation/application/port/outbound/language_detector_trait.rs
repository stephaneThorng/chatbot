pub trait LanguageDetectorPort: Send + Sync {
    fn detect(&self, text: &str) -> String;
}
