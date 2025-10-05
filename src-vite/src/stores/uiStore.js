
import { defineStore } from 'pinia';

export const useUIStore = defineStore('ui', {
  state: () => ({
    isInputActive: false,
  }),
  actions: {
    setInputActive(isActive) {
      this.isInputActive = isActive;
    },
  },
});
