use std::sync::Arc;

use crate::core::nlu_engine::application::nlu_analysis_decoder::{
    decode_nlu_analysis, validate_artifacts,
};
use crate::core::nlu_engine::domain::analysis::{InferenceContext, NluAnalysis, TaggedInput};

use super::analyze_text_command::AnalyzeTextCommand;
use super::port::inbound::analyze_text_trait::AnalyzeTextPort;
use super::port::outbound::nlu_model_runtime_trait::{NluModelRuntimePort, NluRuntimeError};

/// Application use case that orchestrates tagged-input construction, runtime
/// execution, and decoding into a domain `NluAnalysis`.
pub struct AnalyzeTextUseCase {
    runtime: Arc<dyn NluModelRuntimePort>,
}

impl AnalyzeTextUseCase {
    /// Creates the use case with the runtime port implementation to call.
    pub fn new(runtime: Arc<dyn NluModelRuntimePort>) -> Self {
        Self { runtime }
    }
}

impl AnalyzeTextPort for AnalyzeTextUseCase {
    fn analyze(&self, command: AnalyzeTextCommand) -> Result<NluAnalysis, NluRuntimeError> {
        validate_artifacts(self.runtime.contract(), self.runtime.label_maps())?;
        let raw_text = command.text;
        let context = InferenceContext::new(command.lang, command.domain, command.task);
        let tagged_input = TaggedInput::build(&raw_text, &context);
        let inference = self.runtime.run(tagged_input.clone())?;
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
    use std::sync::Mutex;

    use crate::core::nlu_engine::application::nlu_artifacts::{
        ContractLabels, LabelMaps, OnnxContract,
    };
    use crate::core::nlu_engine::application::nlu_model_inference::{
        NluModelInference, OnnxModelOutput,
    };
    use crate::core::nlu_engine::domain::analysis::TaggedInput;

    use super::*;

    struct CapturingRuntime {
        contract: OnnxContract,
        label_maps: LabelMaps,
        last_input: Mutex<Option<TaggedInput>>,
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
                last_input: Mutex::new(None),
            }
        }

        fn last_tagged_input(&self) -> TaggedInput {
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

        fn run(&self, input: TaggedInput) -> Result<NluModelInference, NluRuntimeError> {
            *self.last_input.lock().unwrap() = Some(input);
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
        let runtime = Arc::new(CapturingRuntime::new());
        let usecase = AnalyzeTextUseCase::new(runtime.clone());

        let analysis = usecase
            .analyze(AnalyzeTextCommand {
                text: "Hello".to_string(),
                lang: "en".to_string(),
                domain: "restaurant".to_string(),
                task: None,
            })
            .unwrap();

        let tagged_input = runtime.last_tagged_input();
        assert_eq!(tagged_input.text, "[LANG=en] [DOMAIN=restaurant] Hello");
        assert_eq!(tagged_input.prefix_length, 30);
        assert_eq!(analysis.tagged_text, tagged_input.text);
        assert_eq!(analysis.intent.name, "greet");
    }
}
