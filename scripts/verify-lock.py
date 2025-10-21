#!/usr/bin/env python3
import hashlib, sys, json, os

ROOT = os.path.dirname(os.path.dirname(__file__))
HASHES = {
  "plugins/field/Source/dsp/EMUAuthenticTables.h": "06fbc6e2fd27db087219f0d9f53f9962997f8cb8001490458f6053bc2e46aa86",
  "plugins/field/Source/dsp/ZPlaneFilter.h": "d73839b6852f7f5d8f135b18cea9c8f68c89962e33342b91bdea99607003e821",
  "plugins/field/Source/dsp/EnvelopeFollower.h": "81d97f6fcbe3547cf319a47205c7ab560c5691c55d8654091ac12b0a6318ba07"
}

def sha256_file(p):
    h=hashlib.sha256()
    with open(p,'rb') as f:
        while True:
            b=f.read(8192)
            if not b: break
            h.update(b)
    return h.hexdigest()

def main():
    ok = True
    for rel, expected in HASHES.items():
        p = os.path.join(ROOT, rel)
        if not os.path.exists(p):
            print(f"[LOCK] Missing locked file: {rel}")
            ok = False
            continue
        h = sha256_file(p)
        if h != expected:
            print(f"[LOCK] Hash mismatch for {rel}")
            print(f"       expected: {expected}")
            print(f"       got     : {h}")
            ok = False
    if not ok:
        print("\nLOCK FAILED: DSP files modified. Revert changes or update hashes after re‑validation.")
        return 2
    print("LOCK OK")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
