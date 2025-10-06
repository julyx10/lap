
import { defineStore } from 'pinia';

export const useUIStore = defineStore('ui', {
  state: () => ({
    inputStack: [],
  }),
  getters: {
    isInputActive: (state) => (name) => {
      return state.inputStack.length > 0 && state.inputStack[state.inputStack.length - 1] === name;
    },
  },
  actions: {
    pushInputHandler(name) {
      this.inputStack.push(name);
    },
    popInputHandler() {
      this.inputStack.pop();
    },
  },
});
