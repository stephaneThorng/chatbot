/// The business domain this session belongs to.
/// Fixed per session (determined by access token or endpoint).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DomainType {
    Restaurant,
    Hotel,
}
