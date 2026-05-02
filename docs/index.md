---
layout: home

hero:
  name: "Lap"
  text: "Local-first, AI-powered Photo manager."
  tagline: "Built for family albums, photographers, and large local libraries."
  image:
    src: /screenshots/lap-home-0.1.10_1.png
    alt: Lap local photo library manager screenshot
  actions:
    - theme: brand
      text: Download
      link: https://github.com/julyx10/lap/releases/latest
    - theme: alt
      text: What's New in v0.2.1
      link: /guide/release-notes/v0.2.1
    - theme: alt
      text: View on GitHub
      link: https://github.com/julyx10/lap

features:
  - title: No Cloud Required
    details: Keep your photos on your own disk. Lap is a local-first photo manager with no forced cloud upload.
    icon: ☁️
  - title: Private by Default
    details: Browsing, indexing, and search run locally on your device so your family albums stay under your control.
    icon: 🔒
  - title: Folder-First Workflow
    details: Use your existing folders directly. No import lock-in, no proprietary library migration.
    icon: 📂
  - title: Local AI Search
    details: Find photos with text search, similar-image search, face clustering, and smart tags processed on-device.
    icon: 🧠
  - title: Built for Large Libraries
    details: Smooth browsing and organization across thousands of photos and videos in real-world collections.
    icon: ⚡
  - title: Free to Use
    details: No subscription plan or recurring fee. Install and manage your memories freely.
    icon: 💸
---

<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'

const isZoomed = ref(false)
const imgSrc = ref('')
let heroCleanup = null

onMounted(() => {
  // Use a MutationObserver or a timeout to wait for VPHero to render if needed,
  // but usually onMounted is enough for static SSR content in VitePress.
  const interval = setInterval(() => {
    const heroImg = document.querySelector('.VPHero .image-src')
    const heroImageWrap = document.querySelector('.VPHero .image')
    if (heroImg && heroImageWrap) {
      heroImg.style.cursor = 'zoom-in'
      heroImg.addEventListener('click', (e) => {
        imgSrc.value = e.target.src
        isZoomed.value = true
      })

      const handleMove = (event) => {
        const rect = heroImageWrap.getBoundingClientRect()
        const rx = ((event.clientX - rect.left) / rect.width - 0.5) * 2
        const ry = ((event.clientY - rect.top) / rect.height - 0.5) * 2
        heroImageWrap.style.setProperty('--hero-tilt-x', `${rx.toFixed(3)}`)
        heroImageWrap.style.setProperty('--hero-tilt-y', `${ry.toFixed(3)}`)
      }

      const resetMove = () => {
        heroImageWrap.style.setProperty('--hero-tilt-x', '0')
        heroImageWrap.style.setProperty('--hero-tilt-y', '0')
      }

      heroImageWrap.addEventListener('mousemove', handleMove)
      heroImageWrap.addEventListener('mouseleave', resetMove)
      heroCleanup = () => {
        heroImageWrap.removeEventListener('mousemove', handleMove)
        heroImageWrap.removeEventListener('mouseleave', resetMove)
      }
      clearInterval(interval)
    }
  }, 100)
  
  // Clean up interval after 5 seconds just in case
  setTimeout(() => clearInterval(interval), 5000)
})

onBeforeUnmount(() => {
  heroCleanup?.()
})
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
  background:
    radial-gradient(circle at 15% 20%, rgba(14, 165, 233, 0.18), transparent 26%),
    radial-gradient(circle at 85% 18%, rgba(245, 158, 11, 0.16), transparent 24%),
    radial-gradient(circle at 72% 72%, rgba(15, 118, 110, 0.18), transparent 30%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.02), transparent 25%);
}

.VPHero {
  position: relative;
  overflow: hidden;
}

.VPHero::before,
.VPHero::after {
  content: "";
  position: absolute;
  border-radius: 999px;
  filter: blur(70px);
  pointer-events: none;
  opacity: 0.7;
}

.VPHero::before {
  top: 2rem;
  right: -4rem;
  width: 18rem;
  height: 18rem;
  background: rgba(14, 165, 233, 0.2);
  animation: heroFloat 12s ease-in-out infinite;
}

.VPHero::after {
  left: -3rem;
  bottom: 1rem;
  width: 16rem;
  height: 16rem;
  background: rgba(245, 158, 11, 0.16);
  animation: heroFloat 14s ease-in-out infinite reverse;
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

.VPHero .image {
  position: relative;
  --hero-tilt-x: 0;
  --hero-tilt-y: 0;
}

.VPHero .image::before {
  content: "";
  position: absolute;
  inset: 10% 0% -8% 8%;
  border-radius: 2rem;
  background:
    linear-gradient(135deg, rgba(14, 165, 233, 0.18), rgba(15, 118, 110, 0.1)),
    rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 40px 120px rgba(0, 0, 0, 0.28);
  transform: rotate(-6deg);
}

.VPHero .image::after {
  content: "";
  position: absolute;
  inset: 16% 8% -12% 16%;
  border-radius: 2rem;
  background:
    radial-gradient(circle at top, rgba(245, 158, 11, 0.24), transparent 55%),
    linear-gradient(115deg, transparent 28%, rgba(255, 255, 255, 0.12) 42%, transparent 56%);
  background-size: auto, 220% 100%;
  background-position: center, 140% 0;
  filter: blur(36px);
  opacity: 0.75;
  animation: glowSweep 8s ease-in-out infinite;
}

.VPHero .image-container {
  isolation: isolate;
}

.VPFeatures {
  position: relative;
  padding-top: 2rem !important;
}

.VPFeatures .container {
  max-width: 1180px !important;
}

.VPFeatures .items {
  gap: 0.75rem !important;
}

.VPFeatures .item {
  padding: 0 !important;
  width: 100% !important;
  max-width: none !important;
  flex: 1 1 auto !important;
}

.VPFeatures .VPLink {
  height: 100%;
}

.VPFeatures .box {
  height: 100%;
  border-radius: 24px !important;
  padding: 1.15rem 1.05rem 1.05rem !important;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.03)),
    rgba(255, 255, 255, 0.02);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.05),
    0 18px 50px rgba(0, 0, 0, 0.16);
  transition:
    transform 0.22s ease,
    border-color 0.22s ease,
    box-shadow 0.22s ease,
    background 0.22s ease;
}

.VPFeatures .box:hover {
  transform: translateY(-6px);
  border-color: rgba(14, 165, 233, 0.28);
  background:
    linear-gradient(180deg, rgba(14, 165, 233, 0.08), rgba(255, 255, 255, 0.04)),
    rgba(255, 255, 255, 0.03);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.06),
    0 24px 60px rgba(0, 0, 0, 0.22);
}

.VPFeatures .icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.8rem;
  height: 2.8rem;
  margin-bottom: 0.9rem;
  border-radius: 0.95rem;
  font-size: 1.25rem;
  background: linear-gradient(135deg, rgba(14, 165, 233, 0.16), rgba(245, 158, 11, 0.14));
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12);
}

.VPFeatures .title {
  font-size: 1.02rem !important;
  font-weight: 700 !important;
  line-height: 1.35 !important;
}

.VPFeatures .details {
  margin-top: 0.55rem !important;
  font-size: 0.95rem !important;
  line-height: 1.65 !important;
  color: var(--vp-c-text-2) !important;
}

@media (min-width: 960px) {
  .VPFeatures .items {
    display: grid !important;
    grid-template-columns: repeat(3, minmax(0, 1fr)) !important;
    gap: 0.85rem !important;
  }

  .VPFeatures .item {
    min-width: 0 !important;
  }

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
    perspective: 1600px !important;
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

  .VPHero .image-container::before,
  .VPHero .image-container::after {
    content: "";
    position: absolute;
    inset: auto;
    width: 78%;
    aspect-ratio: 16 / 10;
    border-radius: 18px;
    background-size: cover;
    background-position: center;
    border: 1px solid rgba(255, 255, 255, 0.12);
    box-shadow: 0 28px 80px rgba(0, 0, 0, 0.26);
    transform-style: preserve-3d;
    pointer-events: none;
  }

  .VPHero .image-container::before {
    top: 9%;
    left: -7%;
    background-image: url('/screenshots/lap-home-0.1.10_2.png');
    transform:
      rotateX(calc(10deg + var(--hero-tilt-y) * -2deg))
      rotateY(calc(22deg + var(--hero-tilt-x) * 4deg))
      rotateZ(-10deg)
      translateY(calc(var(--hero-tilt-y) * 10px))
      scale(0.9);
    opacity: 0.92;
    animation: sideCardLeft 10s ease-in-out infinite;
  }

  .VPHero .image-container::after {
    right: -5%;
    bottom: 6%;
    background-image: url('/screenshots/lap-home3.png');
    transform:
      rotateX(calc(2deg + var(--hero-tilt-y) * 2deg))
      rotateY(calc(-18deg + var(--hero-tilt-x) * 4deg))
      rotateZ(8deg)
      translateY(calc(var(--hero-tilt-y) * -8px))
      scale(0.86);
    opacity: 0.88;
    animation: sideCardRight 11s ease-in-out infinite;
  }

  .VPHero .image-src {
    border-radius: 18px !important;
    border: 1px solid rgba(255, 255, 255, 0.16) !important;
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.34) !important;
    transform:
      rotateX(calc(7deg + var(--hero-tilt-y) * -3deg))
      rotateY(calc(-10deg + var(--hero-tilt-x) * 7deg))
      rotateZ(1deg)
      translate3d(calc(var(--hero-tilt-x) * 6px), calc(var(--hero-tilt-y) * -10px), 0) !important;
    transition:
      transform 0.5s cubic-bezier(0.4, 0, 0.2, 1),
      box-shadow 0.5s ease !important;
    position: relative !important;
    z-index: 2;
    will-change: transform;
  }

  .VPHero .image-src:hover {
    box-shadow: 0 40px 110px rgba(0, 0, 0, 0.42) !important;
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

  .VPHero .image::before {
    inset: 6% 4% -4% 6%;
    transform: rotate(-2deg);
  }

  .VPHero .image::after,
  .VPHero .image-container::before,
  .VPHero .image-container::after {
    display: none;
  }

  .VPFeatures {
    padding-top: 1rem !important;
  }
}

@keyframes heroFloat {
  0%, 100% { transform: translate3d(0, 0, 0); }
  50% { transform: translate3d(0, 18px, 0); }
}

@keyframes sideCardLeft {
  0%, 100% { transform: rotateX(10deg) rotateY(22deg) rotateZ(-10deg) translateY(0) scale(0.9); }
  50% { transform: rotateX(8deg) rotateY(18deg) rotateZ(-8deg) translateY(10px) scale(0.92); }
}

@keyframes sideCardRight {
  0%, 100% { transform: rotateX(2deg) rotateY(-18deg) rotateZ(8deg) translateY(0) scale(0.86); }
  50% { transform: rotateX(4deg) rotateY(-14deg) rotateZ(6deg) translateY(-8px) scale(0.88); }
}

@keyframes glowSweep {
  0%, 100% { background-position: center, 140% 0; }
  50% { background-position: center, -40% 0; }
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
