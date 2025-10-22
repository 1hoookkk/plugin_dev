# DeepSeek-OCR Setup Guide (WSL2 + RTX 4060)

## Current Status

✅ WSL2 Ubuntu running
✅ Python 3.12 installed
✅ CUDA 13.0 available
✅ RTX 4060 GPU detected (8GB)
🔄 **Installing PyTorch + CUDA libraries (~3 GB, ~10-15 minutes)**

## What's Happening

PyTorch is downloading CUDA runtime libraries. This is a one-time installation.

Monitor progress:
```bash
wsl -d Ubuntu bash -c "ps aux | grep pip"
```

## Installation Commands (Auto-run when PyTorch finishes)

Run these in WSL2 Ubuntu once PyTorch is installed:

```bash
# Activate environment
cd ~/deepseek-ocr-workspace
source venv/bin/activate

# Install remaining dependencies
pip install transformers pillow requests tqdm

# Clone DeepSeek-OCR
git clone https://github.com/deepseek-ai/DeepSeek-OCR.git
cd DeepSeek-OCR

# Test GPU
python3 -c "import torch; print(f'GPU: {torch.cuda.is_available()}')"
```

## Project Structure

```
~/deepseek-ocr-workspace/
├── venv/                          # Python virtual environment
├── DeepSeek-OCR/                  # Cloned repository
├── test_images/                   # Your test photos go here
└── results/                       # OCR results saved here
```

## Quick Test (After Installation)

```python
# File: ~/deepseek-ocr-workspace/test_inference.py
from PIL import Image
from transformers import AutoModel, AutoTokenizer
import torch

# Load model
model = AutoModel.from_pretrained("deepseek-ai/DeepSeek-OCR", trust_remote_code=True, torch_dtype=torch.bfloat16)
tokenizer = AutoTokenizer.from_pretrained("deepseek-ai/DeepSeek-OCR", trust_remote_code=True)

# Load image
image = Image.open("test_images/sample.jpg")

# Extract
messages = [{"role": "user", "content": [{"type": "image", "image": image}, {"type": "text", "text": "Extract all text from this image"}]}]
inputs = tokenizer(messages, return_tensors="pt")
outputs = model.generate(**inputs, max_new_tokens=1024)
print(tokenizer.decode(outputs[0], skip_special_tokens=True))
```

## Use Cases for Your Projects

### 1. Construction Quote Generator (Idea #1)
**Prompt:** "Analyze this bathroom renovation photo. List all fixtures visible: toilets, sinks, showers, tiles, lights. Count each item."

**Expected output:**
```json
{
  "room": "bathroom",
  "fixtures": {
    "toilets": 1,
    "sinks": 2,
    "showers": 1,
    "tiles": "ceramic floor + walls",
    "lights": 3
  }
}
```

### 2. Defect/Snag Organizer (Idea #2)
**Prompt:** "This is a construction site photo. Identify defects: what's wrong? Where? How severe (low/med/high)? Which trade is responsible?"

**Expected output:**
```json
{
  "location": "bathroom wall",
  "defect": "cracked plaster",
  "severity": "medium",
  "responsible_trade": "plastering",
  "notes": "appears to be settlement crack"
}
```

### 3. Government Valuation Extraction (PropSignal use case)
**Prompt:** "This is a property valuation document. Extract: address, capital improved value, land value, valuation date."

## Performance Expectations

With RTX 4060 (8GB):
- **Throughput:** ~100-200 tokens/second (slower than A100, but fast enough for most tasks)
- **Memory:** ~6-7GB per inference
- **First load:** ~30-60 seconds (model download + GPU load)
- **Subsequent runs:** ~5-10 seconds per image

## Troubleshooting

### "CUDA out of memory"
- Reduce `max_new_tokens` (default 1024 → try 512)
- Use `torch.float32` instead of `bfloat16` (slower but uses less memory)

### "Model not found"
- First run downloads ~7GB model from HuggingFace
- Ensure internet connection
- Check disk space in `/home` (need ~10GB)

### "Slow on first run"
- Normal - model is compiling CUDA kernels
- Second run will be faster (within same session)

## Next Steps

1. **Wait for PyTorch installation to complete** (~10-15 minutes)
2. **Run test on a simple image**
3. **Test with real photos from your discovery interviews**
4. **Measure accuracy and speed**
5. **Build MVP web app** (Next.js + FastAPI backend)

## Files Created

- `deepseek_ocr_test.py` - Multi-use case test script
- `setup_deepseek.sh` - Automated setup script
- This guide

## Testing with Your Ideas

### Step 1: Gather Test Images
Ask people in your network to send 2-3 photos of:
- Bathroom/kitchen renovations (for quote generator)
- Construction defects (for snag organizer)
- Property documents (for PropSignal)

### Step 2: Run Extraction
```bash
cd ~/deepseek-ocr-workspace
source venv/bin/activate

# Copy your test images
cp /mnt/c/user_photos/* test_images/

# Run test
python3 /mnt/c/plugin_dev/deepseek_ocr_test.py
```

### Step 3: Evaluate Results
Check results in:
- `~/deepseek-ocr-workspace/results/`
- `C:\plugin_dev\deepseek_result_*.json`

## Next Phase: MVP Web App

Once you've validated accuracy, build a simple web app:

**Tech Stack:**
- Frontend: Next.js 14 (React)
- Backend: FastAPI (Python)
- Upload: S3 or Supabase Storage
- DB: Supabase PostgreSQL

**Simple MVP:**
1. Upload image → send to FastAPI backend
2. Backend runs DeepSeek-OCR
3. Return structured JSON → display in UI
4. User edits if needed → export PDF/CSV

Estimated time: 2-3 days
