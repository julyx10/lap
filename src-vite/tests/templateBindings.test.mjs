import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { parse, compileScript, compileTemplate } from '@vue/compiler-sfc';

const srcDir = fileURLToPath(new URL('../src', import.meta.url));

// Template identifiers the script does not declare compile to `_ctx.name`
// and render as undefined at runtime; `vite build` does not catch them.
function unresolvedTemplateIdentifiers(file) {
  const { descriptor } = parse(readFileSync(file, 'utf8'), { filename: file });
  if (!descriptor.template || !descriptor.scriptSetup) return [];
  const script = compileScript(descriptor, { id: 'test', inlineTemplate: false });
  const { code } = compileTemplate({
    source: descriptor.template.content,
    filename: file,
    id: 'test',
    compilerOptions: { bindingMetadata: script.bindings, prefixIdentifiers: true },
  });
  const names = [...code.matchAll(/_ctx\.([A-Za-z_$][\w$]*)/g)].map(match => match[1]);
  // $t, $emit, $slots, ... are component instance globals.
  return [...new Set(names)].filter(name => !name.startsWith('$'));
}

function vueFiles(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap(entry => {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) return vueFiles(full);
    return entry.name.endsWith('.vue') ? [full] : [];
  });
}

test('settings controls are bound to declared state, including the Tianditu token', () => {
  const file = path.join(srcDir, 'views/Settings.vue');
  assert.deepEqual(unresolvedTemplateIdentifiers(file), []);

  const { descriptor } = parse(readFileSync(file, 'utf8'), { filename: file });
  const { bindings } = compileScript(descriptor, { id: 'test', inlineTemplate: false });
  for (const name of [
    'tiandituTokenInput', 'tiandituTokenStatus', 'tiandituTokenStatusLabel', 'tiandituTokenStatusClass',
    'isOmlxProvider', 'omlxConnectionMessage', 'embeddingCoverageText', 'reindexStatus',
  ]) {
    assert.ok(bindings[name], `${name} must be declared in Settings.vue`);
  }
});

test('every component template only references declared bindings', () => {
  const problems = vueFiles(srcDir)
    .map(file => [path.relative(srcDir, file), unresolvedTemplateIdentifiers(file)])
    .filter(([, names]) => names.length > 0);
  assert.deepEqual(problems, []);
});
