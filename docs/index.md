---
layout: home

hero:
  name: "Lap"
  text: "Your Photos, Your Rules."
  tagline: "The local-first, AI-powered photo manager."
  image:
    src: /logo.png
    alt: Lap Logo
  actions:
    - theme: brand
      text: Download
      link: https://github.com/julyx10/lap/releases/latest
    - theme: alt
      text: View on GitHub
      link: https://github.com/julyx10/lap

features:
  - title: Privacy First
    details: Stop trading your privacy for convenience. Lap runs 100% offline. Your photos never leave your device.
    icon: 🔒
  - title: Blazing Fast
    details: Built with Rust and Tauri. Designed to handle libraries with 100,000+ assets smoothly.
    icon: ⚡
  - title: Local AI Magic
    details: Find "cat in the grass", recognize faces, and discover similar shots—all powered by local AI.
    icon: 🧠
  - title: File System Sync
    details: No proprietary database lock-in. Move a file in Finder, it updates in Lap instantly.
    icon: 📂
  - title: Cross Platform
    details: Available on macOS (Apple Silicon). Windows version coming soon.
    icon: 🖥️
  - title: Open Source
    details: Community driven and transparent. Built with modern tech like Vue and Tauri.
    icon: 🤝
---

## Screenshots

<div class="screenshots-grid">
  <div class="screenshot-item">
    <h4>Library - Dark Theme</h4>
    <img src="/screenshots/01 lap-main-dark.jpg" alt="Library" />
    <p>Handle 100k+ photos with smooth scrolling and instant thumbnails.</p>
  </div>
  <div class="screenshot-item">
    <h4>AI Search - Light Theme</h4>
    <img src="/screenshots/02 lap-ai-search.jpg" alt="AI Search" />
    <p>"Sunset over the ocean" — Find exact moments with offline AI.</p>
  </div>
  <div class="screenshot-item">
    <h4>Calendar</h4>
    <img src="/screenshots/03 lap-calendar.jpg" alt="Calendar" />
    <p>Time-travel through your memories by year, month, or day.</p>
  </div>
  <div class="screenshot-item">
    <h4>Tags</h4>
    <img src="/screenshots/04 lap-tag.jpg" alt="Tags" />
    <p>Organize your way: Bulk tagging and powerful filtering.</p>
  </div>
  <div class="screenshot-item">
    <h4>Location</h4>
    <img src="/screenshots/05 lap-location.jpg" alt="Location" />
    <p>Interactive Map View: See where your story happened.</p>
  </div>
  <div class="screenshot-item">
    <h4>Camera</h4>
    <img src="/screenshots/06 lap-camera.jpg" alt="Camera" />
    <p>Group by Gear: Rediscover shots by Camera Make & Model.</p>
  </div>
  <div class="screenshot-item">
    <h4>Edit Image</h4>
    <img src="/screenshots/07 lap-edit-image.jpg" alt="Edit Image" />
    <p>Quick essential edits without leaving your flow.</p>
  </div>
  <div class="screenshot-item">
    <h4>Video</h4>
    <img src="/screenshots/08 lap-video.jpg" alt="Video" />
    <p>Integrated Player: Watch videos seamlessly alongside photos.</p>
  </div>
</div>

<script setup>
</script>

<style>
:root {
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: -webkit-linear-gradient(120deg, #bd34fe 30%, #41d1ff);
}

.screenshots-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 24px;
  margin-top: 32px;
  padding: 0 24px;
}

@media (max-width: 768px) {
  .screenshots-grid {
    grid-template-columns: 1fr;
  }
}

.screenshot-item {
  text-align: center;
  background: var(--vp-c-bg-soft);
  border-radius: 12px;
  padding: 16px;
}

.screenshot-item h4 {
  margin: 0 0 12px 0;
  font-weight: 600;
  color: var(--vp-c-text-1);
}

.screenshot-item img {
  width: 100%;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.screenshot-item p {
  margin: 12px 0 0 0;
  font-size: 14px;
  color: var(--vp-c-text-2);
  font-style: italic;
}
</style>
