use langdetect_rs::detector_factory::DetectorFactory;

use crate::core::conversation::application::port::outbound::language_detector_trait::LanguageDetectorPort;

pub struct LangdetectLanguageDetector {
    detector: DetectorFactory,
}

impl LangdetectLanguageDetector {
    pub fn new() -> Self {
        Self {
            detector: DetectorFactory::default().build(),
        }
    }
}

impl Default for LangdetectLanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageDetectorPort for LangdetectLanguageDetector {
    fn detect(&self, text: &str) -> String {
        match self.detector.detect(text, None).ok().as_deref() {
            Some("id") => "id".to_string(),
            Some("en") => "en".to_string(),
            _ => "en".to_string(),
        }
    }
}
