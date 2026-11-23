import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import ts from 'typescript';

// Run with PLAYWRIGHT_MODULE pointing to an installed Playwright package.
const { chromium } = createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE || 'playwright');

test('native Escape dispatch gives preview priority and rechecks deferred state', async () => {
  const source = fs.readFileSync(new URL('../src/components/Tag.vue', import.meta.url), 'utf8');
  const start = source.indexOf('async function handleFilterEscape(');
  const handler = ts.transpile(source.slice(start, source.indexOf('function selectTag(', start)), {
    target: ts.ScriptTarget.ES2022,
  });
  const browser = await chromium.launch({ headless: true, channel: process.env.PLAYWRIGHT_CHANNEL });
  try {
    const page = await browser.newPage();
    for (const mode of ['preview', 'ordinary', 'inactive', 'disposed', 'dialog']) {
      await page.goto('about:blank');
      await page.evaluate(({ handler, mode }) => {
        let disposed = false;
        const uiStore = { activePane: 'left-sidebar', inputStack: [] };
        const libConfig = { activePane: 'main', tag: { multi: true } };
        window.exits = 0;
        const handle = new Function('uiStore', 'libConfig', 'config', 'SIDEBAR', 'editor', 'search', 'searchInputRef', 'exitMultiFilter', 'isDisposed',
          `${handler.replace('!disposed', '!isDisposed()')};return handleFilterEscape;`)(
          uiStore, libConfig, { main: { sidebarIndex: 1 } }, { TAG: 1 }, { value: null },
          { value: '' }, { value: null }, () => window.exits++, () => disposed);
        window.addEventListener('keydown', event => { window.pending = handle(event); }, true);
        window.addEventListener('keydown', event => {
          if (mode === 'preview') event.preventDefault();
          if (mode === 'inactive') uiStore.activePane = 'content';
          if (mode === 'disposed') disposed = true;
          if (mode === 'dialog') {
            const dialog = document.createElement('div');
            dialog.setAttribute('role', 'dialog');
            document.body.append(dialog);
          }
        });
      }, { handler, mode });
      // Use a native keyboard event: dispatchEvent hides the microtask ordering bug.
      await page.keyboard.press('Escape');
      const exits = await page.evaluate(async () => { await window.pending; return window.exits; });
      assert.equal(exits, mode === 'ordinary' ? 1 : 0, mode);
    }
  } finally {
    await browser.close();
  }
});
