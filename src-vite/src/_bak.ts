// async function getThumbnail() {
//   try {
//     for (let file of file_list.value) {
//       const file_path = current_folder.value.path + '\\' + file.name;
//       console.log('getThumbnail:', file, file_path);

//       // Send the task to the worker
//       worker.postMessage({ fileId: file.id, path: file_path });

//       // Handle the response from the worker
//       worker.onmessage = function (event) {
//         const { fileId, thumbnail } = event.data;
//         const fileToUpdate = file_list.value.find(f => f.id === fileId);
//         if (fileToUpdate) {
//           fileToUpdate.thumbnail = `data:image/png;base64,${thumbnail}`;
//         }
//       };
//     }
//   } catch (error) {
//     console.error('getThumbnail error:', error);
//     }
// }


// // Function to fetch thumbnails using a Web Worker
// function fetchThumbnailsWithWorker(path) {
//   // const worker = new Worker('/thumbnailWorker.js');
//   // Import the worker using Vite's special worker syntax
//   const worker = new Worker(new URL('../workers/thumbnailWorker.js', import.meta.url), { type: 'module' });

//   // Extract cloneable properties (id, name, etc.) from file_list
//   const cloneableFileList = file_list.value.map(file => ({
//     id: file.id,
//     name: file.name
//   }));

//   // Post message to the worker with the simplified file list and folder path
//   worker.postMessage({
//     fileList: cloneableFileList, 
//     currentFolderPath: path
//   });

//   // Listen for messages from the worker
//   worker.onmessage = (e) => {
//     const { success, data, error } = e.data;

//     if (success) {
//       // Update file_list with thumbnails returned by the worker
//       data.forEach((item) => {
//         const file = file_list.value.find(f => f.id === item.file.id);
//         if (file) {
//           file.thumbnail = item.thumbnail;
//         }
//       });
//     } else {
//       console.error('Thumbnail fetch failed:', error);
//     }

//     // Terminate the worker once done
//     worker.terminate();
//   };

// }