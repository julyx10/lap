/// get the config stores - using lazy getters to avoid Pinia initialization issues
import { useConfigStore } from '@/stores/configStore';
import { useLibraryStore } from '@/stores/libraryStore';

// Lazy-initialized store instances
let _config = null;
let _libConfig = null;

// Getter that lazily initializes the config store
export const config = new Proxy({}, {
  get(_, prop) {
    if (!_config) {
      _config = useConfigStore();
    }
    return _config[prop];
  },
  set(_, prop, value) {
    if (!_config) {
      _config = useConfigStore();
    }
    _config[prop] = value;
    return true;
  }
});

// Getter that lazily initializes the library store
export const libConfig = new Proxy({}, {
  get(_, prop) {
    if (!_libConfig) {
      _libConfig = useLibraryStore();
    }
    return _libConfig[prop];
  },
  set(_, prop, value) {
    if (!_libConfig) {
      _libConfig = useLibraryStore();
    }
    _libConfig[prop] = value;
    return true;
  }
});