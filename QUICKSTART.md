# DeepSeek-OCR Quick Start (WSL2 Ubuntu)

## Status: PyTorch Installing 🔄

PyTorch + CUDA libraries are downloading in background. ETA: ~10-15 minutes.

**To monitor progress:**
```bash
wsl -d Ubuntu bash -c "ps aux | grep pip"
```

---

## Once PyTorch Finishes ✅

### Step 1: Activate Environment

```bash
wsl -d Ubuntu bash -c "
cd ~/deepseek-ocr-workspace
source venv/bin/activate
echo 'Environment activated!'
"
```

### Step 2: Install Dependencies

```bash
wsl -d Ubuntu bash -c "
cd ~/deepseek-ocr-workspace
source venv/bin/activate
pip install transformers pillow requests tqdm fastapi uvicorn
echo 'Dependencies installed!'
"
```

### Step 3: Test GPU Connectivity

```bash
wsl -d Ubuntu bash -c "
cd ~/deepseek-ocr-workspace
source venv/bin/activate
python3 << 'EOF'
import torch
print(f'PyTorch version: {torch.__version__}')
print(f'CUDA available: {torch.cuda.is_available()}')
if torch.cuda.is_available():
    print(f'GPU: {torch.cuda.get_device_name(0)}')
    print(f'Memory: {torch.cuda.get_device_properties(0).total_memory / 1e9:.1f}GB')
EOF
"
```

Expected output:
```
PyTorch version: 2.7.1+cu118
CUDA available: True
GPU: NVIDIA GeForce RTX 4060
Memory: 8.0GB
```

### Step 4: Clone DeepSeek-OCR Repository

```bash
wsl -d Ubuntu bash -c "
cd ~/deepseek-ocr-workspace
git clone https://github.com/deepseek-ai/DeepSeek-OCR.git
cd DeepSeek-OCR
echo 'Repository cloned!'
"
```

### Step 5: First Inference (Download Model)

The first run will download the 7GB model from HuggingFace.

```bash
wsl -d Ubuntu bash -c "
cd ~/deepseek-ocr-workspace
source venv/bin/activate
python3 /mnt/c/plugin_dev/deepseek_ocr_test.py
"
```

This will:
1. Download DeepSeek-OCR model (~7GB)
2. Load on GPU
3. Run test extraction (if you provide test_image.jpg)

---

## Quick Test on Your Own Image

### Option A: From Windows

Copy your test image to:
```
C:\plugin_dev\test_image.jpg
```

Then run:
```bash
wsl -d Ubuntu bash -c "
cd ~/deepseek-ocr-workspace
source venv/bin/activate
python3 << 'EOF'
from PIL import Image
from transformers import AutoModel, AutoTokenizer
import torch

# Load model
print('Loading model...')
model = AutoModel.from_pretrained("deepseek-ai/DeepSeek-OCR", trust_remote_code=True, torch_dtype=torch.bfloat16)
tokenizer = AutoTokenizer.from_pretrained("deepseek-ai/DeepSeek-OCR", trust_remote_code=True)

# Load image from Windows
image = Image.open("/mnt/c/plugin_dev/test_image.jpg")
print(f'Image: {image.size}')

# Extract
prompt = "Extract all text from this image"
messages = [{"role": "user", "content": [{"type": "image", "image": image}, {"type": "text", "text": prompt}]}]
inputs = tokenizer(messages, return_tensors="pt")
outputs = model.generate(**inputs, max_new_tokens=512)
result = tokenizer.decode(outputs[0], skip_special_tokens=True)
print(f'Result:\n{result}')
EOF
"
```

### Option B: Run FastAPI Backend

Start the inference server:

```bash
wsl -d Ubuntu bash -c "
cd ~/deepseek-ocr-workspace
source venv/bin/activate
uvicorn /mnt/c/plugin_dev/deepseek_backend:app --reload --host 0.0.0.0 --port 8000
"
```

Then use from Windows:

```bash
# Upload image and extract
curl -X POST http://localhost:8000/extract \
  -F "file=@C:\plugin_dev\test_image.jpg" \
  -F "prompt=Extract all text"
```

Visit `http://localhost:8000/docs` for interactive API documentation.

---

## For Your Business Ideas

### 1. Construction Quote Generator

**Test extraction:**
```python
prompt = """Analyze this bathroom renovation photo. Extract:
- Room type
- Fixtures (toilets, sinks, showers, tiles, lights)
- Count of each
Return as JSON."""
```

**API endpoint:**
```bash
curl -X POST http://localhost:8000/extract-quote \
  -F "file=@bathroom.jpg"
```

### 2. Defect/Snag List

**Test extraction:**
```python
prompt = """Analyze this construction defect photo. Extract:
- Location
- Defect type
- Severity (1-5)
- Responsible trade
Return as JSON."""
```

**API endpoint:**
```bash
curl -X POST http://localhost:8000/extract-defect \
  -F "file=@defect.jpg"
```

### 3. PropSignal (Government Valuation)

**Test extraction:**
```python
prompt = """Extract property valuation data:
- Address
- Capital improved value
- Land value
- Valuation date
Return as JSON."""
```

---

## Next Phase: Send Discovery Messages

Once you have extraction working:

1. **Send messages from `C:\plugin_dev\New folder\discovery_messages.md`**
2. **Ask people to send 2-3 photos of their problem**
3. **Test DeepSeek-OCR on real photos**
4. **Measure accuracy + speed**
5. **Build MVP web app** (2-3 days)

---

## MVP Web App Stack

- **Frontend:** Next.js 14 + React (Next.js App Router)
- **Backend:** FastAPI + DeepSeek-OCR (already have `deepseek_backend.py`)
- **Database:** Supabase PostgreSQL (free tier available)
- **Storage:** Supabase Storage (5GB free)
- **Auth:** Supabase Auth (email magic link)
- **Payments:** Stripe (when you start charging)

**Timeline:** 2-3 days to build MVP

---

## Files You Have

✅ `C:\plugin_dev\deepseek_ocr_test.py` - Test script
✅ `C:\plugin_dev\deepseek_backend.py` - FastAPI backend
✅ `C:\plugin_dev\setup_deepseek.sh` - Setup automation
✅ `C:\plugin_dev\DEEPSEEK_SETUP_GUIDE.md` - Detailed guide
✅ `C:\plugin_dev\New folder\discovery_messages.md` - Discovery templates
✅ `C:\plugin_dev\New folder\interview_guide.md` - Interview framework

---

## Troubleshooting

### "CUDA out of memory"
```python
# Reduce max_tokens
outputs = model.generate(**inputs, max_new_tokens=256)  # was 1024
```

### "Model download too slow"
- WSL2 internet can be slower. Be patient or run in native Ubuntu VM.

### "First inference is slow"
- Normal (60+ seconds on first run). Subsequent runs: 5-10 seconds.

### "AttributeError: module 'transformers' has no attribute..."
```bash
pip install --upgrade transformers
```

---

## Quick Reference: Commands

```bash
# Activate environment
wsl -d Ubuntu bash -c "cd ~/deepseek-ocr-workspace && source venv/bin/activate && bash"

# Run test
python3 /mnt/c/plugin_dev/deepseek_ocr_test.py

# Start API
uvicorn /mnt/c/plugin_dev/deepseek_backend:app --host 0.0.0.0 --port 8000

# Check GPU
python3 -c "import torch; print(f'CUDA: {torch.cuda.is_available()}')"

# Test on specific image
python3 << 'EOF'
from transformers import AutoModel, AutoTokenizer
from PIL import Image
import torch
model = AutoModel.from_pretrained("deepseek-ai/DeepSeek-OCR", trust_remote_code=True, torch_dtype=torch.bfloat16)
tokenizer = AutoTokenizer.from_pretrained("deepseek-ai/DeepSeek-OCR", trust_remote_code=True)
image = Image.open("/path/to/image.jpg")
messages = [{"role": "user", "content": [{"type": "image", "image": image}, {"type": "text", "text": "Extract all text"}]}]
inputs = tokenizer(messages, return_tensors="pt")
outputs = model.generate(**inputs, max_new_tokens=512)
print(tokenizer.decode(outputs[0], skip_special_tokens=True))
EOF
```

---

## Next Steps

1. ⏳ Wait for PyTorch to finish (~5-10 min)
2. ✅ Run Step 1-5 above
3. 📸 Gather test images (send discovery messages)
4. 🧪 Test extraction on real photos
5. 🚀 Build MVP web app
6. 💰 Start validation with paying customers

**You got this! 🎉**
