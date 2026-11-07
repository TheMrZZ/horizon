#!/usr/bin/env python3
"""Command removal bridge: strict saved fields and an explicit full diagnostic."""
import json,sys,hashlib,tempfile
from pathlib import Path
import test_command_exec_parity as driver
CASES=[
 dict(name='helmet_plain',item='iron_helmet',count=1,id='minecraft:iron_helmet',marker='max_damage'),
 dict(name='helmet_custom',item='iron_helmet[custom_data={audit:17}]',count=1,id='minecraft:iron_helmet',marker='max_damage',custom=True),
 dict(name='helmet_kept',item='iron_helmet[max_damage=166,custom_data={audit:17}]',count=1,id='minecraft:iron_helmet',marker='max_damage',custom=True,damage=166),
 dict(name='helmet_removed',item='iron_helmet[!max_damage,custom_data={audit:17}]',count=1,id='minecraft:iron_helmet',marker='max_damage',custom=True,removed=True),
 dict(name='helmet_namespaced',item='minecraft:iron_helmet[!minecraft:max_damage,minecraft:custom_data={audit:17}]',count=1,id='minecraft:iron_helmet',marker='max_damage',custom=True,removed=True),
 dict(name='helmet_removed_two',item='iron_helmet[!max_damage,max_stack_size=2,custom_data={audit:17}]',count=2,id='minecraft:iron_helmet',marker='max_damage',custom=True,removed=True,size=2),
 dict(name='stone_plain',item='stone',count=1,id='minecraft:stone',marker='max_stack_size'),
 dict(name='stone_removed',item='stone[!max_stack_size,custom_data={audit:17}]',count=1,id='minecraft:stone',marker='max_stack_size',custom=True,removed=True),
 dict(name='count_rejected',item='iron_helmet',count=2,id='minecraft:emerald',marker='max_damage',rejected=True),
]
INVALID=[dict(name='h086_helmet_two',item='iron_helmet[max_stack_size=2]',count=1,id='minecraft:emerald',marker='max_damage',rejected=True),dict(name='h086_helmet_99',item='iron_helmet[max_stack_size=99]',count=1,id='minecraft:emerald',marker='max_damage',rejected=True),dict(name='h086_stone_damageable',item='stone[max_damage=10,max_stack_size=2]',count=1,id='minecraft:emerald',marker='max_damage',rejected=True)]
def program(full=False):
 ops=[]
 def c(cmd,label,result=None,admission=True,raw=False,java_results=None):
  o=dict(op='cmd',cmd=cmd,label=label,admission=admission)
  if result is not None:o['expect_result']=result
  if raw:o.update(record=full,trace=True)
  if java_results is not None:o['java_results']=java_results
  ops.append(o)
 c('/tick freeze','freeze');c('/gamerule spawn_mobs false','no-spawn');c('/gamerule advance_time false','no-daylight');c('/forceload add 0 0','force',1);ops.append(dict(op='settle',required_chunks=[[0,0]]));c('/fill 0 199 0 15 199 15 stone','floor',256);c('/setblock 4 200 4 chest[facing=north,type=single,waterlogged=false]','chest-place',1);c('/tp @s 12.5 200 12.5 0 0','owner-pos',1);c('/gamemode creative @s','owner-mode');c('/data get entity @s UUID','owner-uuid',4);c('/execute if entity @s[type=player,nbt={Health:20f,playerGameType:1,Pos:[12.5d,200d,12.5d]}]','owner-body',1);c('/execute if loaded 4 200 4','loaded',1);c('/execute if block 4 199 4 stone','support',1);c('/execute if block 4 200 4 chest[facing=north,type=single,waterlogged=false]','chest-state',1)
 for field,result in [('id',15),('x',4),('y',200),('z',4),('Items',0)]:c('/data get block 4 200 4 '+field,'chest-'+field,result)
 c('/scoreboard objectives add bridge dummy','objective');c('/execute store result score start bridge run time query gametime','clock-start',raw=True)
 c('/item replace block 4 200 4 container.1 with stone[custom_data={keep:23}] 7','unrelated-place',1)
 base='Items[{Slot:0b}]'
 for case in CASES+(INVALID if full else []):
  name=case['name'];diagnostic=name.startswith('h086_');removed=case.get('removed',False);rejected=case.get('rejected',False)
  c('/item replace block 4 200 4 container.0 with emerald[custom_data={sentinel:1}] 3',name+'-sentinel-place',1)
  c('/execute if data block 4 200 4 Items[{Slot:0b,id:"minecraft:emerald",count:3,components:{"minecraft:custom_data":{sentinel:1}}}]',name+'-sentinel-before',1)
  for k in ['result','success']:c(f'/scoreboard players set {k} bridge -99',name+'-reset-'+k,-99)
  c(f'/execute store result score result bridge store success score success bridge run item replace block 4 200 4 container.0 with {case["item"]} {case["count"]}',name+'-command',admission=False,java_results=[[False,0]] if rejected else [[True,1]])
  for k in ['result','success']:c(f'/scoreboard players get {k} bridge',name+'-'+k,admission=not diagnostic)
  count=3 if rejected else case['count']
  c(f'/execute if data block 4 200 4 Items[{{Slot:0b,id:"{case["id"]}",count:{count}}}]',name+'-saved-identity',1,admission=not diagnostic)
  c(f'/data get block 4 200 4 {base}.id',name+'-saved-id',len(case['id']),admission=not diagnostic)
  c(f'/data get block 4 200 4 {base}.count',name+'-saved-count',count,admission=not diagnostic)
  marker=base+'.components."!minecraft:'+case['marker']+'"'
  c('/execute if data block 4 200 4 '+marker,name+'-marker-presence',admission=False,java_results=[[True,1]] if removed else [[False,0]])
  for k in ['result','success']:c(f'/scoreboard players set {k} bridge -99',name+'-marker-reset-'+k,-99)
  c('/execute store result score result bridge store success score success bridge run data get block 4 200 4 '+marker,name+'-marker-value',admission=False,java_results=[[True,0]] if removed else [[False,0]])
  for k in ['result','success']:c(f'/scoreboard players get {k} bridge',name+'-marker-'+k,admission=False)
  typed='Items[{Slot:0b,components:{"!minecraft:'+case['marker']+'":{}}}]'
  c('/execute if data block 4 200 4 '+typed,name+'-marker-compound',admission=False,java_results=[[True,1]] if removed else [[False,0]])
  c(f'/data get block 4 200 4 {base}.components."minecraft:custom_data"',name+'-custom-data',admission=not diagnostic,java_results=[[True,1]] if case.get('custom') or rejected else [[False,0]])
  c(f'/data get block 4 200 4 {base}.components."minecraft:max_damage"',name+'-kept-damage',admission=not diagnostic,java_results=[[True,166]] if case.get('damage') else [[False,0]])
  c(f'/data get block 4 200 4 {base}.components."minecraft:max_stack_size"',name+'-kept-size',admission=not diagnostic,java_results=[[True,2]] if case.get('size') else [[False,0]])
  c('/execute if data block 4 200 4 Items[{Slot:1b,id:"minecraft:stone",count:7,components:{"minecraft:custom_data":{keep:23}}}]',name+'-unrelated-unchanged',1)
  c('/data get block 4 200 4 '+base,name+'-full-stack',admission=False,raw=True);c('/data get block 4 200 4 '+base+'.components',name+'-full-components',admission=False,raw=True)
 c('/execute store result score end bridge run time query gametime','clock-end',raw=True);c('/scoreboard players operation end bridge -= start bridge','clock-delta-operation',0);c('/scoreboard players get end bridge','clock-delta',0)
 return ops
if __name__=='__main__':
 full='--full-diagnostic' in sys.argv
 if full:sys.argv.remove('--full-diagnostic')
 reference=None
 if '--reference' in sys.argv:
  i=sys.argv.index('--reference');reference=Path(sys.argv[i+1]).resolve();del sys.argv[i:i+2]
 if '--work-dir' not in sys.argv:
  parent=Path(tempfile.mkdtemp(prefix='item-removal-',dir=driver.lab.LOCAL));sys.argv+=['--work-dir',str(parent/'run')]
 build=Path(sys.argv[sys.argv.index('--work-dir')+1]).resolve();assert not build.exists(),'preserve attempts';build.mkdir(parents=True)
 ops=program(full);recorded=[o for o in ops if o['op']=='cmd' and o.get('record',True)]
 (build/'declaration.json').write_text(json.dumps(dict(script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),full_diagnostic=full,records=len(recorded),labels=[o['label'] for o in recorded],cases=CASES,invalid_controls=INVALID if full else [],scope='default strict valid bridge plus count rejection; full includes all raw compound presentations/clocks and H086 expected rejection/state/store records, no normalization'),indent=2)+'\n')
 driver.corpus_module.groups=lambda:[('Item command removal patch',ops)];expected=reference.read_bytes() if reference else None
 sys.argv.append('--allow-gaps');driver.main();jb=(build/'vanilla.jsonl').read_bytes()
 if expected is not None:assert jb==expected and reference.read_bytes()==expected,'reference bytes changed'
 java=[json.loads(s) for s in jb.splitlines()];assert len(java)==len(recorded)
 for o,row in zip(recorded,java):
  if 'java_results' in o:assert row['results']==o['java_results'],('Java prerequisite',o,row)
 if '--reference-only' not in sys.argv:
  native=[json.loads(s) for s in (build/'rust.jsonl').read_bytes().splitlines()];assert len(native)==len(recorded)
  residuals=[dict(label=o['label'],expected=a,actual=b) for o,a,b in zip(recorded,java,native) if a!=b];r=dict(records=len(recorded),passed=len(recorded)-len(residuals),residuals=residuals,full_diagnostic=full,reference_sha256=hashlib.sha256(jb).hexdigest());(build/'strict-report.json').write_text(json.dumps(r,indent=2)+'\n')
  if not full:
   for o,a,b in zip(recorded,java,native):
    if o['admission']:assert a==b,('admission',o,a,b)
  print(f'Strict records: {r["passed"]}/{r["records"]}')
  if residuals:raise SystemExit(1)
