# Model Training

Python-only NLU training project for one multi-task model:

- intent classification
- BIO NER
- English and Indonesian
- restaurant domain for v1
- future `hotel` domain tag support

The runtime API is intentionally out of scope. This folder produces a model artifact and metrics only.

## Dataset Format

JSONL rows store raw user text. Entity spans are character offsets inside `text`, not inside the tagged model input.

```json
{"text":"Jean for 4 people tomorrow at 8pm","task":"WF_RESERVATION_CREATE","lang":"en","domain":"restaurant","intent":"reservation_create","entities":[{"start":0,"end":4,"type":"person"},{"start":9,"end":17,"type":"people_count"},{"start":18,"end":26,"type":"date"},{"start":30,"end":33,"type":"time"}]}
```

Model input is built at training time:

```text
[TASK=WF_RESERVATION_CREATE] [LANG=en] [DOMAIN=restaurant] Jean for 4 people tomorrow at 8pm
```

Rows outside an active workflow omit `task`, so the input starts with `[LANG=...] [DOMAIN=...]`.

## Commands

## Setup

Use Python 3.12 for the training environment. The project dependencies include
PyTorch, Transformers, ONNX export tools, and pytest.

From the repository root:

```powershell
cd model_training
py -3.12 -m venv .venv
.\.venv\Scripts\Activate.ps1
python -m pip install --upgrade pip
python -m pip install -r requirements.txt
```

If `py -3.12` is not available, install Python 3.12 from
https://www.python.org/downloads/windows/ and enable "Add python.exe to PATH"
during installation.

After activation, confirm the environment:

```powershell
python --version
python -m pytest --version
```

## Dataset Generation

From `model_training` with the virtual environment active:

```powershell
python generate_dataset.py
python -m pytest tests
```

Generation writes deterministic JSONL files under `data/restaurant/`:

- `train.jsonl`
- `validation.jsonl`
- `eval.jsonl`

The generator should produce about 5000 total examples across English and
Indonesian. The tests verify row count, no duplicate texts per
`domain/lang/task/intent`, entity spans, `REST-...` reservation references,
conversation date examples, and structured price conditions.

## Training

From `model_training` with the virtual environment active:

```powershell
python train.py
python evaluate.py --model-dir outputs/restaurant_xlmr
```

Useful smoke train:

```powershell
python train.py --train data/restaurant/train.jsonl --validation data/restaurant/validation.jsonl --output outputs/smoke --max-train-samples 24 --max-validation-samples 12 --epochs 1
```

Export ONNX for Rust inference:

```powershell
python export_onnx.py --model-dir outputs/restaurant_xlmr
```

The ONNX export preserves the exact tagged input contract used at training time:

```text
[TASK=WF_RESERVATION_CREATE] [LANG=id] [DOMAIN=restaurant] empat orang besok 20.30
```

If `task` is absent, the input starts with `[LANG=...] [DOMAIN=...]`.

## Outputs

Training writes:

- model and tokenizer files
- `label_maps.json`
- `training_config.yaml`
- `metrics.json`
- `debug_bio_preview.json`

ONNX export writes:

- `onnx/model.onnx`
- `onnx/tokenizer.json`
- `onnx/label_maps.json`
- `onnx/training_config.yaml`
- `onnx/onnx_contract.json`
- `onnx/onnx_validation.json`
