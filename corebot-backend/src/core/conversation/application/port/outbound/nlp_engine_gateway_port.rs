use crate::core::conversation::domain::domain_type::DomainType;
use crate::core::conversation::domain::model::intent::NluTask;
use crate::core::nlu_engine::domain::analysis::NluAnalysis;

pub trait NlpEngineGatewayPort {
    fn analyze(
        &self,
        text: &str,
        lang: &str,
        domain: DomainType,
        task: Option<NluTask>,
    ) -> NluAnalysis;
}
