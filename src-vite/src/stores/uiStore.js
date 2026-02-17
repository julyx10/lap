
import { defineStore } from 'pinia';

export const useUIStore = defineStore('ui', {
  state: () => ({
    inputStack: [],
    fileVersions: {},
    isFullScreen: false,
    activeAdjustments: {
      filePath: null,
      brightness: 0,
      contrast: 0,
      saturation: 100,
      hue: 0,
      blur: 0,
      filter: null
    }
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
    removeInputHandler(name) {
      const index = this.inputStack.indexOf(name);
      if (index !== -1) {
        this.inputStack.splice(index, 1);
      }
    },
    updateFileVersion(filePath) {
      const currentVersion = this.fileVersions[filePath] || 0;
      this.fileVersions[filePath] = currentVersion + 1;
    },
    setActiveAdjustments(filePath, adjustments) {
      this.activeAdjustments = {
        filePath,
        ...adjustments
      };
    },
    clearActiveAdjustments() {
      this.activeAdjustments = {
        filePath: null,
        brightness: 0,
        contrast: 0,
        saturation: 100,
        hue: 0,
        blur: 0,
        filter: null
      };
    }
  },
});
