# Lap interactive demo — actual frontend components

This version compiles the current Lap frontend into a static browser application. It uses the actual components and app stylesheet rather than a handwritten imitation. Only the Tauri/system interfaces are replaced by a deterministic browser fixture adapter.

## Requested areas

| Area | Actual source used |
| --- | --- |
| Left panels | `Library.vue`, `AlbumList.vue`, `AlbumFolder.vue`, `SmartAlbumList.vue`, `SmartAlbumEdit.vue`, `ImageSearch.vue`, `Tag.vue`, `Calendar.vue`, `Person.vue`, `Location.vue`, `Camera.vue`, `CollectionTray.vue` |
| Settings | `views/Settings.vue` and `SettingsAbout.vue`: General, Layout, Viewer, RAW, Search, Advanced, Shortcuts, About, with the original controls, labels and layout |
| Filmstrip | `Content.vue`, `GridView.vue`, `MediaViewer.vue`, `Image.vue`: real large preview, navigation, zoom controls and all four strip placements |
| Multi-select | `SelectionPanel.vue`, `PanelActionButton.vue`, `FavoriteRatingControl.vue`: original action sections, icon/text buttons, enabled states and dialogs |
| Dedup | `DedupPane.vue`: duplicate/similar tabs, summary, sets, split pane, keep selection, culling and delete workflow |
| File info | `FileInfo.vue`, `ImageHistogram.vue`: collapsible preview/general/metadata/map, thumbnail/histogram, full sample EXIF, marks, tags, comments and related dialogs |

The build only adds accessibility labels and `data-demo-component` attributes for browser QA, redirects native thumbnail/preview protocols to local sample assets, and makes the Settings window use the browser window adapter. App templates and business interaction logic otherwise come directly from the repository.

`source-manifest.json` records hashes of key source files so changes to the app can be detected before the next demo release.

## Build and embed

Sources: `docs/demo-native/`. Build script: `docs/scripts/build-native-demo.mjs`.

Install dependencies in both `src-vite` and `docs`, then run:

```sh
pnpm --dir docs build
```

The `prebuild` hook compiles the current app into the demo before VitePress copies public assets. `pnpm --dir docs dev` also regenerates the demo on startup; run `pnpm --dir docs build:demo` again after changing app sources during development. Deployment installs both dependency sets, builds, and runs the Node tests. Generated HTML, bundles and source manifests are ignored by Git. The resulting site is static and needs no backend or API key.

The homepage and **Try Lap** navigation already point to this demo. With the current site base:

```html
<iframe
  src="/lap/demos/lap/index.html"
  title="Lap interactive photo manager demo"
  style="width:100%;height:790px;border:0;border-radius:16px"
  loading="lazy"
></iframe>
```

Keep the complete `assets/` and `thumbnails/` directories with `index.html`. Serve over HTTP rather than opening through `file://`. The interface follows the desktop app; the standalone page gives it the most room.

## Sample behavior and boundaries

- There are 20 fixture records with two exact duplicate pairs, a sample album with two folders, two collections, two custom smart albums, tags, capture dates, camera/lens/location metadata and a predefined face cluster.
- Folder, collection, metadata, selection, favorites, ratings, culling, tags, comments, rename, duplicate cleanup and configuration are simulated in browser memory/localStorage. **Reset samples** restores the fixture data.
- The original settings form is retained, including all options, conditional controls and disabled states. Presentation preferences affect the original frontend directly. Native window changes and desktop-only maintenance operations are simulated.
- File/directory chooser actions use explicit sample paths. They do not read the user's filesystem, upload files, scan disks, launch applications or install models. Native operations display a simulation notice. File Info refresh returns the current fixture data. Saving metadata to XMP shows a desktop-only message.
- Similarity/search results use predefined sample metadata; exact duplicates use seeded matching hashes and thumbnail data. Face clustering is predefined. No AI model runs online.
- RAW previews use a JPEG fixture; decoding, video/Live Photo playback, actual system copying, printing and real library backup/restore are not provided by the browser adapter.
- File info uses fictional metadata. The original histogram component computes from the sample image pixels. The original map uses fictional sample coordinates and the app's selected map provider.
- Thumbnail fixtures originated from the existing website's `screenshots/lap_library.png`; full-size previews therefore display the same low-resolution sample image.

Browser storage keys: `lap-native-data-v1` and `lap-native-config-v1`. No personal photos are included.

## Verification

```sh
pnpm --dir docs test
python3 -m http.server 4179 --bind 127.0.0.1 --directory docs/public
node docs/tests/lap-demo.browser.cjs
```

The browser suite requires Playwright and Chrome. `PLAYWRIGHT_MODULE` can point to an existing Playwright package; `LAP_DEMO_URL` overrides the URL, and `LAP_SCREENSHOT_DIR` saves screenshots. It checks all nine left panels, album/folder and smart-rule results, eight settings categories, live theme changes, filmstrip preview at all four placements, original multi-select buttons and batch mutations/dialogs, duplicate/similar groups and keep actions, and file-info preview tabs. `--site` additionally checks the built homepage and navigation using a server on port 4180 with the documentation output mounted at `/lap/`.

Folder-drop regression: `node docs/tests/lap-album-drop.browser.cjs` uses the same Playwright/server environment and checks the real AlbumList placeholder, cancellation, duplicate/overlap prevention, multi-folder registration, indexing, and source-path confirmation. It verifies fixture IPC calls, not native OS drag payload delivery; test Finder/Explorer/Linux file-manager drags in a desktop build as well.

Desktop folder-drop compatibility: macOS can read native drag-pasteboard paths; Windows/Linux use DOM entries and file URIs. When only a directory entry is available, a folder chooser confirms the source path. If drag metadata is protected until drop, the Albums pane accepts the folder at release time without showing a speculative placeholder. Full hover feedback therefore depends on the host WebView/file manager; no native drop handler was enabled, preserving existing HTML drag/reorder behavior.
