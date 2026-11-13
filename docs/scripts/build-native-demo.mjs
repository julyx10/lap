import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  readFileSync,
  writeFileSync,
  readdirSync,
  unlinkSync,
  existsSync,
} from "node:fs";
import { createHash } from "node:crypto";
import {
  createState,
  addDuplicateFixtures,
} from "../demo-native/fixtures.mjs";
const require = createRequire(
  new URL("../../src-vite/package.json", import.meta.url),
);
const load = async (name) => import(pathToFileURL(require.resolve(name)).href);
const { build } = await load("vite");
const { default: vue } = await load("@vitejs/plugin-vue");
const { default: tailwind } = await load("@tailwindcss/vite");
const { default: svg } = await load("vite-svg-loader");
const root = fileURLToPath(new URL("../demo-native/", import.meta.url));
const app = fileURLToPath(new URL("../../src-vite/src/", import.meta.url));
const out = fileURLToPath(new URL("../public/demos/lap/", import.meta.url));
const seed = addDuplicateFixtures(createState());
seed.photos[0].comment = "A sample memory — click to edit this comment.";
seed.photos.forEach(
  (p) => (p.base64 = readFileSync(`${out}/${p.image}`).toString("base64")),
);
writeFileSync(`${root}/fixtures.generated.json`, JSON.stringify(seed));
const sourceFiles = [
  "views/Home.vue",
  "views/Settings.vue",
  "components/Library.vue",
  "components/Content.vue",
  "components/CollectionTray.vue",
  "components/RefreshFileInfoDialog.vue",
  "components/SearchableSelect.vue",
  "composables/useLibraryChangeRefresh.ts",
  "common/availability.ts",
  "common/fileMenu.ts",
  "common/utils.ts",
  "components/AlbumList.vue",
  "components/AlbumFolder.vue",
  "components/SmartAlbumList.vue",
  "components/SmartAlbumEdit.vue",
  "components/ImageSearch.vue",
  "components/Tag.vue",
  "components/Calendar.vue",
  "components/Person.vue",
  "components/Location.vue",
  "components/Camera.vue",
  "components/SelectionPanel.vue",
  "components/DedupPane.vue",
  "components/FileInfo.vue",
  "components/ImageHistogram.vue",
  "components/MediaViewer.vue",
  "assets/app.css",
];
writeFileSync(
  `${out}/source-manifest.json`,
  JSON.stringify(
    {
      components: sourceFiles.map((path) => ({
        path: `src-vite/src/${path}`,
        sha256: createHash("sha256")
          .update(readFileSync(`${app}/${path}`))
          .digest("hex"),
      })),
    },
    null,
    2,
  ) + "\n",
);
await build({
  configFile: false,
  root,
  base: "./",
  plugins: [
    {
      name: "clean-superseded-demo-bundles",
      writeBundle(_options, bundle) {
        const dir = `${out}/assets`;
        if (existsSync(dir))
          for (const name of readdirSync(dir))
            if (!bundle[`assets/${name}`]) unlinkSync(`${dir}/${name}`);
      },
    },
    {
      name: "lap-browser-adapter",
      enforce: "pre",
      transform(code, id) {
        if (id.endsWith("/components/TButton.vue"))
          return code.replace(
            'role="button"',
            'role="button" :aria-label="tooltip || text" :data-demo-tooltip="tooltip"',
          );
        const component = id.match(
          /\/(Library|AlbumList|SmartAlbumList|ImageSearch|Tag|Calendar|Person|Location|Camera|SelectionPanel|DedupPane|FileInfo)\.vue$/,
        )?.[1];
        if (component)
          return code.replace(
            /(<template>\s*<div)/,
            '$1 data-demo-component="' + component + '"',
          );
        if (id.endsWith("/common/utils.ts"))
          return code.replaceAll(
            'const base = `${scheme}/${_thumbLibraryId}/${fileId}`;',
            'const base = demoMediaUrl(fileId);',
          ).replace(/^/, 'import { demoMediaUrl } from "' + root + '/tauri.mjs";\n');
        if (id.endsWith("/views/Settings.vue"))
          return code.replace(
            "const appWindow = getCurrentWebviewWindow()",
            "const appWindow = getCurrentWebviewWindow('settings')",
          );
      },
    },
    vue({
      template: {
        compilerOptions: {
          isCustomElement: (tag) => tag.startsWith("calendar-"),
        },
      },
    }),
    tailwind(),
    svg(),
  ],
  resolve: {
    alias: [
      { find: /^@tauri-apps\/.*/, replacement: `${root}/tauri.mjs` },
      { find: "@", replacement: app },
      {
        find: "cally",
        replacement: fileURLToPath(
          new URL(
            "../../src-vite/node_modules/cally/dist/cally.js",
            import.meta.url,
          ),
        ),
      },
      ...[
        "vue",
        "pinia",
        "vue-i18n",
        "vue-router",
        "pinia-plugin-persistedstate",
      ].map((name) => ({
        find: new RegExp("^" + name + "$"),
        replacement: require.resolve(name),
      })),
    ],
    dedupe: ["vue", "pinia", "vue-i18n"],
  },
  build: {
    outDir: out,
    emptyOutDir: false,
    sourcemap: false,
    chunkSizeWarningLimit: 1500,
  },
});
