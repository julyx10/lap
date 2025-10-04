<template>
  <router-view />
</template>
 
<script setup>
import { onMounted, onUnmounted } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

const handleKeyDown = (event) => {
  // Allow paste operations (Ctrl+V / Cmd+V) to work normally
  const isPasteOperation = (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'v';
  
  // Prevent default browser shortcuts if a modifier key is pressed, except for paste
  if ((event.ctrlKey || event.metaKey || event.altKey) && !isPasteOperation) {
    event.preventDefault();
  }

  emit('global-keydown', {
    key: event.key,
    altKey: event.altKey,
    ctrlKey: event.ctrlKey,
    metaKey: event.metaKey,
    shiftKey: event.shiftKey,
  });
};

onMounted(async () => {
  const win = getCurrentWebviewWindow();
  if (win.label === 'main') {
    window.addEventListener('keydown', handleKeyDown);
  }
});

onUnmounted(async () => {
  const win = getCurrentWebviewWindow();
  if (win.label === 'main') {
    window.removeEventListener('keydown', handleKeyDown);
  }
});
</script>
