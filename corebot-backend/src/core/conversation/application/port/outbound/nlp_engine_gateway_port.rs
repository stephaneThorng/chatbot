use crate::core::conversation::application::nlu_analysis_result::NluAnalysisResult;
use crate::core::conversation::domain::domain_type::DomainType;
use crate::core::conversation::domain::model::intent::NluTask;

pub trait NlpEngineGatewayPort {
    fn analyze(
        &self,
        text: &str,
        lang: &str,
        domain: DomainType,
        task: Option<NluTask>,
    ) -> NluAnalysisResult;
}
