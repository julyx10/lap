import { createApp, h, nextTick, watch } from "vue";
import { createPinia } from "pinia";
import { createRouter, createMemoryHistory } from "vue-router";
import "cally";
import "./theme.css";
import "./shell.css";
import { i18n, initializeLanguage } from "@/common/i18n";
import { useConfigStore } from "@/stores/configStore";
import { useLibraryStore } from "@/stores/libraryStore";
import { useUIStore } from "@/stores/uiStore";
import { useAppTheme } from "@/composables/useAppearance";
import Home from "@/views/Home.vue";
import Settings from "@/views/Settings.vue";
import ImageViewer from "@/views/ImageViewer.vue";
import ImageEditor from "@/views/ImageEditor.vue";
import Montage from "@/views/Montage.vue";
import ToastContainer from "@/components/ToastContainer.vue";
import { overlay, overlayBounds, notice, resetDemo, invoke } from "./tauri.mjs";

const pinia = createPinia();
const app = createApp({
  setup() {
    useAppTheme();
    const components = {
      settings: Settings,
      imageviewer: ImageViewer,
      imageeditor: ImageEditor,
      montage: Montage,
    };
    const config = useConfigStore(),
      lib = useLibraryStore();
    watch(
      () => lib.$state,
      () => {
        if (lib._initialized) lib.save();
      },
      { deep: true },
    );
    return () =>
      h("div", { class: "demo-root" }, [
        h("header", { class: "demo-notice" }, [
          h("strong", "Lap · Interactive demo"),
          h(
            "span",
            "20 sample photos · Changes stay in this browser · File / AI operations are simulated",
          ),
          h("button", { onClick: resetDemo }, "Reset samples"),
        ]),
        h("div", { class: "demo-workspace" }, [
          h(Home, { style: { width: "100%", height: "100%" } }),
        ]),
        overlay.value &&
          h(
            "div",
            {
              class: "demo-overlay",
              role: "dialog",
              "aria-modal": "true",
              "aria-label": overlay.value,
            },
            [
              h(
                "div",
                {
                  class: "demo-window " + overlay.value,
                  style: {
                    width: `min(${overlayBounds.value.width}px,96vw)`,
                    height: `min(${overlayBounds.value.height}px,94vh)`,
                  },
                },
                [
                  h("div", { class: "demo-window-bar" }, [
                    h(
                      "button",
                      {
                        onClick: () => (overlay.value = null),
                        "aria-label": "Close dialog",
                      },
                      "×",
                    ),
                  ]),
                  h(components[overlay.value] || Settings, {
                    key: overlay.value,
                    style: { width: "100%", height: "100%" },
                  }),
                ],
              ),
            ],
          ),
        notice.value &&
          h(
            "div",
            { class: "demo-native-notice", role: "status" },
            notice.value,
          ),
        h(ToastContainer),
      ]);
  },
});
app.use(pinia);
app.use(i18n);
app.use(
  createRouter({
    history: createMemoryHistory(),
    routes: [{ path: "/", component: Home }],
  }),
);
app.config.globalProperties.$invoke = invoke;
const config = useConfigStore();
const saved = localStorage.getItem("lap-native-config-v1");
if (saved) {
  try {
    config.$patch(JSON.parse(saved));
  } catch {}
} else {
  config.leftPanel.show = true;
  config.rightPanel.show = true;
  config.collectionTray.expanded = true;
  config.settings.appearance = 1;
  config.settings.darkThemeId = "dark";
  config.settings.autoCheckUpdates = false;
  config.settings.face.enabled = true;
  config.settings.grid.style = 2;
  config.settings.grid.sizePosition = 0.2;
  config.settings.language = "en";
}
await initializeLanguage(config.settings.language || "en");
await useLibraryStore().init();
const library = useLibraryStore();
if (!library.smartAlbums.length) {
  library.smartAlbums = [
    {
      id: "demo-five-stars",
      name: "Five-star photos",
      description: "Rating is five stars",
      source: "rules",
      query: {
        version: 1,
        match: "all",
        rules: [
          { id: "demo-rating", field: "rating", operator: "is", value: 5 },
        ],
      },
      group: { type: 0 },
      sort: { type: 0, order: 1 },
      coverFileId: 9,
      count: 5,
      createdAt: 1740000000000,
      updatedAt: 1740000000000,
    },
    {
      id: "demo-favorites",
      name: "Favorite photos",
      description: "Favorites across the sample library",
      source: "rules",
      query: {
        version: 1,
        match: "all",
        rules: [
          { id: "demo-fav", field: "favorite", operator: "is", value: true },
        ],
      },
      group: { type: 0 },
      sort: { type: 0, order: 1 },
      coverFileId: 11,
      count: 4,
      createdAt: 1740000000000,
      updatedAt: 1740000000000,
    },
  ];
}
useUIStore().startupReady = true;
config.$subscribe(() => {
  localStorage.setItem("lap-native-config-v1", JSON.stringify(config.$state));
});
window.__lapDemoUI = { config, library: useLibraryStore() };
const applyScale = () =>
  (document.documentElement.style.fontSize = `${16 * Number(config.settings.scale || 1)}px`);
watch(() => config.settings.scale, applyScale, { immediate: true });
watch(overlay, async () => {
  await nextTick();
  applyScale();
});
app.mount("#native-demo");
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape" && overlay.value) {
    overlay.value = null;
  }
});
