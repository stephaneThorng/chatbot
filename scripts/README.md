# Chatbot Test Scripts

This directory contains scripts to set up and run the chatbot for testing.

## Prerequisites

- **Java 21** (JDK)
- **Python 3.11**

## Quick Start

### 1. Install Dependencies

**Windows:**
```powershell
.\scripts\install.bat
```

**Unix/macOS:**
```bash
./scripts/install.sh
```

This will:
- Verify Java 21 is installed
- Verify Python 3.11 is installed
- Create/check runtime venv (`.venv`) for NLP API
- Create/check training venv (`.venv311`) for model training

### 2. Launch Services

**Windows:**
```powershell
.\scripts\launch.bat
```

**Unix/macOS:**
```bash
./scripts/launch.sh
```

This will start:
- NLP API on port 8000
- Ktor backend on port 8080

Optionally launches the chat CLI.

### 3. Test the Chatbot

Open your browser to:
- Backend API: http://localhost:8080
- OpenAPI Docs: http://localhost:8080/openapi

Or use the terminal chat client when prompted in launch.bat/launch.sh.

### 4. Train Models (Optional)

Only run this if you need to retrain the NLP models.

**Windows:**
```powershell
.\scripts\train.bat
```

**Unix/macOS:**
```bash
./scripts/train.sh
```

This will:
1. Train intent classifier
2. Train NER model
3. Run evaluation

Training requires:
- ~10GB free disk space
- GPU recommended (CPU training is slow)

## Troubleshooting

### Port already in use

If ports 8000 or 8080 are busy, stop the existing services:
- Windows: `taskkill /F /IM python.exe` (for NLP)
- Check for other Java processes using port 8080

### Java not found

Install JDK 21 from https://adoptium.net/

### Python not found

Install Python 3.11 from https://www.python.org/downloads/

### Training out of memory

Reduce batch size in training scripts or use a machine with more RAM.