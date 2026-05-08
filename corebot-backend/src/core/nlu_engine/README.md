# NLU Engine

`nlu_engine` owns local NLP inference for the Rust backend. The application
layer applies the same tagged input format used during training, validates the
loaded artifact contract, and decodes model outputs into a Rust domain analysis.
The ONNX output adapter loads artifacts, tokenizes prepared tagged text, and
runs ONNX Runtime.

## Runtime Contract

The runtime artifact directory is provided through `COREBOT_NLU_ONNX_DIR` and
must contain:

- `model.onnx`
- `tokenizer.json`
- `label_maps.json`
- `onnx_contract.json`

The ONNX model must expose these inputs:

- `input_ids`
- `attention_mask`

The ONNX model must expose these outputs:

- `intent_logits`
- `ner_logits`

`label_maps.json` is the source of truth for mapping model output indexes back
to intent and BIO labels. Rust must not hard-code model labels.

## Tagged Input

The Rust application preprocessing must mirror the Python training input format
exactly.

With a workflow task:

```text
[TASK=WF_BOOK] [LANG=id] [DOMAIN=restaurant] empat orang besok 20.30
```

Without a workflow task:

```text
[LANG=en] [DOMAIN=restaurant] Hello
```

The context tags are model context only. They must never be returned as
entities, even if the model predicts a non-`O` BIO label on those tokens.

## Inference Flow

`AnalyzeTextUseCase::analyze` is intentionally the orchestration pipeline:

1. Validate that the loaded artifacts are internally consistent.
2. Build the tagged input from command text and inference context.
3. Ask the output runtime port to tokenize tagged text and run ONNX Runtime.
4. Rank intent logits with softmax.
5. Decode BIO token labels into entity spans on the original raw text.

`OnnxNluRuntime::run` must stay adapter-focused: it receives already-prepared
tagged input, runs tokenizer and ONNX Runtime, and returns raw logits plus token
metadata. It must not build tagged input or decode `NluAnalysis`.

The output is `NluAnalysis`, which includes the tagged text, primary intent,
ranked intent candidates, decoded entities, and token-level NER labels for
debugging or future observability.

## Failure Policy

There is no fallback runtime in this feature. Missing or invalid ONNX artifacts
are startup/configuration errors.

When the model, tokenizer, label maps, or contract disagree, the runtime should
return `NluRuntimeError::InvalidArtifact` instead of panicking. This is important
because mismatched artifacts are deployment/configuration issues, not user input
issues.
