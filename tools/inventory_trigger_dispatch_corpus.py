"""Draft real owner inventory mutations; no synthetic criterion invocation."""
import hashlib
import uuid

PREFIX='inventory_dispatch'
ADVANCEMENTS={
 f'{PREFIX}:single':{'criteria':{'seen':{'trigger':'minecraft:inventory_changed','conditions':{'items':[{'items':'minecraft:diamond'}]}}}},
 f'{PREFIX}:multi':{'criteria':{'seen':{'trigger':'minecraft:inventory_changed','conditions':{'items':[{'items':'minecraft:diamond'},{'items':'minecraft:dirt'}]}}}},
}

def program():
 ops=[]
 def cmd(text,label,expected=None,admission=True,category='control',record=True):
  op=dict(op='cmd',cmd=text,label=label,admission=admission,category=category,record=record)
  if expected is not None:op['expect_result']=expected
  ops.append(op)
 def fresh(label,text,expected=1,admission=True,category='control'):
  for k in ['result','success']:
   cmd(f'/scoreboard players set {k} invdispatch -99',label+'-reset-'+k,-99,record=False)
  cmd('/execute store result score result invdispatch store success score success invdispatch run '+text,label+'-action',expected,admission,category)
  cmd('/scoreboard players get result invdispatch',label+'-result',expected,admission,category)
  cmd('/scoreboard players get success invdispatch',label+'-success',1,admission,category)
 def progress(label,single,multi,admission=True):
  for name,value in [('single',single),('multi',multi)]:
   sign='if' if value else 'unless'
   fresh(label+'-'+name,f'execute {sign} entity @s[advancements={{{PREFIX}:{name}={{seen=true}}}}]',1,admission,'control' if admission else 'dispatch')
 def inv(label,slots):
  cmd('/data get entity @s Inventory',label+'-full-inventory',admission=False,category='raw')
  for slot,item,count in slots:
   path=f'Inventory[{{Slot:{slot}b,id:"minecraft:{item}",count:{count}}}]'
   cmd('/execute if data entity @s '+path,label+'-typed-slot'+str(slot),1)
   cmd('/data get entity @s '+path+'.count',label+'-count-slot'+str(slot),count)
 cmd('/tick freeze','freeze',1)
 for text,label in [('gamerule spawn_mobs false','spawn'),('gamerule advance_time false','time'),('gamerule announce_advancements false','announce'),('gamerule show_advancement_messages false','display')]:cmd('/'+text,label)
 cmd('/forceload add 0 0','force',1);ops.append(dict(op='settle',required_chunks=[[0,0]]))
 cmd('/fill 0 199 0 15 199 15 stone','floor',256)
 cmd('/tp @s 4.5 200.0 4.5 0 0','position',1)
 cmd('/gamemode creative @s','mode')
 u=uuid.UUID(bytes=hashlib.md5(b'OfflinePlayer:Player').digest(),version=3)
 words=','.join(str(int.from_bytes(u.bytes[i:i+4],'big',signed=True)) for i in range(0,16,4))
 guard=f'@s[type=player,name=Player,nbt={{UUID:[I;{words}],Health:20f,playerGameType:1,Pos:[4.5d,200d,4.5d]}}]'
 cmd('/execute if entity '+guard,'owner',1)
 cmd('/data get entity @s UUID','owner-uuid',4)
 cmd('/data get entity @s','owner-full',admission=False,category='raw')
 cmd('/execute if loaded 4 200 4','loaded',1)
 cmd('/execute if block 4 199 4 stone','support',1)
 cmd('/scoreboard objectives add invdispatch dummy','objective',1)
 cmd('/datapack list enabled','packs',admission=False,category='raw')
 cmd('/execute store result score start invdispatch run time query gametime','clock-start',admission=False,category='raw')
 cmd('/clear @s','clear')
 cmd('/execute unless data entity @s Inventory[]','inventory-empty',1)
 # Exact named-criterion grant/revoke proves both definitions resolved to this actual owner.
 for name in ['single','multi']:
  fresh('definition-'+name,'advancement grant @s only '+PREFIX+':'+name+' seen')
  fresh('definition-revoke-'+name,'advancement revoke @s only '+PREFIX+':'+name+' seen')
 progress('empty',False,False)
 fresh('matching-first','item replace entity @s inventory.0 with diamond 1')
 inv('diamond',[(9,'diamond',1)])
 progress('natural-single',True,False)
 fresh('multi-second-item','item replace entity @s inventory.1 with dirt 1')
 inv('two-matching',[(9,'diamond',1),(10,'dirt',1)])
 progress('natural-multi',True,True)
 for name in ['single','multi']:fresh('revoke-'+name,'advancement revoke @s only '+PREFIX+':'+name+' seen')
 progress('revoked-retained',False,False)
 fresh('unrelated-change','item replace entity @s inventory.2 with stone 1')
 inv('after-unrelated',[(9,'diamond',1),(10,'dirt',1),(11,'stone',1)])
 # Assigned rows remain strict; multi is independently required to regrant.
 progress('after-unrelated',False,True,False)
 for op in ops:
  if op.get('label','').startswith('after-unrelated-multi-'):
   op['admission']=True;op['category']='control'
 fresh('matching-again','item replace entity @s inventory.2 with diamond 1')
 inv('after-matching',[(9,'diamond',1),(10,'dirt',1),(11,'diamond',1)])
 progress('matching-changed-positive',True,True)
 cmd('/execute if entity '+guard,'owner-final',1)
 cmd('/data get entity @s','owner-final-full',admission=False,category='raw')
 cmd('/execute store result score end invdispatch run time query gametime','clock-end',admission=False,category='raw')
 cmd('/execute if score start invdispatch = end invdispatch','relative-clock-zero',1)
 cmd('/scoreboard objectives remove invdispatch','cleanup',0)
 cmd('/tick unfreeze','unfreeze',0)
 return ops
