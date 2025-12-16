// src/router.js
import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  {
    path: '/',
    name: 'Main',
    component: () => import('@/views/Main.vue'),
  },
  {
    path: '/image-viewer',
    name: 'ImageViewer',
    component: () => import('@/views/ImageViewer.vue'),
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('@/views/Settings.vue'),
  }
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;