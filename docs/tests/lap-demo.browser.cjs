// E2E checks against the same Vue components used by the desktop app.
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || "playwright");
const fs = require("node:fs");
const assert = require("node:assert/strict");
(async () => {
  const browser = await chromium.launch({
    headless: true,
    channel: process.env.LAP_BROWSER_CHANNEL || "chrome",
  });
  const page = await browser.newPage({
    viewport: { width: 1600, height: 1050 },
  });
  page.setDefaultTimeout(10000);
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  page.on("console", (m) => {
    if (m.type() === "error") errors.push(m.text());
  });
  const output = process.env.LAP_SCREENSHOT_DIR;
  const shot = async (name) => {
    if (output) {
      fs.mkdirSync(output, { recursive: true });
      await page.screenshot({
        path: output + "/" + name + ".png",
        fullPage: true,
      });
    }
  };
  const comp = (name) => page.locator('[data-demo-component="' + name + '"]');
  const waitCount = async (count) =>
    page.waitForFunction(
      (n) =>
        document.querySelectorAll("[data-thumbnail-container]").length === n,
      count,
    );
  try {
    await page.goto(
      process.env.LAP_DEMO_URL || "http://127.0.0.1:4179/demos/lap/index.html",
    );
    await comp("FileInfo").waitFor();
    await page.waitForFunction(() =>
      [...document.querySelectorAll("[data-thumbnail-container] img")].some(
        (i) => i.complete && i.naturalWidth > 0,
      ),
    );
    assert.equal(
      await page.evaluate(() => window.__lapDemoBackend.state.photos.length),
      20,
    );
    await shot("native-source");
    console.log("PASS: actual Home/Grid/FileInfo and loaded sample thumbnails");
    const rail = page.locator('.demo-workspace .fixed.top-14 [role="button"]');
    const chooseSection = async (index) => {
      const current = await page.evaluate(() => ({
        index: window.__lapDemoUI.config.main.sidebarIndex,
        shown: window.__lapDemoUI.config.leftPanel.show,
      }));
      if (current.index !== index || !current.shown)
        await rail.nth(index).click();
    };
    const components = [
      "Library",
      "AlbumList",
      "SmartAlbumList",
      "ImageSearch",
      "Tag",
      "Calendar",
      "Person",
      "Location",
      "Camera",
    ];
    for (let i = 0; i < 9; i++) {
      await rail.nth(i).click();
      await comp(components[i]).waitFor();
      if (i === 2)
        await comp("SmartAlbumList")
          .getByText("Five-star photos", { exact: true })
          .waitFor();
      if (i === 4)
        await comp("Tag").getByText("Outdoors", { exact: true }).waitFor();
      if (i === 5) await comp("Calendar").getByText(/2025/).first().waitFor();
      if (i === 6)
        await comp("Person").getByText("Sophie", { exact: true }).waitFor();
      if (i === 7)
        await comp("Location")
          .getByText(/Austria/)
          .first()
          .waitFor();
      if (i === 8)
        await comp("Camera")
          .getByText("Canon", { exact: true })
          .first()
          .waitFor();
      await shot("panel-" + i);
    }
    console.log("PASS: all nine native sidebar components and sample content");
    await rail.nth(1).click();
    await page
      .locator("[data-album-list-root]")
      .getByText("Sample-Wikimedia", { exact: true })
      .first()
      .dblclick();
    const root = page
      .locator(
        '[data-folder-tree-root] [data-file-drop-path="/Pictures/Sample-Wikimedia"]',
      )
      .first();
    await root.locator("svg").first().click();
    await comp("AlbumList").getByText("Travel 2024", { exact: true }).waitFor();
    await comp("AlbumList").getByText("Nature 2025", { exact: true }).click();
    await waitCount(10);
    await shot("album-tree");
    await rail.nth(2).click();
    await comp("SmartAlbumList")
      .getByText("Five-star photos", { exact: true })
      .click();
    await waitCount(5);
    await comp("SmartAlbumList")
      .getByText("Favorite photos", { exact: true })
      .click();
    await waitCount(4);
    await comp("SmartAlbumList")
      .getByRole("button", { name: "New smart album", exact: true })
      .click();
    await page.getByText("All rules", { exact: true }).waitFor();
    await shot("smart-editor");
    await page.keyboard.press("Escape");
    console.log("PASS: album folder tree and smart rules/editor");
    await chooseSection(0);
    await comp("Library").getByText("All files", { exact: true }).click();
    await waitCount(20);
    const dragSource = page.locator("[data-thumbnail-container]").first(),
      dragTarget = page.locator('[data-collection-drop-id="2"]');
    const src = await dragSource.boundingBox(),
      dest = await dragTarget.boundingBox();
    await page.mouse.move(src.x + src.width / 2, src.y + src.height / 2);
    await page.mouse.down();
    await page.mouse.move(dest.x + dest.width / 2, dest.y + dest.height / 2, {
      steps: 16,
    });
    await page.mouse.up();
    await page.waitForFunction(() =>
      window.__lapDemoBackend.state.photos.some((p) =>
        p.collectionIds.includes(2),
      ),
    );
    console.log("PASS: original drag/drop adds sample photo to a collection");

    await rail.last().click();
    await page.locator(".demo-window.settings").waitFor();
    const tabs = page.locator(".demo-window.settings .w-40 > div");
    assert.equal(await tabs.count(), 9);
    const expected = [
      "LANGUAGE & REGION",
      "GRID",
      "NAVIGATION",
      "PAIRED FILES",
      "IMPORT METADATA",
      "PHOTO SEARCH",
      "THUMBNAIL CACHE",
      "GLOBAL",
      "Version",
    ];
    for (let i = 0; i < expected.length; i++) {
      await tabs.nth(i).click();
      const content = await page.locator(".demo-window").innerText();
      assert(content.toUpperCase().includes(expected[i].toUpperCase()), `Missing ${expected[i]} in Settings tab ${i}`);
      await shot("settings-" + i);
    }
    await tabs.nth(0).click();
    const mode = page
      .locator(".demo-window.settings")
      .getByText("Color mode", { exact: true })
      .locator("../..")
      .locator("select");
    await mode.selectOption("0");
    await page.waitForFunction(
      () => document.documentElement.dataset.theme === "light",
    );
    await mode.selectOption("1");
    await tabs.nth(1).click();
    const placement = () =>
      page
        .locator(".demo-window.settings")
        .getByText("Filmstrip placement", { exact: true })
        .locator("../..")
        .locator("select");
    await placement().selectOption("1");
    await page
      .getByRole("button", { name: "Close dialog", exact: true })
      .click();
    console.log(
      "PASS: all nine original Settings pages and live appearance change",
    );
    await chooseSection(0);
    await comp("Library").getByText("All files", { exact: true }).click();
    await waitCount(20);
    await page.getByRole("button", { name: "Filmstrip", exact: true }).click();
    const previewReady = () =>
      page.waitForFunction(() =>
        [
          ...document.querySelectorAll(".demo-workspace .slide-wrapper img"),
        ].some((i) => i.complete && i.naturalWidth > 0),
      );
    await previewReady();
    await shot("filmstrip-native");
    for (const position of ["0", "2", "3", "1"]) {
      await rail.last().click();
      await tabs.nth(1).click();
      await placement().selectOption(position);
      await page
        .getByRole("button", { name: "Close dialog", exact: true })
        .click();
      await previewReady();
      assert.equal(
        await page.evaluate(
          () => window.__lapDemoUI.config.settings.grid.previewPosition,
        ),
        Number(position),
      );
    }
    await page
      .getByRole("button", { name: "Justified view", exact: true })
      .click();
    await page.waitForFunction(
      () => window.__lapDemoUI.config.settings.grid.viewMode === "grid",
    );
    console.log("PASS: filmstrip preview and all four placements");
    await page
      .getByRole("button", { name: "Selection", exact: true })
      .click();
    const selection = comp("SelectionPanel");
    await selection.waitFor();
    for (const name of [
      "Select all",
      "Deselect all",
      "Invert selection",
      "Move in library...",
      "Move to folder...",
      "Copy to folder...",
      "Rename...",
      "Move to trash",
      "Compare...",
      "Create collage...",
      "Refresh file info",
    ])
      assert.equal(
        await selection.getByRole("button", { name, exact: true }).count(),
        1,
        name,
      );
    await selection
      .getByRole("button", { name: "Select all", exact: true })
      .click();
    await selection.getByRole("button", { name: /^Favorite/ }).click();
    await page.waitForFunction(() =>
      window.__lapDemoBackend.state.photos.every((p) => p.favorite),
    );
    await selection.getByRole("button", { name: /^3 stars/ }).click();
    await page.waitForFunction(() =>
      window.__lapDemoBackend.state.photos.every((p) => p.rating === 3),
    );
    await shot("selection-native");
    await selection
      .getByRole("button", { name: "Tags...", exact: true })
      .click();
    await page.getByText("Outdoors", { exact: true }).last().waitFor();
    await page.keyboard.press("Escape");
    await selection
      .getByRole("button", { name: "Collections...", exact: true })
      .click();
    await page.getByText("For printing", { exact: true }).last().waitFor();
    await page.keyboard.press("Escape");
    await selection
      .getByRole("button", { name: "Rename...", exact: true })
      .click();
    await page.getByText("Preview", { exact: true }).last().waitFor();
    await page.keyboard.press("Escape");
    await selection
      .getByRole("button", { name: "Close", exact: true })
      .click();
    console.log("PASS: native multi-select actions, batch edits and dialogs");
    await page
      .getByRole("button", { name: "Duplicates", exact: true })
      .click();
    const dedup = comp("DedupPane");
    await dedup.getByText("Duplicate Sets", { exact: true }).waitFor().catch(async (error) => {
      console.error(await dedup.innerText(), errors);
      throw error;
    });
    await page.waitForFunction(() =>
      window.__lapDemoBackend.commands.some(
        (c) => c.command === "dedup_list_groups",
      ),
    );
    await shot("dedup-native");
    await dedup
      .getByRole("button", { name: "Similar Images", exact: true })
      .click();
    await dedup
      .getByRole("button", { name: "Analyze 20 photos", exact: true })
      .click();
    await dedup.getByText("Similar Sets", { exact: true }).waitFor();
    await shot("similar-native");
    await dedup
      .getByRole("button", { name: "Duplicates", exact: true })
      .click();
    const keep = dedup.getByRole("button", { name: "Keep", exact: true }).last();
    await keep.locator("../..").hover();
    await keep.click();
    assert(
      await page.evaluate(() =>
        window.__lapDemoBackend.commands.some(
          (c) => c.command === "dedup_set_keep",
        ),
      ),
    );
    await dedup.locator('input[type="checkbox"]').check();
    await dedup
      .getByRole("button", { name: "Move to trash", exact: true })
      .click();
    await page.waitForFunction(
      () =>
        window.__lapDemoBackend.state.photos.filter((p) => p.deleted).length ===
        1,
    );
    console.log(
      "PASS: native duplicate/similar scan, keep and single-file trash",
    );
    const info = comp("FileInfo");
    if (!(await info.isVisible()))
      await page.getByRole("button", { name: "Info", exact: true }).click();
    await info.waitFor();
    await info.getByRole("button", { name: "Edit metadata", exact: true }).click();
    await page.locator("#metadata-e_title").fill("Edited demo title");
    await page.getByRole("button", { name: "Save", exact: true }).click();
    await info.getByText("Edited demo title", { exact: true }).waitFor();
    await info.getByRole("tab", { name: "Histogram", exact: true }).click();
    await info.locator('svg[viewBox="0 0 256 64"]').waitFor();
    await shot("info-native");
    await info.getByRole("tab", { name: "Thumbnail", exact: true }).click();
    console.log("PASS: actual FileInfo metadata and histogram/thumbnail tabs");
    assert.deepEqual(errors, []);
    if (process.argv.includes("--site")) {
      await page.goto("http://127.0.0.1:4180/lap/", {
        waitUntil: "domcontentloaded",
      });
      const features = page.getByRole("region", { name: "Features", exact: true });
      await features.getByRole("heading", { name: "Local AI search", exact: true }).waitFor();
      assert.equal(await features.locator("article").count(), 8);
      assert.equal(await features.locator("img, svg, iframe").count(), 0);
      assert.equal(await page.locator("iframe").count(), 0);
      await page.setViewportSize({ width: 390, height: 844 });
      assert.equal(await features.locator("article").evaluateAll(cards =>
        new Set(cards.map(card => card.getBoundingClientRect().left)).size,
      ), 1);
      await shot("homepage-mobile");
      await page.setViewportSize({ width: 1280, height: 900 });
      assert.equal(await features.locator("article").evaluateAll(cards =>
        new Set(cards.slice(0, 2).map(card => card.getBoundingClientRect().top)).size,
      ), 1);
      await shot("homepage-features");
      await page.getByRole("link", { name: "Demo", exact: true }).click();
      await page.waitForURL("**/lap/demos/lap/index.html", {
        waitUntil: "domcontentloaded",
      });
      await comp("FileInfo").waitFor();
      console.log("PASS: text-only homepage features and standalone Demo navigation");
    }
    console.log("PASS: no browser exceptions");
  } finally {
    await browser.close();
  }
})().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
