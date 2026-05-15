use crate::core::conversation::application::dto::nlu_analysis_result::NluAnalysisResult;
use crate::core::conversation::domain::domain_type::DomainType;
use crate::core::conversation::domain::model::intent::NluTask;
use crate::core::conversation::domain::model::slot::SlotName;

pub trait NlpEngineGatewayPort {
    fn analyze(
        &self,
        text: &str,
        lang: &str,
        domain: DomainType,
        task: Option<NluTask>,
        slot_hint: Option<SlotName>,
    ) -> NluAnalysisResult;
}
