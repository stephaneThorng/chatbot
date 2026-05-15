use std::cmp::Ordering;

use crate::core::nlu_engine::application::nlu_artifacts::{LabelMaps, OnnxContract};
use crate::core::nlu_engine::application::nlu_model_inference::OnnxModelOutput;
use crate::core::nlu_engine::application::port::outbound::nlu_model_runtime_port::NluRuntimeError;
use crate::core::nlu_engine::domain::analysis::{
    NerTokenLabel, NluAnalysis, NluEntity, NluIntent, NluIntentCandidate, TaggedInput,
};
use crate::core::nlu_engine::domain::entity_type::EntityType;

/// Decodes raw runtime outputs into the domain-level analysis returned by the use case.
pub fn decode_nlu_analysis(
    tagged_input: TaggedInput,
    raw_text: &str,
    tokens: &[String],
    offsets: &[(usize, usize)],
    outputs: OnnxModelOutput,
    contract: &OnnxContract,
    label_maps: &LabelMaps,
) -> Result<NluAnalysis, NluRuntimeError> {
    let ranked_intents = ranked_intents(&outputs.intent_logits, contract)?;
    let primary_intent = ranked_intents
        .first()
        .ok_or_else(|| NluRuntimeError::Onnx("model returned no intent scores".to_string()))?;

    let ner_label_count = contract.labels.ner.len();
    validate_ner_logits_shape(&outputs.ner_logits, tokens.len(), ner_label_count)?;
    let entities = decode_entities(
        raw_text,
        tagged_input.prefix_length,
        tokens,
        offsets,
        &outputs.ner_logits,
        ner_label_count,
        label_maps,
    )?;
    let ner_labels = decode_token_labels(
        tagged_input.prefix_length,
        tokens,
        offsets,
        &outputs.ner_logits,
        ner_label_count,
        label_maps,
    )?;

    Ok(NluAnalysis {
        processed_text: tagged_input.text,
        intent: NluIntent {
            name: primary_intent.name.clone(),
            confidence: primary_intent.confidence,
        },
        intents: ranked_intents,
        entities,
        ner_labels,
    })
}

pub fn validate_artifacts(
    contract: &OnnxContract,
    label_maps: &LabelMaps,
) -> Result<(), NluRuntimeError> {
    if contract.labels.intents.is_empty() {
        return Err(NluRuntimeError::InvalidArtifact(
            "onnx contract must define at least one intent label".to_string(),
        ));
    }
    if contract.labels.ner.is_empty() {
        return Err(NluRuntimeError::InvalidArtifact(
            "onnx contract must define at least one NER label".to_string(),
        ));
    }
    for index in 0..contract.labels.intents.len() {
        if !label_maps.intent_id2label.contains_key(&index.to_string()) {
            return Err(NluRuntimeError::InvalidArtifact(format!(
                "label_maps.json is missing intent_id2label entry {index}"
            )));
        }
    }
    for index in 0..contract.labels.ner.len() {
        if !label_maps.ner_id2label.contains_key(&index.to_string()) {
            return Err(NluRuntimeError::InvalidArtifact(format!(
                "label_maps.json is missing ner_id2label entry {index}"
            )));
        }
    }
    Ok(())
}

fn ranked_intents(
    logits: &[f32],
    contract: &OnnxContract,
) -> Result<Vec<NluIntentCandidate>, NluRuntimeError> {
    if logits.len() != contract.labels.intents.len() {
        return Err(NluRuntimeError::InvalidArtifact(format!(
            "intent_logits length {} does not match {} intent labels",
            logits.len(),
            contract.labels.intents.len()
        )));
    }
    let probabilities = softmax(logits);
    let mut ranked = contract
        .labels
        .intents
        .iter()
        .enumerate()
        .map(|(index, name)| NluIntentCandidate {
            name: name.clone(),
            confidence: probabilities[index],
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .confidence
            .partial_cmp(&left.confidence)
            .unwrap_or(Ordering::Equal)
    });
    Ok(ranked)
}

fn decode_token_labels(
    prefix_length: usize,
    tokens: &[String],
    offsets: &[(usize, usize)],
    ner_values: &[f32],
    ner_label_count: usize,
    label_maps: &LabelMaps,
) -> Result<Vec<NerTokenLabel>, NluRuntimeError> {
    validate_offsets(tokens, offsets)?;
    validate_ner_logits_shape(ner_values, tokens.len(), ner_label_count)?;
    let mut labels = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let (start, end) = offsets[index];
        if start == end {
            continue;
        }
        let label_index =
            argmax(&ner_values[(index * ner_label_count)..((index + 1) * ner_label_count)]);
        let label = label_maps
            .ner_id2label
            .get(&label_index.to_string())
            .cloned()
            .unwrap_or_else(|| "O".to_string());
        let adjusted_start = start.saturating_sub(prefix_length);
        let adjusted_end = end.saturating_sub(prefix_length);
        labels.push(NerTokenLabel {
            token: token.clone(),
            label,
            start: adjusted_start,
            end: adjusted_end,
        });
    }
    Ok(labels)
}

fn decode_entities(
    raw_text: &str,
    prefix_length: usize,
    tokens: &[String],
    offsets: &[(usize, usize)],
    ner_values: &[f32],
    ner_label_count: usize,
    label_maps: &LabelMaps,
) -> Result<Vec<NluEntity>, NluRuntimeError> {
    validate_offsets(tokens, offsets)?;
    validate_ner_logits_shape(ner_values, tokens.len(), ner_label_count)?;
    let mut entities = Vec::new();
    let mut current_type: Option<String> = None;
    let mut current_start = 0usize;
    let mut current_end = 0usize;
    let mut confidences = Vec::new();

    for (index, _) in tokens.iter().enumerate() {
        let (token_start, token_end) = offsets[index];
        if token_start == token_end {
            continue;
        }

        let logits = &ner_values[(index * ner_label_count)..((index + 1) * ner_label_count)];
        let probabilities = softmax(logits);
        let label_index = argmax(logits);
        let label = label_maps
            .ner_id2label
            .get(&label_index.to_string())
            .cloned()
            .unwrap_or_else(|| "O".to_string());

        if token_end <= prefix_length || label == "O" {
            flush_entity(
                raw_text,
                &mut entities,
                &mut current_type,
                &mut current_start,
                &mut current_end,
                &mut confidences,
            );
            continue;
        }

        let adjusted_start = token_start.saturating_sub(prefix_length);
        let adjusted_end = token_end.saturating_sub(prefix_length);
        let (prefix, entity_type) = match label.split_once('-') {
            Some(parts) => parts,
            None => {
                flush_entity(
                    raw_text,
                    &mut entities,
                    &mut current_type,
                    &mut current_start,
                    &mut current_end,
                    &mut confidences,
                );
                continue;
            }
        };

        match (prefix, current_type.as_deref()) {
            ("B", Some(active)) if active == entity_type => {
                current_end = adjusted_end;
                confidences.push(probabilities[label_index]);
            }
            ("B", _) => {
                flush_entity(
                    raw_text,
                    &mut entities,
                    &mut current_type,
                    &mut current_start,
                    &mut current_end,
                    &mut confidences,
                );
                current_type = Some(entity_type.to_string());
                current_start = adjusted_start;
                current_end = adjusted_end;
                confidences.push(probabilities[label_index]);
            }
            ("I", Some(active)) if active == entity_type => {
                current_end = adjusted_end;
                confidences.push(probabilities[label_index]);
            }
            ("I", _) => {
                flush_entity(
                    raw_text,
                    &mut entities,
                    &mut current_type,
                    &mut current_start,
                    &mut current_end,
                    &mut confidences,
                );
                current_type = Some(entity_type.to_string());
                current_start = adjusted_start;
                current_end = adjusted_end;
                confidences.push(probabilities[label_index]);
            }
            _ => {
                flush_entity(
                    raw_text,
                    &mut entities,
                    &mut current_type,
                    &mut current_start,
                    &mut current_end,
                    &mut confidences,
                );
            }
        }
    }

    flush_entity(
        raw_text,
        &mut entities,
        &mut current_type,
        &mut current_start,
        &mut current_end,
        &mut confidences,
    );
    Ok(entities)
}

fn flush_entity(
    raw_text: &str,
    entities: &mut Vec<NluEntity>,
    current_type: &mut Option<String>,
    current_start: &mut usize,
    current_end: &mut usize,
    confidences: &mut Vec<f32>,
) {
    let entity_type = match current_type.take() {
        Some(entity_type) => entity_type,
        None => {
            confidences.clear();
            return;
        }
    };
    let start = *current_start;
    let end = *current_end;
    if start >= end
        || end > raw_text.len()
        || !raw_text.is_char_boundary(start)
        || !raw_text.is_char_boundary(end)
    {
        confidences.clear();
        return;
    }
    let value = raw_text[start..end].to_string();
    let confidence = if confidences.is_empty() {
        0.0
    } else {
        confidences.iter().sum::<f32>() / confidences.len() as f32
    };
    entities.push(NluEntity {
        entity_type: EntityType::from(&entity_type),
        value: value.clone(),
        raw_value: value,
        start,
        end,
        confidence,
    });
    confidences.clear();
}

fn argmax(values: &[f32]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.partial_cmp(right.1).unwrap_or(Ordering::Equal))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn validate_offsets(tokens: &[String], offsets: &[(usize, usize)]) -> Result<(), NluRuntimeError> {
    if tokens.len() != offsets.len() {
        return Err(NluRuntimeError::InvalidArtifact(format!(
            "token count {} does not match offset count {}",
            tokens.len(),
            offsets.len()
        )));
    }
    Ok(())
}

fn validate_ner_logits_shape(
    ner_values: &[f32],
    token_count: usize,
    ner_label_count: usize,
) -> Result<(), NluRuntimeError> {
    if ner_label_count == 0 {
        return Err(NluRuntimeError::InvalidArtifact(
            "onnx contract must define at least one NER label".to_string(),
        ));
    }
    let expected = token_count.checked_mul(ner_label_count).ok_or_else(|| {
        NluRuntimeError::InvalidArtifact("NER logits shape overflows usize".to_string())
    })?;
    if ner_values.len() != expected {
        return Err(NluRuntimeError::InvalidArtifact(format!(
            "ner_logits length {} does not match token_count {token_count} * ner_label_count {ner_label_count}",
            ner_values.len()
        )));
    }
    Ok(())
}

fn softmax(values: &[f32]) -> Vec<f32> {
    let max_value = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let exp = values
        .iter()
        .map(|value| (*value - max_value).exp())
        .collect::<Vec<_>>();
    let total = exp.iter().sum::<f32>();
    exp.into_iter().map(|value| value / total).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nlu_engine::application::nlu_artifacts::LabelMaps;

    #[test]
    fn decode_entities_ignores_context_tags() {
        let tokens = vec![
            "[TASK=WF_RESERVATION_CREATE]".to_string(),
            "[LANG=id]".to_string(),
            "[DOMAIN=restaurant]".to_string(),
            "Agus".to_string(),
            "Wijaya".to_string(),
        ];
        let offsets = vec![(0, 28), (29, 38), (39, 58), (59, 63), (64, 70)];
        let label_maps = LabelMaps {
            intent_label2id: Default::default(),
            intent_id2label: Default::default(),
            ner_label2id: Default::default(),
            ner_id2label: [
                ("0".to_string(), "O".to_string()),
                ("1".to_string(), "B-person".to_string()),
                ("2".to_string(), "I-person".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        let ner_logits = vec![
            2.0, 0.1, 0.1, 2.0, 0.1, 0.1, 2.0, 0.1, 0.1, 0.1, 2.0, 0.1, 0.1, 0.1, 2.0,
        ];

        let entities = decode_entities(
            "Agus Wijaya",
            59,
            &tokens,
            &offsets,
            &ner_logits,
            3,
            &label_maps,
        )
        .unwrap();

        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].entity_type, EntityType::Person);
        assert_eq!(entities[0].value, "Agus Wijaya");
    }

    #[test]
    fn decode_entities_merges_repeated_b_labels_for_same_entity_type() {
        let tokens = vec!["next".to_string(), "tu".to_string(), "esday".to_string()];
        let offsets = vec![(0, 4), (5, 7), (7, 12)];
        let label_maps = LabelMaps {
            intent_label2id: Default::default(),
            intent_id2label: Default::default(),
            ner_label2id: Default::default(),
            ner_id2label: [
                ("0".to_string(), "O".to_string()),
                ("1".to_string(), "B-date".to_string()),
                ("2".to_string(), "I-date".to_string()),
            ]
            .into_iter()
            .collect(),
        };
        let ner_logits = vec![
            0.1, 2.0, 0.1, // next -> B-date
            0.1, 2.0, 0.1, // tu -> B-date
            0.1, 0.1, 2.0, // esday -> I-date
        ];

        let entities = decode_entities(
            "next tuesday",
            0,
            &tokens,
            &offsets,
            &ner_logits,
            3,
            &label_maps,
        )
        .unwrap();

        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].entity_type, EntityType::Date);
        assert_eq!(entities[0].value, "next tuesday");
    }

    #[test]
    fn decode_entities_rejects_inconsistent_ner_shape() {
        let tokens = vec!["Agus".to_string()];
        let offsets = vec![(0, 4)];
        let label_maps = LabelMaps {
            intent_label2id: Default::default(),
            intent_id2label: Default::default(),
            ner_label2id: Default::default(),
            ner_id2label: [("0".to_string(), "O".to_string())].into_iter().collect(),
        };

        let result = decode_entities("Agus", 0, &tokens, &offsets, &[0.1, 0.2], 1, &label_maps);

        assert!(matches!(result, Err(NluRuntimeError::InvalidArtifact(_))));
    }

    #[test]
    fn flush_entity_skips_invalid_utf8_boundaries() {
        let mut entities = Vec::new();
        let mut current_type = Some("person".to_string());
        let mut current_start = 1usize;
        let mut current_end = 3usize;
        let mut confidences = vec![0.9];

        flush_entity(
            "Ã‰lodie",
            &mut entities,
            &mut current_type,
            &mut current_start,
            &mut current_end,
            &mut confidences,
        );

        assert!(entities.is_empty());
    }
}
