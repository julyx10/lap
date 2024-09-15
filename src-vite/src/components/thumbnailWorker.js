self.onmessage = async function (event) {
  const { fileId, path } = event.data;
  try {
    const thumbnail = await invoke('get_file_thumb', { fileId, path });
    self.postMessage({ fileId, thumbnail });
  } catch (error) {
    console.error('Worker error:', error);
  }
};
