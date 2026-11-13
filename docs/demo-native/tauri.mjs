// Deterministic browser adapter for the unmodified Lap frontend components.
// No native IPC, filesystem access, upload, model execution or system launch.
import { ref } from "vue";
import fixtures from "./fixtures.generated.json";
export const overlay = ref(null),
  overlayBounds = ref({ width: 600, height: 620 }),
  notice = ref("");
let timer;
export function inform(text) {
  notice.value = text;
  clearTimeout(timer);
  timer = setTimeout(() => (notice.value = ""), 4500);
}
const clone = (value) => JSON.parse(JSON.stringify(value));
const KEY = "lap-native-data-v1";
let db = clone(fixtures);
try {
  const saved = JSON.parse(localStorage.getItem(KEY));
  if (saved?.photos?.length) db = saved;
} catch {}
db.tagGroups ||= [
  { id: 1, name: "Subjects", sort_order: 0 },
  { id: 2, name: "My tags", sort_order: 1 },
];
db.tagNames ||= [...new Set(db.photos.flatMap((p) => p.tags))];
db.tagGroupIds ||= Object.fromEntries(db.tagNames.map((name) => [name, 1]));
let libraryState = db.libraryState || null,
  dedupScanned = false,
  similarScanned = false;
let thumbnailCacheLocation = { path: "/Browser/Lap Demo/Cache", custom: false };
const thumbnailCacheMigrations = new Map();
const keeps = new Map(),
  listeners = new Map();
export const commands = [];
const save = () => localStorage.setItem(KEY, JSON.stringify(db));
const live = () => db.photos.filter((p) => !p.deleted);
const photo = (id) => db.photos.find((p) => p.id === Number(id));
const folderPath = (id) => {
  const f = db.folders.find((f) => f.id === id),
    a = db.albums.find((a) => a.id === f?.album);
  return f ? `${a.path}/${f.name}` : a?.path || "/Pictures/Sample-Wikimedia";
};
const nativeFolder = (f) => ({
  id: f.id,
  name: f.name,
  path: folderPath(f.id),
  album_id: f.album,
  is_accessible: true,
  is_favorite: !!f.favorite,
  is_expanded: false,
  has_subfolders: false,
  children: [],
  created_at: 1700000000,
  modified_at: 1740000000,
});
const nativeAlbum = (a) => ({
  ...a,
  cover_file_id: a.cover || 9,
  file_types: 7,
  small_image_filter: 0,
  excluded_folders: [],
  total: live().filter((p) => p.album === a.id).length,
  indexed: live().filter((p) => p.album === a.id).length,
  last_scan_count: 20,
  last_scan_time: 1740000000000,
  created_at: 1700000000,
  modified_at: 1740000000,
  is_accessible: true,
  is_favorite: false,
  description: "Browser sample album",
  has_subfolders: true,
});
function nativeFile(p) {
  if (!p) return null;
  const time = Math.floor(new Date(p.date + "T12:00:00").getTime() / 1000),
    a = db.albums.find((a) => a.id === p.album);
  return {
    id: p.id,
    folder_id: p.folder,
    name: p.name,
    size: Math.round(Number(p.size) * 1024 * 1024),
    file_type: p.type === "RAW" ? 3 : 1,
    format_label: p.type,
    created_at: time,
    modified_at: time,
    taken_date: time,
    width: p.thumbWidth * 20,
    height: p.thumbHeight * 20,
    duration: 0,
    is_favorite: p.favorite,
    rating: p.rating,
    culling_flag: { pick: 1, reject: 2, unreviewed: 0 }[p.culling],
    rotate: p.rotation,
    comments: p.comment,
    has_tags: !!p.tags.length,
    has_faces: p.person ? 1 : 2,
    e_make: p.camera.split(" ")[0],
    e_model: p.camera,
    e_lens_make: p.camera.split(" ")[0],
    e_lens_model: "24-70mm F2.8",
    e_exposure_time: "1/1000",
    e_f_number: "8",
    e_focal_length: "45.58",
    e_iso_speed: "200",
    e_orientation: 1,
    e_software: "Lap browser sample",
    e_artist: "Sample photographer",
    e_description: "Demo metadata",
    e_copyright: "Sample",
    gps_latitude: 48.23 + p.id * 0.002,
    gps_longitude: 15.33 + p.id * 0.003,
    geo_name: p.location,
    geo_admin1: p.location,
    geo_cc: { Austria: "AT", Italy: "IT", Switzerland: "CH" }[p.location],
    file_path: folderPath(p.folder) + "/" + p.name,
    album_id: p.album,
    album_name: a?.name || "Sample album",
    album_accessible: true,
    album_visible: true,
    has_thumbnail: true,
    thumbnail: "data:image/jpeg;base64," + p.base64,
    has_embedding: true,
    has_collections: !!p.collectionIds.length,
    last_scan_time: Date.now() - 86400000,
    cloud_only: false,
    metadata_pending: false,
    media_subtype: null,
  };
}
const tagRecords = () =>
  db.tagNames
    .map((name, i) => ({
      id: i + 1,
      name,
      group_id: db.tagGroupIds[name] || 1,
      group_name:
        db.tagGroups.find((g) => g.id === (db.tagGroupIds[name] || 1))?.name ||
        "Subjects",
      count: live().filter((p) => p.tags.includes(name)).length,
      sort_order: i,
    }))
    .filter((t) => t.name);
function query(params = {}, collectionId) {
  const p = Object.fromEntries(
    Object.entries(params || {}).map(([key, value]) => [
      key.replace(/[A-Z]/g, (c) => "_" + c.toLowerCase()),
      value,
    ]),
  );
  let list = live();
  if (collectionId)
    list = list.filter((f) => f.collectionIds.includes(Number(collectionId)));
  if (p.search_file_name)
    list = list.filter((f) =>
      f.name.toLowerCase().includes(p.search_file_name.toLowerCase()),
    );
  if (p.search_folder)
    list = list.filter((f) => folderPath(f.folder) === p.search_folder);
  if (p.search_all_subfolders)
    list = list.filter((f) =>
      folderPath(f.folder).startsWith(p.search_all_subfolders),
    );
  if (p.is_favorite) list = list.filter((f) => f.favorite);
  if (Number(p.rating) >= 0)
    list = list.filter((f) => f.rating === Number(p.rating));
  if (Number(p.rating) === -2) list = list.filter((f) => f.rating > 0);
  if (Number(p.culling_flag) >= 0)
    list = list.filter(
      (f) =>
        ({ pick: 1, reject: 2, unreviewed: 0 })[f.culling] ===
        Number(p.culling_flag),
    );
  if (p.tag_id > 0)
    list = list.filter((f) => f.tags.includes(db.tagNames[p.tag_id - 1]));
  if (p.tag_group_id > 0)
    list = list.filter((f) =>
      f.tags.some((t) => (db.tagGroupIds[t] || 1) === p.tag_group_id),
    );
  if (p.person_id > 0) list = list.filter((f) => f.person);
  if (p.make) list = list.filter((f) => f.camera.startsWith(p.make));
  if (p.model) list = list.filter((f) => f.camera === p.model);
  if (p.location_name || p.location_admin1)
    list = list.filter(
      (f) => f.location === (p.location_name || p.location_admin1),
    );
  if (p.start_date > 0)
    list = list.filter(
      (f) => new Date(f.date).getTime() / 1000 >= p.start_date,
    );
  if (p.end_date > 0)
    list = list.filter((f) => new Date(f.date).getTime() / 1000 <= p.end_date);
  if (p.search_file_type > 0)
    list = list.filter(
      (f) => ((f.type === "RAW" ? 4 : 1) & p.search_file_type) !== 0,
    );
  if (p.rules?.length)
    list = list.filter((f) => {
      const results = p.rules.map((r) => {
        const flag = { pick: 1, reject: 2, unreviewed: 0 }[f.culling],
          nf = nativeFile(f);
        const values = {
          rating: f.rating,
          favorite: f.favorite,
          is_favorite: f.favorite,
          culling: flag,
          culling_flag: flag,
          file_type: f.type === "RAW" ? 3 : 1,
          name: f.name,
          folder_name: folderPath(f.folder),
          extension: f.name.split(".").at(-1),
          tag: f.tags.map((t) => db.tagNames.indexOf(t) + 1),
          album: f.album,
          album_id: f.album,
          collection: f.collectionIds,
          person: f.person ? 1 : 0,
          has_gps: true,
          size: nf.size,
          width: nf.width,
          height: nf.height,
          duration: 0,
          date_taken: nf.taken_date,
          date_created: nf.created_at,
          date_modified: nf.modified_at,
          camera: nf.e_make + "||" + nf.e_model,
          lens: nf.e_lens_make + "||" + nf.e_lens_model,
          location: nf.geo_cc + "||" + nf.geo_admin1 + "||" + nf.geo_name,
        };
        const val = values[r.field];
        const value = r.value;
        if (r.operator === "contains")
          return String(val)
            .toLowerCase()
            .includes(String(value).toLowerCase());
        if (r.operator === "not_contains")
          return !String(val)
            .toLowerCase()
            .includes(String(value).toLowerCase());
        if (["gte", "greater_than_or_equal"].includes(r.operator))
          return Number(val) >= Number(value);
        if (r.operator === "gt" || r.operator === "after")
          return Number(val) > Number(value);
        if (r.operator === "lt" || r.operator === "before")
          return Number(val) < Number(value);
        if (r.operator === "lte") return Number(val) <= Number(value);
        if (r.operator === "between")
          return (
            Number(val) >= Number(value?.[0] ?? value?.min) &&
            Number(val) <= Number(value?.[1] ?? value?.max)
          );
        if (r.operator === "empty")
          return !val || (Array.isArray(val) && !val.length);
        if (r.operator === "not_empty")
          return !!val && (!Array.isArray(val) || val.length > 0);
        const equal = Array.isArray(val)
          ? val.map(String).includes(String(value))
          : typeof val === "boolean"
            ? val ===
              (value === true ||
                value === 1 ||
                value === "true" ||
                value === "1")
            : String(val) === String(value);
        return ["neq", "not_equals", "is_not", "not_has"].includes(r.operator)
          ? !equal
          : equal;
      });
      return p.match === "any" ? results.some(Boolean) : results.every(Boolean);
    });
  list = list.map(nativeFile);
  list.sort((a, b) =>
    p.sort_type === 3
      ? a.name.localeCompare(b.name)
      : p.sort_type === 4
        ? a.size - b.size
        : a[
            p.sort_type === 1
              ? "created_at"
              : p.sort_type === 2
                ? "modified_at"
                : "taken_date"
          ] -
          b[
            p.sort_type === 1
              ? "created_at"
              : p.sort_type === 2
                ? "modified_at"
                : "taken_date"
          ],
  );
  if (p.sort_order === 1) list.reverse();
  return list;
}
function grouped(list, params = {}) {
  const by = Number(params.groupBy ?? params.group_by ?? 0),
    map = new Map();
  for (const f of list) {
    const date = new Date(f.taken_date * 1000).toISOString().slice(0, 10);
    const key =
      by === 1
        ? folderPath(f.folder_id)
        : by === 2
          ? date
          : by === 3
            ? date.slice(0, 7)
            : by === 4
              ? String(f.rating)
              : by === 5
                ? f.geo_name
                : by === 6
                  ? f.e_model
                  : by === 7
                    ? f.e_lens_model
                    : by === 8
                      ? date.slice(0, 4)
                      : by === 9
                        ? f.format_label
                        : by === 10
                          ? String(f.culling_flag)
                          : "All files";
    if (!map.has(key)) map.set(key, []);
    map.get(key).push(f);
  }
  const rows = [],
    groups = [];
  let fileIndex = 0;
  for (const [key, files] of map) {
    const size = files.reduce((s, f) => s + f.size, 0);
    groups.push({
      groupId: key,
      label: key,
      count: files.length,
      size,
      rowIndex: rows.length,
    });
    rows.push({
      type: "group",
      rowId: "group-" + key,
      groupId: key,
      label: key,
      count: files.length,
      size,
    });
    for (const file of files)
      rows.push({
        type: "item",
        rowId: "file-" + file.id,
        groupId: key,
        fileIndex: fileIndex++,
        file,
      });
  }
  return {
    rows,
    groups,
    totalItemCount: list.length,
    totalRowCount: rows.length,
    totalSize: list.reduce((s, f) => s + f.size, 0),
  };
}
function groups(similar = false) {
  const map = new Map();
  for (const p of live()) {
    const key = similar
      ? p.tags.includes("Nature")
        ? "Nature"
        : p.tags.includes("Landscape")
          ? "Landscape"
          : "Portraits"
      : p.hash || p.id;
    if (!map.has(key)) map.set(key, []);
    map.get(key).push(p);
  }
  return [...map.values()]
    .filter((g) => g.length > 1)
    .map((g) => {
      const groupId = similar
        ? g[0].tags.includes("Nature")
          ? 1
          : g[0].tags.includes("Landscape")
            ? 2
            : 3
        : Number(g[0].hash.split("-").at(-1));
      return {
        id: groupId,
        hash: g[0].hash || "sample",
        file_size: nativeFile(g[0]).size,
        file_count: g.length,
        total_size: g.reduce((s, p) => s + nativeFile(p).size, 0),
        reviewed: keeps.has(groupId) ? 1 : 0,
        updated_at: 1740000000,
        representative: nativeFile(g[0]),
        files: g.map(nativeFile),
        items: g.map((p, j) => ({
          group_id: groupId,
          file_id: p.id,
          is_keep: keeps.get(groupId)
            ? Number(keeps.get(groupId) === p.id)
            : Number(j === 0),
          is_selected: 0,
          score: 1 - j * 0.01,
          file: nativeFile(p),
        })),
      };
    });
}
export async function listen(name, callback) {
  if (!listeners.has(name)) listeners.set(name, new Set());
  listeners.get(name).add(callback);
  return () => listeners.get(name)?.delete(callback);
}
export async function emit(name, payload) {
  for (const fn of listeners.get(name) || [])
    await fn({ event: name, payload, id: 0 });
}
export const emitTo = async (_target, name, payload) => emit(name, payload);
export const once = async (name, fn) => {
  const off = await listen(name, (e) => {
    off();
    fn(e);
  });
  return off;
};
const windows = new Map();
function makeWindow(label = "main") {
  const noop = async () => {};
  return {
    label,
    show: async () => {
      if (label !== "main") overlay.value = label;
    },
    hide: async () => (overlay.value = null),
    close: async () => {
      overlay.value = null;
      windows.delete(label);
    },
    destroy: async () => {
      overlay.value = null;
      windows.delete(label);
    },
    isMinimized: async () => false,
    isMaximized: async () => false,
    isVisible: async () => true,
    isFullscreen: async () => false,
    scaleFactor: async () => 1,
    innerSize: async () => ({ width: innerWidth, height: innerHeight }),
    outerSize: async () => ({ width: innerWidth, height: innerHeight }),
    outerPosition: async () => ({ x: 0, y: 0 }),
    setSize: async (size) => {
      if (label !== "main")
        overlayBounds.value = { width: size.width, height: size.height };
    },
    setMinSize: noop,
    setMaxSize: noop,
    setPosition: noop,
    setFocus: noop,
    setTitle: noop,
    setAlwaysOnTop: noop,
    setResizable: noop,
    setDecorations: noop,
    setBackgroundColor: noop,
    setFullscreen: noop,
    unminimize: noop,
    minimize: noop,
    maximize: noop,
    unmaximize: noop,
    toggleMaximize: noop,
    center: noop,
    startDragging: noop,
    onCloseRequested: async () => noop,
    onResized: async () => noop,
    onMoved: async () => noop,
    onFocusChanged: async () => noop,
    onDragDropEvent: async () => noop,
    listen,
    emit,
    emitTo,
    once: async (name, cb) => {
      if (name === "tauri://created") queueMicrotask(() => cb({}));
      return noop;
    },
  };
}
export class WebviewWindow {
  constructor(label, options) {
    const win = makeWindow(label);
    windows.set(label, win);
    if (options?.width && options?.height)
      overlayBounds.value = { width: options.width, height: options.height };
    overlay.value = label;
    return win;
  }
  static async getByLabel(label) {
    return windows.get(label) || null;
  }
  static async getAll() {
    return [...windows.values()];
  }
}
const mainWindow = makeWindow();
export const getCurrentWebviewWindow = (label) =>
  label ? makeWindow(label) : mainWindow;
export const getCurrentWindow = () => mainWindow;
export const getCurrentWebview = () => ({
  setZoom: async () => {},
  ...mainWindow,
});
export const getAllWebviewWindows = async () => [
  mainWindow,
  ...windows.values(),
];
export class LogicalSize {
  constructor(width, height) {
    this.width = width;
    this.height = height;
  }
}
export class PhysicalSize extends LogicalSize {}
export class LogicalPosition {
  constructor(x, y) {
    this.x = x;
    this.y = y;
  }
}
export class PhysicalPosition extends LogicalPosition {}
export class Channel {
  onmessage = () => {};
}
export const getName = async () => "Lap";
export const getVersion = async () => "0.3.3";
export const locale = async () => "en-US";
export const platform = () => "macos";
export const type = () => "macos";
export const version = async () => "15.0";
export const check = async () => null;
export const relaunch = async () =>
  inform("Restart is simulated in this browser demo.");
export const exit = relaunch;
export const open = async (options) => {
  const path = options?.directory
    ? "/Pictures/Browser Album"
    : "/Applications/Preview.app";
  inform(`Sample chooser: ${path}. No real file or folder is accessed.`);
  return path;
};
export const saveDialog = async () => null;
export { saveDialog as save };
export const message = async (text) => inform(String(text));
export async function ask(text, options = {}) {
  return new Promise((resolve) => {
    const dialog = document.createElement("dialog");
    dialog.className = "demo-confirm";
    const body = document.createElement("p");
    body.textContent = String(text);
    dialog.append(body);
    const note = document.createElement("small");
    note.textContent = "Only sample data in this browser is affected.";
    dialog.append(note);
    const footer = document.createElement("footer");
    for (const [label, value] of [
      [options.cancelLabel || "Cancel", false],
      [options.okLabel || "Confirm", true],
    ]) {
      const b = document.createElement("button");
      b.textContent = label;
      b.onclick = () => {
        dialog.close();
        dialog.remove();
        resolve(value);
      };
      footer.append(b);
    }
    dialog.append(footer);
    dialog.oncancel = () => {
      dialog.remove();
      resolve(false);
    };
    document.body.append(dialog);
    dialog.showModal();
  });
}
export const confirm = ask;
export function demoMediaUrl(fileId) {
  const sample = photo(fileId) || fixtures.photos.find((p) => p.id === Number(fileId));
  return sample ? new URL("./" + sample.image, location.href).href : "";
}
export function convertFileSrc(path) {
  if (String(path).startsWith("data:") || String(path).startsWith("http"))
    return path;
  const p = db.photos.find((p) => String(path).endsWith("/" + p.name));
  return new URL("./" + (p?.image || "thumbnails/01.jpg"), location.href).href;
}
export function resetDemo() {
  localStorage.removeItem(KEY);
  localStorage.removeItem("lap-native-config-v1");
  location.reload();
}
function changed() {
  save();
  queueMicrotask(() => emit("file-metadata-updated"));
}
export async function invoke(command, args = {}) {
  commands.push({ command, args });
  if (commands.length > 500) commands.shift();
  const p = photo(args.fileId ?? args.id),
    params = args.params || {};
  const collection = args.collectionId;
  const list = () => query(params, collection);
  if (command === "get_app_config")
    return {
      current_library_id: 1,
      libraries: [
        {
          id: 1,
          name: "Default Library",
          is_hidden: false,
          path: "/Pictures/Lap Demo",
        },
      ],
    };
  if (
    command === "get_current_library_state" ||
    command === "get_library_state"
  )
    return libraryState;
  if (command === "save_library_state") {
    libraryState = clone(args.state);
    db.libraryState = libraryState;
    save();
    return true;
  }
  if (command === "get_system_region_locale") return "en-US";
  if (
    command === "is_database_corrupted" ||
    command === "is_using_custom_db_storage" ||
    command === "is_indexing"
  )
    return false;
  if (command === "get_all_albums") return db.albums.map(nativeAlbum);
  if (command === "get_album")
    return nativeAlbum(
      db.albums.find((a) => a.id === args.albumId) || db.albums[0],
    );
  if (command === "get_all_album_folders")
    return [
      ...db.albums.map((a) => ({
        id: 1000 + a.id,
        album_id: a.id,
        name: a.name,
        path: a.path,
        has_subfolders: true,
        is_accessible: true,
      })),
      ...db.folders.map(nativeFolder),
    ];
  if (command === "fetch_folder") {
    const f = db.folders.find((f) => folderPath(f.id) === args.path),
      a = db.albums.find((a) => a.path === args.path);
    return {
      ...(f
        ? nativeFolder(f)
        : { id: a?.id || 1, name: a?.name || "Sample", path: args.path }),
      children: f
        ? []
        : db.folders.filter((f) => !a || f.album === a.id).map(nativeFolder),
      is_accessible: true,
      has_subfolders: !f,
    };
  }
  if (command === "select_folder") {
    const f = db.folders.find((f) => folderPath(f.id) === args.folderPath);
    if (f) return nativeFolder(f);
    const a = db.albums.find((a) => a.path === args.folderPath) || db.albums[0];
    return {
      id: 1000 + a.id,
      album_id: a.id,
      name: a.name,
      path: a.path,
      has_subfolders: true,
      is_accessible: true,
    };
  }
  if (
    command === "get_folder_favorite" ||
    command === "get_folder_search_excluded"
  )
    return false;
  if (
    command === "is_directory_accessible" ||
    command === "check_album_accessibility" ||
    command === "check_file_exists" ||
    command === "check_file_accessibility"
  )
    return true;
  if (command === "get_library_visible_counts") {
    const a = live(),
      count = (fn) => a.filter(fn).length;
    return {
      all: a.length,
      favorite: count((p) => p.favorite),
      today: count(
        (p) =>
          p.date.slice(5) ===
          `${String(new Date().getMonth() + 1).padStart(2, "0")}-${String(new Date().getDate()).padStart(2, "0")}`,
      ),
      rated: count((p) => p.rating > 0),
      unrated: count((p) => !p.rating),
      ...Object.fromEntries(
        [1, 2, 3, 4, 5].map((n) => [
          "rating" + n,
          count((p) => p.rating === n),
        ]),
      ),
      pick: count((p) => p.culling === "pick"),
      reject: count((p) => p.culling === "reject"),
      unreviewed: count((p) => p.culling === "unreviewed"),
    };
  }
  if (command === "get_album_visible_counts")
    return Object.fromEntries(
      db.albums.map((a) => [
        a.id,
        live().filter((p) => p.album === a.id).length,
      ]),
    );
  if (command === "get_folder_thumb_count")
    return live().filter((p) =>
      folderPath(p.folder).startsWith(args.folderPath || args.path || ""),
    ).length;
  if (command === "count_folder") {
    const rows = live().filter((p) =>
      folderPath(p.folder).startsWith(args.path || ""),
    );
    return [
      rows.length,
      rows.reduce((sum, p) => sum + nativeFile(p).size, 0),
      0,
      0,
      0,
      0,
      0,
    ];
  }
  if (command === "list_album_subfolders")
    return db.folders
      .map((f) => folderPath(f.id))
      .filter((path) => path.startsWith(args.path + "/"));
  if (command === "recount_album")
    return nativeAlbum(
      db.albums.find((a) => a.id === args.albumId) || db.albums[0],
    );
  if (
    command.includes("query_count_and_sum") ||
    command === "get_collection_count_and_sum"
  ) {
    const a = list();
    return [a.length, a.reduce((s, p) => s + p.size, 0)];
  }
  if (
    command === "get_query_files" ||
    command === "get_smart_query_files" ||
    command === "get_collection_files"
  )
    return list().slice(
      args.offset || 0,
      (args.offset || 0) + (args.limit || 100),
    );
  if (command.includes("grouped_query_rows")) {
    const result = grouped(list(), params);
    return {
      ...result,
      rows: result.rows.slice(
        args.offset || 0,
        (args.offset || 0) + (args.limit || 100),
      ),
    };
  }
  if (command.includes("query_time_line")) return [];
  if (command.includes("group_file_ids"))
    return grouped(list(), params)
      .rows.filter((r) => r.type === "item" && r.groupId === args.groupId)
      .map((r) => r.file.id);
  if (command.includes("query_file_ids")) return list().map((p) => p.id);
  if (command.includes("file_position"))
    return list().findIndex((p) => p.id === args.fileId);
  if (command === "get_files_by_ids")
    return args.fileIds.map(photo).filter(Boolean).map(nativeFile);
  if (
    command === "get_file_info" ||
    command === "update_file_info"
  )
    return nativeFile(p);
  if (command === "is_cloud_only_file") return false;
  if (command === "begin_file_refresh_batch") return 1;
  if (command === "end_file_refresh_batch") return null;
  if (command === "refresh_selected_file_info") {
    return p ? { file: nativeFile(p), size_delta: 0, skipped: false } : null;
  }
  if (command === "save_xmp_metadata") {
    const failures = (args.fileIds || []).map(fileId => ({
      fileId,
      error: "XMP writeback is available in the desktop app.",
    }));
    return { savedIds: [], failures };
  }
  if (command === "get_file_thumb" || command === "get_file_thumb_by_id")
    return {
      file_id: Number(args.fileId),
      error_code: 0,
      thumb_data_base64:
        p?.base64 ||
        fixtures.photos.find((p) => p.id === Number(args.fileId))?.base64 ||
        fixtures.photos[0].base64,
    };
  if (command === "get_file_thumbs")
    return (args.files || []).map((f) => ({
      file_id: f.fileId ?? f.id,
      error_code: 0,
      thumb_data_base64: photo(f.fileId ?? f.id)?.base64,
    }));
  if (command === "get_file_histogram")
    return {
      r: new Array(256).fill(1),
      g: new Array(256).fill(1),
      b: new Array(256).fill(1),
    };
  if (command === "get_taken_dates") {
    const counts = new Map();
    for (const p of live()) counts.set(p.date, (counts.get(p.date) || 0) + 1);
    return [...counts].sort((a, b) => a[0].localeCompare(b[0]));
  }
  if (command === "get_tag_counts")
    return Object.fromEntries(tagRecords().map((t) => [t.id, t.count]));
  if (command === "set_file_favorite") {
    if (p) p.favorite = args.isFavorite;
    changed();
    return true;
  }
  if (command === "set_file_rating") {
    if (p) p.rating = args.rating;
    changed();
    return true;
  }
  if (command === "set_file_culling_flag") {
    if (p) p.culling = ["unreviewed", "pick", "reject"][args.cullingFlag];
    changed();
    return true;
  }
  if (command === "set_file_rotate") {
    if (p) p.rotation = args.fileRotate ?? args.rotate ?? 0;
    changed();
    return true;
  }
  if (command === "edit_file_comment") {
    if (p) p.comment = args.comment;
    changed();
    return nativeFile(p);
  }
  if (command === "rename_file") {
    if (p) p.name = args.newName || args.fileName || p.name;
    changed();
    return nativeFile(p);
  }
  if (command === "preview_batch_rename" || command === "batch_rename_files") {
    const items = (args.fileIds || []).map((id, i) => {
      const p = photo(id),
        o = args.options || {},
        suffix = p.name.includes(".") ? "." + p.name.split(".").at(-1) : "";
      return {
        id,
        oldName: p.name,
        newName: `${o.name || "Sample"}_${String((o.start || 1) + i).padStart(o.digits || 0, "0")}${suffix}`,
        companions: 0,
      };
    });
    if (command === "batch_rename_files") {
      items.forEach((item) => (photo(item.id).name = item.newName));
      changed();
    }
    return { items, errors: [] };
  }
  if (command === "move_file") {
    if (p) {
      const f = db.folders.find(
        (f) =>
          f.id === args.newFolderId || folderPath(f.id) === args.newFolderPath,
      );
      if (f) {
        p.folder = f.id;
        p.album = f.album;
      }
    }
    changed();
    return nativeFile(p);
  }
  if (command === "batch_delete_files" || command === "dedup_delete") {
    const targets =
      command === "dedup_delete"
        ? args.request?.fileIds ||
          groups().flatMap((g) =>
            g.items.filter((i) => !i.is_keep).map((i) => i.file_id),
          )
        : (args.files || []).map((f) => f.id ?? f.fileId);
    targets.forEach((id) => {
      if (photo(id)) photo(id).deleted = true;
    });
    changed();
    return {
      deletedFileIds: targets,
      deletedCount: targets.length,
      failedCount: 0,
      errors: [],
      trashFailedFileIds: [],
    };
  }
  if (command === "delete_file" || command === "delete_db_file") {
    if (p) p.deleted = true;
    changed();
    return true;
  }
  if (command === "add_album") {
    const id = Math.max(...db.albums.map((a) => a.id)) + 1;
    const a = { id, name: args.name || "Browser Album", path: args.folderPath };
    db.albums.push(a);
    db.folders.push({
      id: Math.max(...db.folders.map((f) => f.id)) + 1,
      album: id,
      name: "New folder",
    });
    save();
    return nativeAlbum(a);
  }
  if (command === "edit_album") {
    const a = db.albums.find((a) => a.id === args.id);
    Object.assign(a, { name: args.name, description: args.description });
    save();
    return nativeAlbum(a);
  }
  if (command === "create_folder") {
    const a =
        db.albums.find((a) => args.path.startsWith(a.path)) || db.albums[0],
      f = {
        id: Math.max(...db.folders.map((f) => f.id)) + 1,
        album: a.id,
        name: args.folderName,
      };
    db.folders.push(f);
    save();
    return nativeFolder(f);
  }
  if (command === "rename_folder") {
    const f = db.folders.find((f) => folderPath(f.id) === args.folderPath);
    if (f) f.name = args.newFolderName;
    save();
    return f ? nativeFolder(f) : null;
  }
  if (command === "list_collections")
    return db.collections.map((c, i) => ({
      ...c,
      sortOrder: i,
      count: live().filter((p) => p.collectionIds.includes(c.id)).length,
      createdAt: 1700000000,
      updatedAt: 1740000000,
    }));
  if (command === "get_collection_counts")
    return Object.fromEntries(
      db.collections.map((c) => [
        c.id,
        live().filter((p) => p.collectionIds.includes(c.id)).length,
      ]),
    );
  if (command === "get_collection_selection_counts")
    return db.collections.map((c) => ({
      collection_id: c.id,
      count: (args.fileIds || []).filter((id) =>
        photo(id)?.collectionIds.includes(c.id),
      ).length,
    }));
  if (command === "create_collection") {
    const c = {
      id: Math.max(0, ...db.collections.map((c) => c.id)) + 1,
      name: args.name,
    };
    db.collections.push(c);
    save();
    return c;
  }
  if (command === "rename_collection") {
    const c = db.collections.find((c) => c.id === args.id);
    c.name = args.name;
    save();
    return c;
  }
  if (command === "delete_collection") {
    db.collections = db.collections.filter((c) => c.id !== args.id);
    save();
    return true;
  }
  if (
    command === "add_files_to_collection" ||
    command === "remove_files_from_collection"
  ) {
    const skipped = (args.fileIds || []).filter((id) =>
      photo(id)?.collectionIds.includes(args.collectionId),
    ).length;
    (args.fileIds || []).forEach((id) => {
      const p = photo(id);
      if (p)
        p.collectionIds = command.startsWith("add")
          ? [...new Set([...p.collectionIds, args.collectionId])]
          : p.collectionIds.filter((id) => id !== args.collectionId);
    });
    changed();
    return command === "add_files_to_collection"
      ? { added: (args.fileIds || []).length - skipped, skipped }
      : (args.fileIds || []).length;
  }
  if (command === "get_file_collections")
    return db.collections.filter((c) => p?.collectionIds.includes(c.id));
  if (command === "get_collection_file_ids")
    return live()
      .filter((p) => p.collectionIds.includes(args.collectionId))
      .map((p) => p.id);
  if (command === "get_tags" || command === "get_all_tags") return tagRecords();
  if (command === "get_tag_groups") return db.tagGroups;
  if (command === "get_file_tags" || command === "get_tags_for_file")
    return tagRecords().filter((t) => p?.tags.includes(t.name));
  if (command === "get_tag_selection_counts")
    return tagRecords().map((t) => ({
      tag_id: t.id,
      count: (args.fileIds || []).filter((id) =>
        photo(id)?.tags.includes(t.name),
      ).length,
    }));
  if (command === "apply_tags_to_files") {
    for (const id of args.fileIds || []) {
      const p = photo(id);
      if (!p) continue;
      const remove = (args.removeTagIds || []).map((id) => db.tagNames[id - 1]);
      p.tags = p.tags.filter((t) => !remove.includes(t));
      for (const tagId of args.addTagIds || []) {
        const name = db.tagNames[tagId - 1];
        if (name && !p.tags.includes(name)) p.tags.push(name);
      }
    }
    changed();
    return (args.fileIds || []).length;
  }
  if (command === "batch_update_file_metadata") {
    const change = args.params || {},
      targetIds = change.fileIds || change.file_ids || [];
    for (const id of targetIds) {
      const p = photo(id);
      if (!p) continue;
      const favorite = change.isFavorite ?? change.is_favorite,
        culling = change.cullingFlag ?? change.culling_flag,
        rotation = change.rotateDelta ?? change.rotate_delta;
      if (favorite !== undefined && favorite !== null) p.favorite = favorite;
      if (change.rating !== undefined && change.rating !== null)
        p.rating = change.rating;
      if (culling !== undefined && culling !== null)
        p.culling = ["unreviewed", "pick", "reject"][culling];
      if (rotation !== undefined && rotation !== null)
        p.rotation = (p.rotation + rotation) % 360;
      if (change.comment !== undefined && change.comment !== null)
        p.comment = change.comment;
    }
    changed();
    return targetIds.length;
  }
  if (command === "add_tag_to_file" || command === "remove_tag_from_file") {
    const name = db.tagNames[args.tagId - 1];
    if (p)
      p.tags = command.startsWith("add")
        ? [...new Set([...p.tags, name])]
        : p.tags.filter((t) => t !== name);
    changed();
    return true;
  }
  if (command === "rename_tag") {
    const old = db.tagNames[args.tagId - 1];
    db.tagNames[args.tagId - 1] = args.newName;
    db.photos.forEach(
      (p) => (p.tags = p.tags.map((t) => (t === old ? args.newName : t))),
    );
    changed();
    return tagRecords()[args.tagId - 1];
  }
  if (command === "get_tag_group_name")
    return db.tagGroups.find((g) => g.id === args.id)?.name || "";
  if (command === "save_tag_group") {
    let g = db.tagGroups.find((g) => g.id === args.id);
    if (g) g.name = args.name;
    else {
      g = {
        id: Math.max(0, ...db.tagGroups.map((g) => g.id)) + 1,
        name: args.name,
        sort_order: db.tagGroups.length,
      };
      db.tagGroups.push(g);
    }
    save();
    return g.id;
  }
  if (command === "move_tags_to_group") {
    for (const id of args.tagIds || [])
      db.tagGroupIds[db.tagNames[id - 1]] = args.groupId;
    save();
    return true;
  }
  if (command === "delete_tag_group") {
    db.tagGroups = db.tagGroups.filter((g) => g.id !== args.id);
    Object.keys(db.tagGroupIds).forEach((name) => {
      if (db.tagGroupIds[name] === args.id) db.tagGroupIds[name] = 1;
    });
    save();
    return true;
  }
  if (command === "delete_tag") {
    const name = db.tagNames[args.tagId - 1];
    db.tagNames[args.tagId - 1] = null;
    db.photos.forEach((p) => (p.tags = p.tags.filter((t) => t !== name)));
    changed();
    return true;
  }
  if (command === "get_tag_name")
    return tagRecords().find((t) => t.id === args.tagId)?.name || "";
  if (command === "add_tag" || command === "create_tag") {
    db.tagNames.push(args.name);
    db.tagGroupIds[args.name] = args.groupId || 1;
    save();
    return tagRecords().at(-1);
  }
  if (command === "get_camera_info" || command === "get_lens_info")
    return ["Canon", "Sony"].map((make) => ({
      make,
      models: command.includes("lens")
        ? ["24-70mm F2.8"]
        : [
            ...new Set(
              live()
                .filter((p) => p.camera.startsWith(make))
                .map((p) => p.camera),
            ),
          ],
      counts: [live().filter((p) => p.camera.startsWith(make)).length],
    }));
  if (command === "get_location_info")
    return ["Austria", "Italy", "Switzerland"].map((name) => ({
      cc: { Austria: "AT", Italy: "IT", Switzerland: "CH" }[name],
      admin1: name,
      names: [name],
      counts: [live().filter((p) => p.location === name).length],
    }));
  if (command === "get_persons" || command === "get_persons_page") {
    const persons = [
      {
        id: 1,
        name: "Sophie",
        face_count: 2,
        count: 2,
        thumbnail: photo(2).base64,
      },
    ];
    return command === "get_persons"
      ? persons
      : { persons, total: 1, has_more: false, selected_person_visible: true };
  }
  if (command === "get_face_stats")
    return {
      total_files: 20,
      indexed_files: 20,
      files_with_faces: 2,
      total_faces: 2,
      total_persons: 1,
      face_count: 2,
      person_count: 1,
    };
  if (command === "get_faces_for_file" || command === "get_file_persons")
    return p?.person
      ? [
          {
            id: 1,
            person_id: 1,
            name: "Sophie",
            person_name: "Sophie",
            thumbnail: photo(2).base64,
            bbox: "[0.2,0.1,0.6,0.8]",
          },
        ]
      : [];
  if (command === "is_face_indexing") return [false, 0];
  if (command === "dedup_start_scan") {
    dedupScanned = true;
    queueMicrotask(() =>
      emit("dedup-scan-progress", {
        state: "finished",
        processed: 20,
        total: 20,
        groups: groups().length,
        isScanning: false,
      }),
    );
    return true;
  }
  if (command === "dedup_get_scan_status")
    return {
      state: dedupScanned ? "finished" : "idle",
      processed: dedupScanned ? 20 : 0,
      total: 20,
      groups: groups().length,
      isScanning: false,
      skipped: 0,
    };
  if (command === "dedup_list_groups") return dedupScanned ? groups() : [];
  if (command === "dedup_get_overview")
    return {
      total_groups: dedupScanned ? groups().length : 0,
      total_files: dedupScanned
        ? groups().reduce((s, g) => s + g.file_count - 1, 0)
        : 0,
      total_reclaimable_bytes: dedupScanned
        ? groups().reduce((s, g) => s + g.file_size * (g.file_count - 1), 0)
        : 0,
    };
  if (command === "dedup_set_keep" || command === "similar_set_keep") {
    keeps.set(args.groupId, args.fileId);
    return true;
  }
  if (command === "similar_start_scan") {
    similarScanned = true;
    queueMicrotask(() =>
      emit("similar-scan-progress", {
        state: "finished",
        current: 20,
        total: 20,
        processed: 20,
        groups: groups(true).length,
        isScanning: false,
        scopeKey: args.scopeKey,
      }),
    );
    return true;
  }
  if (command === "similar_has_scan") return similarScanned;
  if (command === "similar_get_scan_status")
    return {
      state: similarScanned ? "finished" : "idle",
      current: 20,
      total: 20,
      processed: 20,
      isScanning: false,
    };
  if (command === "similar_get_eligible_count") return 20;
  if (command === "similar_list_groups")
    return {
      items: similarScanned
        ? groups(true).slice(
            args.offset || 0,
            (args.offset || 0) + (args.limit || 100),
          )
        : [],
      total: similarScanned ? groups(true).length : 0,
    };
  if (command === "similar_get_group")
    return groups(true).find((g) => g.id === args.groupId);
  if (command === "similar_get_overview")
    return {
      total_groups: groups(true).length,
      total_files: 20,
      total_size: 100000000,
    };
  if (
    command === "get_image_search_model_status" ||
    command === "set_image_search_model" ||
    command === "download_multilingual_image_search_model"
  )
    return { activeModel: args.model || 0, multilingualAvailable: true };
  if (command === "get_thumbnail_cache_location") return { ...thumbnailCacheLocation };
  if (command === "get_thumbnail_cache_migration_count") return 20;
  if (command === "cancel_thumbnail_cache_migration") {
    const migration = thumbnailCacheMigrations.get(args.operationId);
    if (!migration) return false;
    migration.cancelled = true;
    return true;
  }
  if (command === "change_thumbnail_cache_location") {
    const migration = { cancelled: false };
    thumbnailCacheMigrations.set(args.operationId, migration);
    try {
      args.onProgress?.onmessage({ phase: "counting", completed: 0, total: 0 });
      for (let completed = 0; completed <= 20; completed++) {
        await new Promise(resolve => setTimeout(resolve, 50));
        if (migration.cancelled) {
          args.onProgress?.onmessage({ phase: "rolling_back", completed: 0, total: 20 });
          await new Promise(resolve => setTimeout(resolve, 400));
          args.onProgress?.onmessage({ phase: "rolled_back", completed: 0, total: 20 });
          throw { code: "CANCELLED", message: "Cache migration cancelled." };
        }
        args.onProgress?.onmessage({ phase: "copying", completed, total: 20 });
      }
      args.onProgress?.onmessage({ phase: "finishing", completed: 20, total: 20 });
      thumbnailCacheLocation = args.newDir == null
        ? { path: "/Browser/Lap Demo/Cache", custom: false }
        : { path: `${args.newDir}/lap-thumbnail-cache`, custom: true };
      inform("Cache migration is simulated. No real files are moved.");
      return { ...thumbnailCacheLocation, cleanupWarnings: [] };
    } finally {
      thumbnailCacheMigrations.delete(args.operationId);
    }
  }
  if (command === "get_db_storage_dir") return "/Browser/Lap Demo";
  if (command === "get_package_info")
    return {
      name: "Lap",
      version: "0.3.3",
      authors: ["Lap"],
      license: "GPL-3.0-or-later",
      repository: "https://github.com/julyx10/lap",
      homepage: "https://julyx10.github.io/lap/",
      description: "Browser sample library",
    };
  if (command === "get_build_time") return 1792195200;
  if (command === "get_db_storage_info")
    return [
      {
        library_id: "1",
        library_name: "Default Library",
        db_file_size: 100000,
      },
    ];
  if (command === "get_storage_file_info")
    return {
      file_name: "sample.db",
      file_size: 100000,
      size: 100000,
      path: "/Browser/Lap Demo/sample.db",
    };
  if (command === "get_supported_format_extensions")
    return {
      image: ["jpg", "png", "webp"],
      raw: ["dng", "cr2", "nef"],
      video: ["mp4", "mov"],
      options: ["jpg", "png", "webp", "dng", "cr2", "nef", "mp4", "mov"].map(
        (value) => ({ value, label: value.toUpperCase() }),
      ),
    };
  if (command === "get_search_models") return [];
  if (command === "get_index_recovery_info") return null;
  if (command === "start_file_drag") {
    queueMicrotask(() => args.onEvent?.onmessage?.("Done"));
    return true;
  }
  if (command === "get_native_drag_state") return false;
  if (command === "search_images" || command === "search_similar_images") {
    const input = args.params || args,
      origin = photo(input.fileId),
      text = String(
        input.searchText || args.text || args.query || "",
      ).toLowerCase();
    inform(
      "Sample search matches seeded tags and metadata. No AI model runs online.",
    );
    return live()
      .filter((p) =>
        origin
          ? p.tags.some((t) => origin.tags.includes(t))
          : [p.name, ...p.tags, p.location, p.camera, p.person]
              .join(" ")
              .toLowerCase()
              .includes(text) ||
            (text.includes("landscape") && p.tags.includes("Landscape")) ||
            (text.includes("portrait") && !!p.person),
      )
      .map(nativeFile);
  }
  if (command === "index_album") {
    inform(
      "Sample indexing complete. No disk scan or AI model runs in the browser.",
    );
    queueMicrotask(() =>
      emit("index_finished", {
        album_id: args.albumId,
        success: true,
        indexed: 20,
        total: 20,
      }),
    );
    return true;
  }
  if (
    command.startsWith("set_") ||
    command.startsWith("save_") ||
    command.startsWith("record_") ||
    [
      "finish_startup",
      "sync_album_folder_mtimes",
      "clear_index_recovery_info",
    ].includes(command)
  )
    return true;
  if (
    command.startsWith("get_") ||
    command.startsWith("list_") ||
    command.startsWith("fetch_")
  ) {
    console.info("[Demo fixture not implemented]", command, args);
    return [];
  }
  if (command.startsWith("is_") || command.startsWith("has_")) return false;
  inform(
    `Desktop operation preview: ${command.replaceAll("_", " ")}. No real files are changed.`,
  );
  return true;
}
// Exposed for deterministic browser QA, without reaching private Vue internals.
export const demoBackend = {
  commands,
  get state() {
    return db;
  },
  query,
  groups,
  nativeFile,
};
window.__lapDemoBackend = demoBackend;
