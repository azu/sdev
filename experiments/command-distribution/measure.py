import hashlib, json, os, platform, random, statistics, subprocess, sys, time
from pathlib import Path

base = Path(__file__).resolve().parent
root = base.parent.parent
with_rust = '--rust' in sys.argv
binaries = {'bun': base/'dist/bun-host', ('rust' if with_rust else 'scriptc-dynamic'): base/('dist/rust-host' if with_rust else 'dist/scriptc-host')}
inputs = [root/'docs/README.md', root/'docs/products/sdev/product.md']
cases = {'version': ['--version'], 'help': ['--help'], 'docs': ['docs', 'docs/templates/product.md'], 'markdown': ['check', *map(str, inputs)]}
samples = {name:{case:[] for case in cases} for name in binaries}
rng = random.Random(17)
env = {'PATH':'/usr/bin:/bin', 'HOME':os.environ['HOME']}

def invoke(name, args, capture=False):
    return subprocess.run([str(binaries[name]),*args],cwd='/private/tmp',env=env,stdout=subprocess.PIPE if capture else subprocess.DEVNULL,stderr=subprocess.PIPE,check=True)

def describe(values):
    ordered=sorted(values)
    return {'n':len(ordered),'median_ms':round(statistics.median(ordered),6),'p10_ms':round(ordered[5],6),'p90_ms':round(ordered[53],6),'batch_medians_ms':[round(statistics.median(values[i:i+20]),6) for i in range(0,60,20)]}

for case,args in cases.items():
    for name in binaries:
        for _ in range(5): invoke(name,args)
    for _ in range(60):
        order=list(binaries);rng.shuffle(order)
        for name in order:
            start=time.perf_counter_ns()
            invoke(name,args)
            samples[name][case].append((time.perf_counter_ns()-start)/1e6)

result={'python':sys.version,'host':platform.platform(),'machine':platform.machine(),'method':'5 warmups, 60 alternating randomized fresh processes per binary/case; wall clock includes spawn and output, warm file cache; 3 batches of 20; no cold cache claim','inputs':[{'path':str(p.relative_to(root)),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in inputs],'summary':{name:{case:describe(values) for case,values in groups.items()} for name,groups in samples.items()},'bytes':{name:binary.stat().st_size for name,binary in binaries.items()},'binary_sha256':{name:hashlib.sha256(binary.read_bytes()).hexdigest() for name,binary in binaries.items()},'samples_ms':samples}
if with_rust:
    stages=['read_ms','parse_ms','references_ms','total_ms']
    internal={name:{stage:[] for stage in stages} for name in binaries}
    profile_args=['profile',*map(str,inputs)]
    counts=None
    for name in binaries:
        for _ in range(5):invoke(name,profile_args,True)
    for _ in range(60):
        order=list(binaries);rng.shuffle(order)
        for name in order:
            value=json.loads(invoke(name,profile_args,True).stdout)
            current={key:value[key] for key in ['files','links','missing']}
            if counts is None: counts=current
            assert current==counts, (name,current,counts)
            for stage in stages:internal[name][stage].append(value[stage])
    result['internal']={'method':'Separate profile command, 5 warmups and 60 alternating fresh processes; stage clocks exclude startup, library initialization, dispatch, and JSON serialization/stdout; no clocks in normal check','counts':counts,'summary':{name:{stage:describe(values) for stage,values in groups.items()} for name,groups in internal.items()},'samples_ms':internal}
output=base/('results/startup-rust.json' if with_rust else 'results/startup.json')
output.write_text(json.dumps(result,indent=2))
print(json.dumps({key:value for key,value in result.items() if key not in ['samples_ms','internal']},indent=2))
if with_rust:print(json.dumps({'internal':{key:value for key,value in result['internal'].items() if key!='samples_ms'}},indent=2))
