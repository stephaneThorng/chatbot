use crate::core::nlu_engine::application::nlu_analysis_decoder::{
    decode_nlu_analysis, validate_artifacts,
};
use crate::core::nlu_engine::domain::analysis::{InferenceContext, NluAnalysis, TaggedInput};

use super::analyze_text_command::AnalyzeTextCommand;
use super::port::inbound::analyze_text_usecase::AnalyzeTextUseCase;
use super::port::outbound::nlu_model_runtime_port::{NluModelRuntimePort, NluRuntimeError};

/// Application use case that orchestrates tagged-input construction, runtime
/// execution, and decoding into a domain `NluAnalysis`.
pub struct AnalyzeTextService<R>
where
    R: NluModelRuntimePort,
{
    runtime: R,
}

impl<R> AnalyzeTextService<R>
where
    R: NluModelRuntimePort,
{
    /// Creates the use case with the runtime port implementation to call.
    pub fn new(runtime: R) -> Self {
        Self { runtime }
    }
}

impl<R> AnalyzeTextUseCase for AnalyzeTextService<R>
where
    R: NluModelRuntimePort,
{
    fn analyze(&self, command: AnalyzeTextCommand) -> Result<NluAnalysis, NluRuntimeError> {
        validate_artifacts(self.runtime.contract(), self.runtime.label_maps())?;
        let raw_text = command.text;
        let context = InferenceContext::new(command.lang, command.domain, command.task);
        let tagged_input = TaggedInput::build(&raw_text, &context);
        let inference = self.runtime.run(&tagged_input)?;
        decode_nlu_analysis(
            tagged_input,
            &raw_text,
            &inference.tokens,
            &inference.offsets,
            inference.outputs,
            self.runtime.contract(),
            self.runtime.label_maps(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;

    use super::*;
    use crate::core::nlu_engine::application::nlu_artifacts::{
        ContractLabels, LabelMaps, OnnxContract,
    };
    use crate::core::nlu_engine::application::nlu_model_inference::{
        NluModelInference, OnnxModelOutput,
    };

    #[derive(Clone)]
    struct CapturingRuntime {
        contract: OnnxContract,
        label_maps: LabelMaps,
        last_input: Arc<Mutex<Option<(String, usize)>>>,
    }

    impl CapturingRuntime {
        fn new() -> Self {
            Self {
                contract: OnnxContract {
                    max_length: 128,
                    model_inputs: vec!["input_ids".to_string(), "attention_mask".to_string()],
                    model_outputs: vec!["intent_logits".to_string(), "ner_logits".to_string()],
                    labels: ContractLabels {
                        intents: vec!["greet".to_string()],
                        ner: vec!["O".to_string()],
                    },
                },
                label_maps: LabelMaps {
                    intent_label2id: [("greet".to_string(), 0)].into_iter().collect(),
                    intent_id2label: [("0".to_string(), "greet".to_string())]
                        .into_iter()
                        .collect(),
                    ner_label2id: [("O".to_string(), 0)].into_iter().collect(),
                    ner_id2label: [("0".to_string(), "O".to_string())].into_iter().collect(),
                },
                last_input: Arc::new(Mutex::new(None)),
            }
        }

        fn last_tagged_input(&self) -> (String, usize) {
            self.last_input.lock().unwrap().as_ref().unwrap().clone()
        }
    }

    impl NluModelRuntimePort for CapturingRuntime {
        fn contract(&self) -> &OnnxContract {
            &self.contract
        }

        fn label_maps(&self) -> &LabelMaps {
            &self.label_maps
        }

        fn run(&self, input: &TaggedInput) -> Result<NluModelInference, NluRuntimeError> {
            *self.last_input.lock().unwrap() = Some((input.text.clone(), input.prefix_length));
            Ok(NluModelInference {
                tokens: vec![
                    "[LANG=en]".to_string(),
                    "[DOMAIN=restaurant]".to_string(),
                    "Hello".to_string(),
                ],
                offsets: vec![(0, 9), (10, 29), (30, 35)],
                outputs: OnnxModelOutput {
                    intent_logits: vec![1.0],
                    ner_logits: vec![1.0, 1.0, 1.0],
                },
            })
        }
    }

    #[test]
    fn usecase_builds_tagged_input_before_running_model() {
        let runtime = CapturingRuntime::new();
        let usecase = AnalyzeTextService::new(runtime.clone());

        let analysis = usecase
            .analyze(AnalyzeTextCommand {
                text: "Hello".to_string(),
                lang: "en".to_string(),
                domain: "restaurant".to_string(),
                task: None,
            })
            .unwrap();

        let (tagged_text, prefix_length) = runtime.last_tagged_input();
        assert_eq!(tagged_text, "[LANG=en] [DOMAIN=restaurant] Hello");
        assert_eq!(prefix_length, 30);
        assert_eq!(analysis.processed_text, tagged_text);
        assert_eq!(analysis.intent.name, "greet");
    }
}
