
import { defineStore } from 'pinia';

export const useUIStore = defineStore('ui', {
  state: () => ({
    inputStack: [],
    fileVersions: {},
    isFullScreen: false,
  }),
  getters: {
    isInputActive: (state) => (name) => {
      return state.inputStack.length > 0 && state.inputStack[state.inputStack.length - 1] === name;
    },
    getFileVersion: (state) => (filePath) => {
      return state.fileVersions[filePath] || 0;
    },
  },
  actions: {
    pushInputHandler(name) {
      this.inputStack.push(name);
    },
    popInputHandler() {
      this.inputStack.pop();
    },
    updateFileVersion(filePath) {
      const currentVersion = this.fileVersions[filePath] || 0;
      this.fileVersions[filePath] = currentVersion + 1;
    },
  },
});
