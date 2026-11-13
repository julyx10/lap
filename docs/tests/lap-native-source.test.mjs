import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
const root = new URL("../../", import.meta.url);
const manifest = JSON.parse(
  readFileSync(
    new URL("docs/public/demos/lap/source-manifest.json", root),
    "utf8",
  ),
);
test("published demo is built against the current app sources for all requested panels", () => {
  assert.equal(manifest.components.length, 27);
  for (const file of manifest.components) {
    const hash = createHash("sha256")
      .update(readFileSync(new URL(file.path, root)))
      .digest("hex");
    assert.equal(
      file.sha256,
      hash,
      `${file.path} changed; rebuild the native demo`,
    );
  }
});
test("entry uses the real Home and Settings components and actual app stylesheet", () => {
  const main = readFileSync(new URL("docs/demo-native/main.mjs", root), "utf8");
  assert(/from ["']@\/views\/Home\.vue["']/.test(main));
  assert(/from ["']@\/views\/Settings\.vue["']/.test(main));
  const css = readFileSync(new URL("docs/demo-native/theme.css", root), "utf8");
  assert(css.includes("src-vite/src/assets/app.css"));
});
