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


    // /// Resize an image to create a thumbnail and return it as a vector of bytes
    // fn get_thumbnail(file_path: &str, orientation: i32, thumbnail_size: u32) -> Result<Option<Self>, String> {
    //     // Attempt to open the image file
    //     let img = match image::open(file_path) {
    //         Ok(image) => image,
    //         Err(_) => return Ok(None), // Return Ok(None) if the image fails to open
    //     };

    //     let img = image::open(img_path).expect("Failed to open image");
    //     // Get the dimensions of the image
    //     let (width, height) = img.dimensions();

    //     // Adjust the image orientation based on the EXIF orientation value
    //     let adjusted_img = match orientation {
    //         3 => img.rotate180(),
    //         6 => img.rotate90(),
    //         8 => img.rotate270(),
    //         _ => img,
    //     };

    //     // Resize the image to a thumbnail
    //     let thumbnail = adjusted_img.thumbnail(thumbnail_size, thumbnail_size);

    //     // Save the thumbnail to an in-memory buffer as a JPEG
    //     let mut buf = Vec::new();
    //     match thumbnail.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg) {
    //         Ok(()) => Ok(Some(buf)),  // Return Ok(Some(buf)) if writing is successful
    //         Err(_) => Ok(None)        // Return Ok(None) if writing fails
    //     }
    // }