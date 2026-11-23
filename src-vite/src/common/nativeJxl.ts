// Failed original URLs include the source path and file versions. Keep only a
// bounded, per-WebView-session record; never disable JXL globally for one file.
const failedOriginals = new Set<string>();

export async function loadNativeJxl(
  original: string,
  fallback: () => string,
  load: (src: string) => Promise<{ src: string; naturalWidth: number; naturalHeight: number }>,
) {
  if (original && !failedOriginals.has(original)) {
    try { return await load(original); }
    catch {
      failedOriginals.add(original);
      if (failedOriginals.size > 256) failedOriginals.delete(failedOriginals.values().next().value!);
    }
  }
  return load(fallback());
}

export function loadBrowserImage(src: string): Promise<{ src: string; naturalWidth: number; naturalHeight: number }> {
  return new Promise((resolve, reject) => {
    if (!src) { reject(new Error('Missing image source')); return; }
    const img = new Image();
    img.decoding = 'async';
    img.onerror = () => reject(new Error('Image load failed'));
    img.onload = async () => {
      try {
        await img.decode();
        if (!img.naturalWidth || !img.naturalHeight) throw new Error('Empty image');
        resolve({ src, naturalWidth: img.naturalWidth, naturalHeight: img.naturalHeight });
      } catch (error) { reject(error); }
    };
    img.src = src;
  });
}
