use crate::core::conversation::domain::domain_type::DomainType;
use crate::core::conversation::domain::intent::NluTask;
use crate::core::nlu_engine::domain::analysis::NluAnalysis;

pub trait NlpEngineGatewayPort: Send + Sync {
    fn analyze(
        &self,
        text: &str,
        lang: &str,
        domain: DomainType,
        task: Option<NluTask>,
    ) -> NluAnalysis;
}
