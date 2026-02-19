
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
      rotate: 0,
      flipX: false,
      flipY: false,
      filter: null,
      resize: null
    }
  }),
  getters: {
    isInputActive: (state) => (name) => {
      return state.inputStack.length > 0 && state.inputStack[state.inputStack.length - 1] === name;
    },
    getFileVersion: (state) => (filePath) => {
      return state.fileVersions[filePath] || 0;
    },
    hasActiveChanges: (state) => (fileInfo) => {
      if (!state.activeAdjustments.filePath || !fileInfo) return false;
      if (state.activeAdjustments.filePath !== fileInfo.file_path) return false;

      const adj = state.activeAdjustments;
      const hasAdjustments = adj.brightness !== 0 || 
                             adj.contrast !== 0 || 
                             adj.saturation !== 100 || 
                             adj.hue !== 0 || 
                             adj.blur !== 0 || 
                             !!adj.filter;
      
      const hasTransform = adj.rotate !== 0 || adj.flipX || adj.flipY;
      
      let hasResize = false;
      if (adj.resize) {
        hasResize = Math.round(adj.resize.width) !== Math.round(fileInfo.width) || 
                    Math.round(adj.resize.height) !== Math.round(fileInfo.height);
      }
      
      return hasAdjustments || hasTransform || hasResize;
    }
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
        ...this.activeAdjustments,
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
        rotate: 0,
        flipX: false,
        flipY: false,
        filter: null,
        resize: null
      };
    }
  },
});
