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

<script setup>
</script>

<style>
:root {
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: -webkit-linear-gradient(120deg, #bd34fe 30%, #41d1ff);
}
</style>
