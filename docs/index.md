---
layout: home

hero:
  name: "Lap"
  text: "Local-first, AI-powered Photo manager."
  tagline: "Built for family albums, photographers, and large local libraries."
  image:
    src: /screenshots/lap_library.png
    alt: Lap local photo library manager screenshot
  actions:
    - theme: brand
      text: Download
      link: https://github.com/julyx10/lap/releases/latest
    - theme: alt
      text: What's New in v0.4.0
      link: /guide/release-notes/v0.4.0
    - theme: alt
      text: View on GitHub
      link: https://github.com/julyx10/lap

---

<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'

const isZoomed = ref(false)
const imgSrc = ref('')
let heroCleanup

onMounted(() => {
  const heroImg = document.querySelector('.VPHero .image-src')
  if (!heroImg) return
  const openPreview = () => {
    imgSrc.value = heroImg.src
    isZoomed.value = true
  }
  heroImg.style.cursor = 'zoom-in'
  heroImg.addEventListener('click', openPreview)
  heroCleanup = () => heroImg.removeEventListener('click', openPreview)
})

onBeforeUnmount(() => heroCleanup?.())
</script>

<style>
:root {
  --vp-home-hero-name-color: var(--vp-c-text-1);
  --vp-home-hero-name-background: none;
}

.VPNavBarTitle .logo {
  border-radius: 8px;
}

.VPHome {
  background: var(--vp-c-bg);
}

.VPHero {
  position: relative;
  overflow: hidden;
}

.VPHero .image-bg {
  display: none;
}

.VPHero .image,
.VPHero .image-container,
.VPHero .image-src {
  background: transparent;
}

.VPHero .container {
  position: relative;
  z-index: 1;
}

.VPHero .main {
  position: relative;
}

.VPHero .name,
.VPHero .text,
.VPHero .tagline {
  max-width: 34rem;
}

.VPHero .name {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
}

.VPHero .name::before {
  content: "";
  width: 4.6rem;
  height: 4.6rem;
  flex: 0 0 4.6rem;
  border-radius: 1rem;
  background: url('/icon.png') center/cover no-repeat;
  box-shadow:
    0 10px 30px rgba(14, 165, 233, 0.22),
    0 4px 12px rgba(0, 0, 0, 0.16);
}

.VPHero .text {
  font-weight: 700;
  letter-spacing: -0.03em;
}

.VPHero .tagline {
  font-size: 1.05rem;
  line-height: 1.75;
}

.VPHero .download-note {
  margin: 16px 0 0;
  font-size: 0.875rem;
  line-height: 1.6;
  color: var(--vp-c-text-2);
}

.VPHero .actions {
  gap: 0.9rem;
}

.VPHero .action .VPButton {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  padding: 0 1.1rem;
  min-height: 2.9rem;
  line-height: 1;
  transition: transform 0.2s ease, box-shadow 0.25s ease, border-color 0.25s ease;
}

.VPHero .action .VPButton.brand {
  box-shadow: 0 16px 40px rgba(14, 165, 233, 0.28);
}

.VPHero .action .VPButton.alt {
  border-color: rgba(255, 255, 255, 0.14);
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
}

.VPHero .action .VPButton:hover {
  transform: translateY(-2px);
}

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
    position: relative !important;
  }

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

  .VPHero .image-src {
    border-radius: 12px !important;
    border: none;
    box-shadow: none;
  }

  .VPHero .name {
    font-size: 3.8rem !important;
    line-height: 1 !important;
  }

  .VPHero .text {
    font-size: 2.35rem !important;
    line-height: 1.08 !important;
  }
}

@media (max-width: 959px) {
  .VPHero .text {
    font-size: 2.1rem !important;
    line-height: 1.12 !important;
  }

  .VPHero .tagline {
    font-size: 0.98rem;
  }

  .VPHero .name::before {
    width: 3.6rem;
    height: 3.6rem;
    flex-basis: 3.6rem;
    border-radius: 0.9rem;
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
