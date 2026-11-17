<script setup>
const features = [
  {
    title: 'Folder-based library management',
    description: 'Open your existing folders and keep files in their original locations. Browse libraries with 100,000+ photos and videos, and revisit disconnected drives with cached thumbnails.',
  },
  {
    title: 'Local AI search',
    description: 'Search with a phrase like “sunset at the beach,” find visually similar photos, or browse by face. All AI processing runs on your device.',
  },
  {
    title: 'Smart Albums and tags',
    description: 'Save rule-based views, group photos in collections, and organize files in bulk with hierarchical tags.',
  },
  {
    title: 'Metadata and XMP',
    description: 'Read EXIF, IPTC, and XMP, edit titles and descriptions, and save ratings, tags, and other supported fields to XMP sidecars.',
  },
  {
    title: 'Photo culling',
    description: 'Preview with Space, advance automatically after rating, and compare up to four photos with autofocus areas available for supported files.',
  },
  {
    title: 'Duplicate cleanup',
    description: 'Review exact duplicates and similar shots, set keep rules by folder or file properties, and see how much space you can reclaim.',
  },
  {
    title: 'RAW and paired files',
    description: 'Choose between RAW rendering and embedded previews, and browse RAW + JPEG/HEIC pairs as one item.',
  },
  {
    title: 'Editing and collages',
    description: 'Crop, straighten, and adjust photos, or combine a selection into a grid, mosaic, or photo pile.',
  },
]
</script>

<template>
  <section class="lap-features" aria-labelledby="features-heading">
    <h2 id="features-heading">Features</h2>
    <div class="feature-grid">
      <article v-for="feature in features" :key="feature.title">
        <h3>{{ feature.title }}</h3>
        <p>{{ feature.description }}</p>
      </article>
    </div>
  </section>
</template>

<style scoped>
.lap-features {
  max-width: 1216px;
  margin: clamp(48px, 7vw, 88px) auto clamp(56px, 8vw, 104px);
  padding: 0 24px;
}

h2 {
  margin-bottom: 24px;
  font-size: clamp(28px, 4vw, 40px);
  line-height: 1.2;
  font-weight: 700;
  letter-spacing: -0.03em;
}

.feature-grid {
  display: grid;
  gap: 16px;
}

article {
  padding: clamp(20px, 2.5vw, 32px);
  border: 1px solid var(--vp-c-divider);
  border-radius: 16px;
  background: var(--vp-c-bg-soft);
}

h3 {
  margin-bottom: 8px;
  font-size: 18px;
  line-height: 1.3;
  font-weight: 650;
}

p {
  max-width: 56ch;
  font-size: 15px;
  line-height: 1.65;
  color: var(--vp-c-text-2);
}

@media (min-width: 640px) {
  .feature-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (min-width: 960px) {
  .lap-features {
    padding: 0 32px;
  }

  .feature-grid {
    grid-template-columns: repeat(6, minmax(0, 1fr));
  }

  article {
    grid-column: span 2;
  }

  article:nth-child(-n + 2) {
    grid-column: span 3;
    background: linear-gradient(145deg, var(--vp-c-brand-soft), var(--vp-c-bg-soft) 74%);
  }

  article:nth-child(-n + 2) h3 {
    font-size: 23px;
  }
}
</style>
