// src/router.js
import { createRouter, createWebHistory } from 'vue-router';
// import ImageView from '@/views/ImageView.vue';

const routes = [
  {
    path: '/',
    name: 'Home',
    component: () => import('@/views/Home.vue'),
  },
  {
    path: '/image',
    name: 'ImageView',
    component: () => import('@/views/ImageView.vue'),
  },
  {
    path: '/about',
    name: 'About',
    component: () => import('@/views/About.vue'),
  }
];

const router = createRouter({
  history: createWebHistory(),
  // history: createWebHistory(import.meta.env.BASE_URL),
  routes,
});

export default router;