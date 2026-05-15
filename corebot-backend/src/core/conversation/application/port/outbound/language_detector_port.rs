pub trait LanguageDetectorPort {
    fn detect(&self, text: &str) -> String;
}
