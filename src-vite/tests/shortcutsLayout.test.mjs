import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import ts from "typescript";

const source = await readFile(
  new URL("../src/common/shortcuts.ts", import.meta.url),
  "utf8",
);
const exports = {};
new Function(
  "exports",
  ts.transpile(source, {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.CommonJS,
  }),
)(exports);
const { matchesShortcut, findShortcutMatches } = exports;

const event = (overrides) => ({
  key: "",
  code: "",
  repeat: false,
  altKey: false,
  ctrlKey: false,
  metaKey: false,
  shiftKey: false,
  ...overrides,
});

const cmd = (key, code, extra = {}) => event({ key, code, metaKey: true, ...extra });

test("QWERTY: Cmd-A selects all", () => {
  assert.equal(matchesShortcut("file.selectAll", cmd("a", "KeyA"), "mac"), true);
});

test("AZERTY: Cmd-Q (physical KeyA slot) no longer triggers Select All", () => {
  // On AZERTY the key labelled "Q" sits at the physical KeyA position, so the
  // browser reports code KeyA with key "q". Matching by code made Cmd-Q fire
  // Select All and swallow the system quit accelerator.
  assert.equal(matchesShortcut("file.selectAll", cmd("q", "KeyA"), "mac"), false);
});

test("AZERTY: Cmd-A (physical KeyQ slot) selects all", () => {
  assert.equal(matchesShortcut("file.selectAll", cmd("a", "KeyQ"), "mac"), true);
});

test("Dvorak: the character decides the action, not the physical slot", () => {
  assert.equal(matchesShortcut("file.selectAll", cmd("a", "KeyS"), "mac"), true);
  assert.equal(matchesShortcut("file.selectAll", cmd("q", "KeyA"), "mac"), false);
});

test("Cyrillic layout: non-ASCII keys fall back to the physical slot", () => {
  // Russian layout: Cmd on the physical KeyC slot produces a Cyrillic char,
  // so copy must keep working through the physical code.
  const cyrillicC = event({ key: "\u0441", code: "KeyC", ctrlKey: true });
  assert.equal(matchesShortcut("file.copy", cyrillicC, "windows"), true);
  // The physical KeyA slot (Cyrillic "\u0444") still selects all, preserving
  // the pre-fix behaviour for non-Latin layouts.
  const cyrillicF = event({ key: "\u0444", code: "KeyA", metaKey: true });
  assert.equal(matchesShortcut("file.selectAll", cyrillicF, "mac"), true);
});

test("shifted letter bindings match by character (Shift+R rotates)", () => {
  assert.equal(
    matchesShortcut("meta.rotateCounterclockwise", event({ key: "R", code: "KeyR", shiftKey: true })),
    true,
  );
  assert.equal(
    matchesShortcut("meta.rotateCounterclockwise", event({ key: "r", code: "KeyR", shiftKey: true })),
    true,
  );
});

test("single-letter actions match by character and still allow shift", () => {
  assert.equal(matchesShortcut("meta.culling.pick", event({ key: "p", code: "KeyP" })), true);
  assert.equal(
    matchesShortcut("meta.culling.pick", event({ key: "P", code: "KeyP", shiftKey: true })),
    true,
  );
  assert.equal(matchesShortcut("meta.culling.pick", event({ key: "x", code: "KeyP" })), false);
});

test("non-letter codes: digits and numpad keep exact physical matching", () => {
  assert.equal(
    matchesShortcut("app.scale.increase", cmd("+", "NumpadAdd")),
    true,
  );
  // Digit bindings stay physical so shifted digit slots keep working on both
  // layouts: QWERTY Shift+1 produces "!" and AZERTY Shift+1 produces "1",
  // and both reach the Digit1 binding through event.code.
  assert.equal(
    matchesShortcut("meta.rating.one", event({ key: "!", code: "Digit1", shiftKey: true })),
    true,
  );
  assert.equal(
    matchesShortcut("meta.rating.one", event({ key: "1", code: "Digit1", shiftKey: true })),
    true,
  );
});

test("punctuation codes follow the layout character too", () => {
  // AZERTY places the comma on the physical Semicolon slot: Cmd-, must open
  // preferences, and the physical Comma slot (producing "m") must not.
  assert.equal(matchesShortcut("app.preferences", cmd(",", "Semicolon"), "mac"), true);
  assert.equal(matchesShortcut("app.preferences", cmd("m", "Comma"), "mac"), false);
  // Slash search: the AZERTY Slash slot produces ":" and must not match.
  assert.equal(matchesShortcut("app.search", cmd("/", "Slash"), "mac"), true);
  assert.equal(matchesShortcut("app.search", cmd(":", "Slash"), "mac"), false);
});

test("context scan: no letter shortcut fires for AZERTY Cmd-Q in content", () => {
  const matches = findShortcutMatches("content", cmd("q", "KeyA"), "mac");
  assert.equal(matches.includes("file.selectAll"), false);
});

test("modifier discipline is unchanged", () => {
  // Cmd-A without the modifier must not select all.
  assert.equal(matchesShortcut("file.selectAll", event({ key: "a", code: "KeyA" }), "mac"), false);
  // Ctrl-A is the non-mac binding.
  assert.equal(
    matchesShortcut("file.selectAll", event({ key: "a", code: "KeyA", ctrlKey: true }), "windows"),
    true,
  );
  // Alt must not be silently accepted.
  assert.equal(
    matchesShortcut("file.selectAll", cmd("a", "KeyA", { altKey: true }), "mac"),
    false,
  );
});
