// thumbnailWorker.js

self.onmessage = async function (e) {
  const { fileList, currentFolderPath } = e.data;

  try {
    const thumbnailPromises = fileList.map(async (file) => {
      const file_path = currentFolderPath + '\\' + file.name;
      console.log('getFileThumb (worker):', file, file_path);

      // Fetch the thumbnail using Tauri invoke call
      const thumbnail = await invoke('get_file_thumb', { fileId: file.id, path: file_path });

      // Return the file and its thumbnail
      return { file, thumbnail: `data:image/jpeg;base64,${thumbnail}` };
    });

    // Wait for all thumbnails to be fetched
    const results = await Promise.all(thumbnailPromises);

    // Send results back to the main thread
    self.postMessage({ success: true, data: results });
  } catch (error) {
    console.error('getFileThumb error (worker):', error);
    self.postMessage({ success: false, error });
  }
};
