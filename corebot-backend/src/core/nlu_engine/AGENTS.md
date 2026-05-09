# NLU Engine Instructions

## Ownership

- `nlu_engine` owns local NLP inference only: preprocessing, tokenization,
  ONNX execution, intent ranking, and BIO entity decoding.
- Do not add restaurant reply generation, workflow state transitions, or
  conversation policy here. Those belong to `conversation`.
- Do not add keyword intent classifiers here. The model owns intent and entity
  extraction.

## Artifact Contract

- Keep Rust preprocessing aligned with `model_training`.
- Preserve the tag order: optional `[TASK=...]`, then `[LANG=...]`, then
  `[DOMAIN=...]`, then raw text.
- Omit `[TASK=...]` completely when no task is present.
- Treat `label_maps.json` and `onnx_contract.json` as runtime contract files.
- Never hard-code intent or NER label indexes in Rust.

## Layer Responsibilities

- `domain/` contains pure NLU domain objects such as `InferenceContext`,
  `TaggedInput`, `NluAnalysis`, intents, entities, and token labels.
- `application/` contains `AnalyzeTextUseCase`, input/output ports, artifact
  contract structs, preprocessing orchestration, artifact validation, and output
  decoding.
- `adapter/output/` contains concrete ONNX Runtime and tokenizer integration.
- Artifact serialization structs such as ONNX contract and label maps belong in
  `application/` or adapter boundary modules, not in `domain/`.
- `OnnxNluRuntime` must receive prepared `TaggedInput`; it must not build it.
- `OnnxNluRuntime` must return raw logits, tokens, and offsets through the
  output port; it must not call `decode_nlu_analysis` or build `NluAnalysis`.
- `AnalyzeTextUseCase` owns the full application pipeline: build tagged input,
  validate artifacts, call the model runtime port, and decode the final
  `NluAnalysis`.

## Error Handling

- Invalid model/tokenizer/label-map shapes must return `NluRuntimeError`, not
  panic.
- Avoid silently falling back to heuristics. If ONNX artifacts are missing, fail
  clearly.
- Keep tests around context-tag exclusion, BIO decoding, and artifact-shape
  validation whenever this feature changes.

## Architecture

- Domain types live under `domain`.
- Ports and use cases live under `application`.
- ONNX Runtime and tokenizer integration live under `adapter/output`.
- Keep adapters depending on application ports and boundary/domain types, not on
  conversation internals or application orchestration helpers.
