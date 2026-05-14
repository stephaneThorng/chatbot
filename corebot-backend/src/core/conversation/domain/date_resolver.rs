use chrono::NaiveDate;

/// Domain contract for resolving a raw date string (as extracted by the NLU)
/// into a concrete calendar date.
///
/// Implementations are language-specific (e.g. `EnglishDateResolver`) and live
/// in the adapter layer. The domain only owns this trait and its error type so
/// application handlers can depend on it without importing adapter modules.
pub trait DateResolver: Send + Sync {
    fn resolve(&self, raw: &str, today: NaiveDate) -> Result<NaiveDate, DateResolveError>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum DateResolveError {
    /// The raw string could not be parsed into any recognised date pattern.
    Unparseable,
    /// The resolved date falls strictly before today.
    PastDate(NaiveDate),
}
