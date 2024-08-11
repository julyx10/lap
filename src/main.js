// When using the Tauri global script (if not using the npm package)
// Be sure to set `build.withGlobalTauri` in `tauri.conf.json` to true
const { invoke } = window.__TAURI__.tauri;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  document.querySelector("#greet-msg").textContent = await invoke("greet", { name: document.querySelector("#greet-input").value });
}

window.addEventListener("DOMContentLoaded", () => {

  // click greet button
  document.querySelector("#greet-button").addEventListener("click", (e) => {
    e.preventDefault();
    greet();
  });

  // click open a folder button
  document.querySelector("#open-folder-button").addEventListener("click", async (e) => {
    await invoke("open_folder").then((res) => {
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
