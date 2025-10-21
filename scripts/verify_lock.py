
#!/usr/bin/env python3
import hashlib, os, sys
ROOT = os.path.dirname(os.path.dirname(__file__))
Z = os.path.join(ROOT, "plugins", "EngineField", "Source", "dsp", "ZPlaneFilter.h")
T = os.path.join(ROOT, "plugins", "EngineField", "Source", "dsp", "EMUAuthenticTables.h")
EXP = {"ZPlaneFilter.h":"dbfe0b15ba73ab1a44a698e31617fecb0a71cb622dc10c7770994f55e3554f33", "EMUAuthenticTables.h":"b3bd58a897b16065c12faa3e2b966f0ac9a28581231ece2be599e19d14cf3756"}
def sha(p): return hashlib.sha256(open(p,"rb").read()).hexdigest()
bad = []
if sha(Z) != EXP["ZPlaneFilter.h"]: bad.append(("ZPlaneFilter.h", sha(Z)))
if sha(T) != EXP["EMUAuthenticTables.h"]: bad.append(("EMUAuthenticTables.h", sha(T)))
if bad:
    print("DSP LOCK VIOLATION:")
    for name,h in bad: print(f"  - {name}: {h} (expected {EXP[name]})")
    sys.exit(2)
print("DSP lock OK.")
