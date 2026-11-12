import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import ts from 'typescript';
import { computed, ref, watch } from 'vue';

const source = await readFile(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
function loadFunction(name, next, values) {
  const start = source.indexOf(`async function ${name}(`);
  const code = ts.transpile(source.slice(start, source.indexOf(next, start)), { target: ts.ScriptTarget.ES2022 });
  return new Function(...Object.keys(values), `${code}; return ${name};`)(...Object.values(values));
}

function fixture({ show = true, grid = true, hydrated = true } = {}) {
  const values = {
    config: { rightPanel: { show, mode: 'info' } },
    isGridLayoutView: ref(grid), selectionPanelOpen: ref(false), selectMode: ref(false),
    selectedItemIndex: ref(0), lastSelectedIndex: ref(0), showQuickView: ref(false),
    fileList: ref([{ id: 1 }, { id: 2 }, { id: 3 }]),
    groupSelectedCountMap: ref({}), groupSelectedSizeMap: ref({}),
    hydrateSelectionRange: async () => hydrated,
    clearLoadedSelectionFlags() {}, resetSelectionSummary() {}, stopSlideShow() {},
    isRealFileItem: item => !!item?.id, ensureGroupedFileAtIndex: () => true,
    toast: { error() {} }, t: key => key,
    setItemSelected(index, selected) { values.fileList.value[index].isSelected = selected; },
  };
  return values;
}

test('Shift range selection preserves an open info or dedup panel', async () => {
  for (const mode of ['info', 'dedup']) {
    const f = fixture();
    f.config.rightPanel.mode = mode;
    const select = loadFunction('selectRangeFromSingleSelection', 'async function handleItemSelectToggled', f);
    await select(0, 2);
    assert.equal(f.selectMode.value, true);
    assert.equal(f.selectionPanelOpen.value, false);
    assert.equal(f.fileList.value.every(file => file.isSelected), true);
  }
});

test('a hidden panel, filmstrip, or failed range load does not open the selection panel', async () => {
  for (const options of [{ show: false }, { grid: false }, { hydrated: false }]) {
    const f = fixture(options);
    const select = loadFunction('selectRangeFromSingleSelection', 'async function handleItemSelectToggled', f);
    await select(0, 2);
    assert.equal(f.selectionPanelOpen.value, false);
  }
});

test('Shift range selection keeps the floating state when selection mode is already active', async () => {
  const f = fixture();
  f.selectMode.value = true;
  const toggle = loadFunction('handleItemSelectToggled', 'function toggleKeyboardSelection', f);
  await toggle(2, true);
  assert.equal(f.selectionPanelOpen.value, false);
});

test('Escape closes the selection panel without restoring info or dedup', () => {
  const start = source.indexOf("    if (selectMode.value) {", source.indexOf("  if (matchesShortcut('view.close', event"));
  const end = source.indexOf("    if (tempViewMode.value", start);
  const code = source.slice(start, end);
  for (const mode of ['info', 'dedup']) {
    const selectMode = ref(true);
    const selectionPanelOpen = ref(true);
    const config = { rightPanel: { show: true, mode } };
    watch(selectMode, active => { if (!active) selectionPanelOpen.value = false; }, { flush: 'sync' });
    let prevented = false;
    new Function('selectMode', 'selectionPanelOpen', 'config', 'handleSelectMode', 'event', code)(
      selectMode, selectionPanelOpen, config, value => { selectMode.value = value; },
      { preventDefault() { prevented = true; } },
    );
    assert.equal(config.rightPanel.show, false);
    assert.equal(selectMode.value, false);
    assert.equal(selectionPanelOpen.value, false);
    assert.equal(prevented, true);
  }
});

test('info and dedup buttons preserve selection mode, including when their panel is already visible', async () => {
  const start = source.indexOf('const toggleInfoPanel = () => {');
  const end = source.indexOf('async function openDedupPanel()', start);
  const code = ts.transpile(source.slice(start, end), { target: ts.ScriptTarget.ES2022 });
  for (const mode of ['info', 'dedup']) {
    for (const panelVisible of [false, true]) {
      const f = fixture();
      f.config.rightPanel.mode = mode;
      f.selectMode.value = true;
      f.selectionPanelOpen.value = !panelVisible;
      let pending;
      const values = {
        ...f,
        isScanStreamingMode: ref(false),
        isInfoPanelOpen: computed(() => !f.selectionPanelOpen.value && f.config.rightPanel.show && f.config.rightPanel.mode === 'info'),
        isDedupPanelOpen: computed(() => !f.selectionPanelOpen.value && f.config.rightPanel.show && f.config.rightPanel.mode === 'dedup'),
        getActivePreviewMediaRef: () => null,
        checkUnsavedChanges(action) { pending = action(); },
        handleSelectMode(value) { f.selectMode.value = value; f.selectionPanelOpen.value = false; },
        openDedupPanel() { f.selectionPanelOpen.value = false; f.config.rightPanel.mode = 'dedup'; f.config.rightPanel.show = true; },
      };
      const buttons = new Function(...Object.keys(values), `${code}; return { toggleInfoPanel, toggleDedupPanel };`)(...Object.values(values));
      buttons[mode === 'info' ? 'toggleInfoPanel' : 'toggleDedupPanel']();
      await pending;
      assert.equal(f.selectMode.value, true);
      assert.equal(f.selectionPanelOpen.value, false);
      assert.equal(f.config.rightPanel.show, !panelVisible);
      assert.equal(f.config.rightPanel.mode, mode);
    }
  }
});


test('Ctrl/Cmd click preserves an open panel, including a click on the current item', async () => {
  for (const modifier of ['ctrlKey', 'metaKey']) {
    for (const index of [0, 1]) {
      const f = fixture();
      f.checkUnsavedChanges = action => action();
      f.handleSelectMode = active => {f.selectMode.value = active;};
      f.handleItemSelectToggled = loadFunction('handleItemSelectToggled', 'function toggleKeyboardSelection', f);
      const start = source.indexOf('function handleItemClicked(');
      const code = ts.transpile(source.slice(start, source.indexOf('function handleItemDblClicked', start)), {target:ts.ScriptTarget.ES2022});
      const click = new Function(...Object.keys(f), `${code}; return handleItemClicked;`)(...Object.values(f));
      click(index, {[modifier]:true});
      assert.equal(f.selectMode.value, true);
      assert.equal(f.selectionPanelOpen.value, false);
    }
  }
});

test('select all preserves the existing panel for both flat and grouped lists', async () => {
  const start = source.indexOf('const selectAllInCurrentList = async () => {');
  const code = ts.transpile(source.slice(start, source.indexOf('const selectNoneInCurrentList', start)), {target:ts.ScriptTarget.ES2022});
  for (const grouped of [false, true]) {
    for (const options of [{}, {show:false}, {grid:false}]) {
      const f = {...fixture(options), groupedModeActive:ref(grouped), selectedFileIds:new Set(),
        getCurrentQueryFileIds:async()=>[1,2,3], groupMetaMap:new Map(), selectedCount:ref(0), selectedSize:ref(0),
        totalFileCount:ref(3), totalFileSize:ref(0), syncSelectionVersions(){}, isPendingFileItem:()=>false,
        getMediaItemSize: file => Number(file?.size ?? 0) + Number(file?.companion_size ?? 0),
        recomputeLoadedGroupSelectedCounts(){}};
      const select = new Function(...Object.keys(f), `${code}; return selectAllInCurrentList;`)(...Object.values(f));
      await select();
      assert.equal(f.selectMode.value, true);
      assert.equal(f.selectionPanelOpen.value, false);
      assert.equal(f.selectedCount.value, 3);
    }
  }
});
