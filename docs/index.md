---
layout: home

hero:
  name: "Lap"
  text: "Local-first, AI-powered Photo manager."
  tagline: ""
  image:
    src: /screenshots/lap-home3.png
    alt: Lap Screenshot
  actions:
    - theme: brand
      text: Download
      link: https://github.com/julyx10/lap/releases/latest
    - theme: alt
      text: What's New in v0.1.6
      link: /guide/release-notes/v0.1.6
    - theme: alt
      text: View on GitHub
      link: https://github.com/julyx10/lap

features:
  - title: 100% Private
    details: Stop trading privacy for convenience. Runs entirely offline. Your photos never leave your device.
    icon: 🔒
  - title: High Performance
    details: Built with Rust and Tauri. Designed to handle libraries with 100,000+ assets with ease.
    icon: ⚡
  - title: Smart Local AI
    details: Search by content, recognize faces, and find similar shots—all processed 100% locally.
    icon: 🧠
  - title: Direct Folder Management
    details: No proprietary databases. Just add a folder to manage files directly in Lap. External changes like renaming sync easily.
    icon: 📂
  - title: Multi-Platform
    details: Native on macOS (Apple Silicon & Intel) and Linux. Windows support coming soon.
    icon: 🖥️
  - title: Open Source
    details: Community-driven and transparent source code.
    icon: 🤝
---

<script setup>
import { ref, onMounted } from 'vue'

const isZoomed = ref(false)
const imgSrc = ref('')

onMounted(() => {
  // Use a MutationObserver or a timeout to wait for VPHero to render if needed,
  // but usually onMounted is enough for static SSR content in VitePress.
  const interval = setInterval(() => {
    const heroImg = document.querySelector('.VPHero .image-src')
    if (heroImg) {
      heroImg.style.cursor = 'zoom-in'
      heroImg.addEventListener('click', (e) => {
        imgSrc.value = e.target.src
        isZoomed.value = true
      })
      clearInterval(interval)
    }
  }, 100)
  
  // Clean up interval after 5 seconds just in case
  setTimeout(() => clearInterval(interval), 5000)
})
</script>

<style>
:root {
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: -webkit-linear-gradient(120deg, #bd34fe 30%, #41d1ff);
}

.VPNavBarTitle .logo {
  border-radius: 8px;
}

/* Desktop layout - side by side 50/50 */
@media (min-width: 960px) {
  .VPHero .container {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
    padding: 0 16px;
  }
  
  .VPHero.has-image .main {
    flex: 0 0 50% !important;
    width: 50% !important;
    max-width: none !important;
    padding-right: 48px;
  }
  
  .VPHero .image {
    flex: 0 0 50% !important;
    width: 50% !important;
    display: flex !important;
    justify-content: center;
    align-items: center;
    order: 2 !important;
    margin: 0 !important;
  }

  .VPHero .image-container {
    width: 100% !important;
    height: auto !important;
    max-width: none !important;
    transform: none !important;
    perspective: 1000px !important;
  }

  /* Force image and background to fill the 50% container */
  .VPHero .image-src,
  .VPHero .image-bg {
    position: relative !important;
    top: auto !important;
    left: auto !important;
    transform: none !important;
    max-width: 100% !important;
    max-height: none !important;
    width: 100% !important;
  }

  /* User adjustment for image scaling and perspective */
  .VPHero .image-src {
    transform: rotateX(5deg) rotateY(-10deg) rotateZ(0deg) !important;
    transition: transform 0.5s cubic-bezier(0.4, 0, 0.2, 1) !important;
  }

  .VPHero .image-src:hover {
    transform: rotateY(0deg) rotateX(0deg) scale(1.05) !important;
  }

  /* Smaller font size for hero text */
  .VPHero .text {
    font-size: 32px !important;
    line-height: 40px !important;
  }
}

@media (min-width: 960px) {
  .VPHero .text {
    font-size: 40px !important;
    line-height: 48px !important;
  }
}

/* Lightbox Styles */
.lightbox-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 9999;
  backdrop-filter: blur(8px);
  cursor: zoom-out;
  animation: fadeIn 0.3s ease;
}

.lightbox-img {
  max-width: 90vw;
  max-height: 90vh;
  object-fit: contain;
  box-shadow: 0 0 40px rgba(0,0,0,0.5);
  transform-origin: center;
  animation: zoomIn 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes zoomIn {
  from { transform: scale(0.8); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}

.close-btn {
  position: absolute;
  top: 20px;
  right: 40px;
  color: white;
  font-size: 48px;
  font-weight: 200;
  cursor: pointer;
  line-height: 1;
  transition: transform 0.2s ease;
}

.close-btn:hover {
  transform: scale(1.1);
}
</style>

<div v-if="isZoomed" class="lightbox-overlay" @click="isZoomed = false">
  <img :src="imgSrc" class="lightbox-img" />
  <div class="close-btn">&times;</div>
</div>
