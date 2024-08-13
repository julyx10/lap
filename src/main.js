// When using the Tauri global script (if not using the npm package)
// Be sure to set `build.withGlobalTauri` in `tauri.conf.json` to true
const { invoke } = window.__TAURI__.tauri;

async function fetchAndDisplayAlbums() {
    try {
        // Fetch the albums data from the Tauri command
        const albums = await invoke('get_albums');

        // Get the album list container
        const albumList = document.getElementById('album-list');

        // Check if the container exists
        if (!albumList) {
            throw new Error('Album list container not found');
        }

        // Clear any existing content
        albumList.innerHTML = '';

        // Append each album to the list
        albums.forEach(album => {
            const listItem = document.createElement('li');
            listItem.className = 'album-item';
            listItem.innerHTML = `
                <strong>Name:</strong> ${album.name}<br>
                <strong>Description:</strong> ${album.description ? album.description : 'N/A'}<br>
                <strong>Location:</strong> ${album.location}<br>
                <strong>Created At:</strong> ${new Date(album.created_at * 1000).toLocaleString()}<br>
                <strong>Updated At:</strong> ${new Date(album.updated_at * 1000).toLocaleString()}
            `;
            albumList.appendChild(listItem);
        });
    } catch (error) {
        console.error('Failed to fetch albums:', error);
    }
}

// Call the function to fetch and display albums when the page loads
fetchAndDisplayAlbums();

window.addEventListener("DOMContentLoaded", () => {

  // click open a folder button
  document.querySelector("#add-folder-button").addEventListener("click", async (e) => {
    await invoke("add_folder").then((res) => {
      if (res === null) {
        res = "No folder selected";
      } else {
        res = "Folder selected: " + res;
      }
      document.querySelector("#folder-path").textContent = res;
    });
  });

  // click open a file button
  document.querySelector("#open-file-button").addEventListener("click", async (e) => {
    await invoke("open_file").then((res) => {
      if (res === null) {
        res = "No file selected";
      } else {
        res = "File selected: " + res;
      }
      document.querySelector("#file-path").textContent = res;
      document.querySelector("#exif-data").textContent += res;
    });
  });
});
