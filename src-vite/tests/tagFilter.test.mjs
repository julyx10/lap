import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
const source = fs.readFileSync(new URL('../src/common/tagFilter.ts', import.meta.url), 'utf8');
const js = ts.transpile(source.replace(/export /g, ''), {target:ts.ScriptTarget.ES2022});
const {selectFilterTag, exitTagFilter, enterTagFilter} = new Function(`${js};return {selectFilterTag,exitTagFilter,enterTagFilter};`)();
test('modifier enters AND mode and keeps the existing tag', () => {
 const state={id:1,multi:false,selectedIds:[],matchAny:true};
 selectFilterTag(state,2,true);
 assert.deepEqual(state,{id:2,multi:true,selectedIds:[1,2],matchAny:false,previousId:1});
 selectFilterTag(state,1); assert.deepEqual(state.selectedIds,[2]);
 selectFilterTag(state,2); assert.deepEqual(state.selectedIds,[]);
 exitTagFilter(state); assert.equal(state.id,1);assert.equal(state.multi,false);
});
test('ordinary click stays single and OR mode survives toggles', () => {
 const state={id:1,multi:false,selectedIds:[],matchAny:false};
 selectFilterTag(state,3); assert.equal(state.multi,false);assert.equal(state.id,3);
 selectFilterTag(state,4,true);state.matchAny=true;selectFilterTag(state,5);
 assert.equal(state.matchAny,true);assert.deepEqual(state.selectedIds,[3,4,5]);
 exitTagFilter(state);assert.equal(state.id,3);
});

test('Escape clears a focused search first, then exits and restores the original tag', async () => {
 const source=fs.readFileSync(new URL('../src/components/Tag.vue',import.meta.url),'utf8');
 const start=source.indexOf('async function handleFilterEscape(');
 const end=source.indexOf('function selectTag(',start);
 const js=ts.transpile(source.slice(start,end),{target:ts.ScriptTarget.ES2022});
 const state={id:5,multi:true,selectedIds:[3,5],matchAny:false,previousId:3};
 const search={value:'travel'};const input={};
 const handle=new Function('libConfig','editor','document','search','searchInputRef','exitMultiFilter','uiStore','config','SIDEBAR',`const disposed=false;${js};return handleFilterEscape;`)(
  {tag:state,activePane:'main'},{value:null},{querySelector:()=>null,activeElement:input},search,{value:input},()=>exitTagFilter(state),{inputStack:[],activePane:'left-sidebar'},{main:{sidebarIndex:1}},{TAG:1});
 const event={key:'Escape',preventDefault(){},stopImmediatePropagation(){}};
 await handle(event);assert.equal(search.value,'');assert.equal(state.multi,true);
 await handle(event);assert.equal(state.multi,false);assert.equal(state.id,3);
});

test('repeated exit does not clear the restored single tag', () => {
 const state={id:2,multi:true,selectedIds:[1,2],matchAny:false,previousId:1};
 exitTagFilter(state);exitTagFilter(state);
 assert.equal(state.id,1);assert.deepEqual(state.selectedIds,[]);
});

test('explicit entry selects the single tag without toggling it off', () => {
 const state={id:8,multi:false,selectedIds:[],matchAny:true};
 enterTagFilter(state);assert.deepEqual(state.selectedIds,[8]);assert.equal(state.matchAny,false);
 selectFilterTag(state,9);enterTagFilter(state);assert.deepEqual(state.selectedIds,[8,9]);
 exitTagFilter(state);assert.equal(state.id,8);
});

test('preview consumption and modal/fullscreen contexts take priority over tag Escape', async () => {
 const source=fs.readFileSync(new URL('../src/components/Tag.vue',import.meta.url),'utf8');
 const start=source.indexOf('async function handleFilterEscape(');
 const end=source.indexOf('function selectTag(',start);
 const js=ts.transpile(source.slice(start,end),{target:ts.ScriptTarget.ES2022});
 const state={multi:true};const ui={inputStack:[],activePane:'left-sidebar'};const doc={querySelector:()=>null,fullscreenElement:null};
 let exits=0;
 const handle=new Function('libConfig','editor','document','search','searchInputRef','exitMultiFilter','uiStore','config','SIDEBAR',`const disposed=false;${js};return handleFilterEscape;`)(
  {tag:state,activePane:'main'},{value:null},doc,{value:''},{value:null},()=>exits++,ui,{main:{sidebarIndex:1}},{TAG:1});
 const event={key:'Escape',defaultPrevented:false,preventDefault(){this.defaultPrevented=true;}};
 const pending=handle(event);
 event.preventDefault(); // A later window handler closed Preview.
 await pending;assert.equal(exits,0);
 ui.inputStack.push('modal');await handle({...event,defaultPrevented:false});assert.equal(exits,0);
 ui.inputStack=[];doc.fullscreenElement={};await handle({...event,defaultPrevented:false});assert.equal(exits,0);
 doc.fullscreenElement=null;ui.activePane='content';
 await handle({...event,defaultPrevented:false});assert.equal(exits,0);
 ui.activePane='left-sidebar';await handle({...event,defaultPrevented:false});assert.equal(exits,1);
});

test('Tag pointer and focus activation enable Escape after leaving content', async () => {
 const source=fs.readFileSync(new URL('../src/components/Tag.vue',import.meta.url),'utf8');
 assert.match(source, /@pointerdown\.capture="activateTagPanel"/);
 assert.match(source, /@focusin="activateTagPanel"/);
 const ui={activePane:'content',inputStack:[],setActivePane(value){this.activePane=value;}};
 const lib={activePane:'collection',tag:{multi:true}};
 const activateSource=source.slice(source.indexOf('function activateTagPanel('),source.indexOf('function isTagSelected('));
 const activate=new Function('uiStore','libConfig',`${activateSource};return activateTagPanel;`)(ui,lib);
 activate();assert.equal(ui.activePane,'left-sidebar');assert.equal(lib.activePane,'main');
 const start=source.indexOf('async function handleFilterEscape(');
 const js=ts.transpile(source.slice(start,source.indexOf('function selectTag(',start)),{target:ts.ScriptTarget.ES2022});
 let exited=false;
 const handle=new Function('uiStore','libConfig','config','SIDEBAR','editor','document','search','searchInputRef','exitMultiFilter',`const disposed=false;${js};return handleFilterEscape;`)(
  ui,lib,{main:{sidebarIndex:1}},{TAG:1},{value:null},{querySelector:()=>null,fullscreenElement:null},{value:''},{value:null},()=>{exited=true;});
 await handle({key:'Escape',defaultPrevented:false,preventDefault(){}});
 assert.equal(exited,true);
});
