import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { config } from '@/common/config';
import { separator, localeComp } from '@/common/utils';

// library

// get app config (libraries list and current library)
export async function getAppConfig() {
  try {
    const config = await invoke('get_app_config');
    if (config) {
      return config;
    }
  } catch (error) {
    console.error('Failed to get app config:', error);
  }
  return null;
}

// add a new library
export async function addLibrary(name) {
  try {
    const library = await invoke('add_library', { name });
    if (library) {
      return library;
    }
  } catch (error) {
    console.error('Failed to add library:', error);
    throw error;
  }
  return null;
}

// edit library name
export async function editLibrary(id, name) {
  try {
    await invoke('edit_library', { id, name });
    return true;
  } catch (error) {
    console.error('Failed to edit library:', error);
    throw error;
  }
}

// remove a library (also deletes the database file)
export async function removeLibrary(id) {
  try {
    await invoke('remove_library', { id });
    return true;
  } catch (error) {
    console.error('Failed to remove library:', error);
    throw error;
  }
}

// switch to a different library
export async function switchLibrary(id) {
  try {
    await invoke('switch_library', { id });
    return true;
  } catch (error) {
    console.error('Failed to switch library:', error);
    throw error;
  }
}

// get library info
export async function getLibraryInfo(id) {
  try {
    const info = await invoke('get_library_info', { id });
    if (info) {
      return info;
    }
  } catch (error) {
    console.error('Failed to get library info:', error);
  }
  return null;
}

// save library state
export async function saveLibraryState(id, state) {
  try {
    await invoke('save_library_state', { id, state });
    return true;
  } catch (error) {
    console.error('Failed to save library state:', error);
  }
  return false;
}

// get library state
export async function getLibraryState(id) {
  try {
    const state = await invoke('get_library_state', { id });
    if (state) {
      return state;
    }
  } catch (error) {
    console.error('Failed to get library state:', error);
  }
  return null;
}

// get current library state
export async function getCurrentLibraryState() {
  try {
    const state = await invoke('get_current_library_state');
    if (state) {
      return state;
    }
  } catch (error) {
    console.error('Failed to get current library state:', error);
  }
  return null;
}

// albums

// get all albums
export async function getAllAlbums() {
  try {
    let albums = [];
    const fetchedAlbums = await invoke('get_all_albums');
    console.log('get_all_albums', fetchedAlbums);
    if (fetchedAlbums) {
      albums = fetchedAlbums.map(album => ({
        ...album, 
        is_expanded: false,
        children: null,
      }));

       // get album's favorite status
       for (let i = 0; i < albums.length; i++) {
        const album = albums[i];
        album.is_favorite = await getFolderFavorite(album.path);
      }

      return albums;
    } 
  } catch (error) {
    console.error('getAlbums...', error);
  }
  return null;
};

// get one album by id
export async function getAlbum(albumId) {
  if(!albumId) {
    return null;
  }
  try {
    const album = await invoke('get_album', { albumId });
    console.log('get_album', album);
    if (album) {
      return album;
    }
  } catch (error) {
    console.error('getAlbum...', error);
  }
  return null;
}

// add an album to db
export async function addAlbum(folderPath) {
  if(!folderPath) {
    return null;
  }
  try {
    const newAlbum = await invoke('add_album', { folderPath });
    console.log('add_album', newAlbum);
    if(newAlbum) {
      return {
        ...newAlbum,
        is_expanded: false,
        children: null,
      };
    };
  } catch (error) {
    console.log('Failed to add album:', error);
  }
  return null;
}

// edit an album's profile
export async function editAlbum(albumId, newName, newDespription) {
  try {
    const album = await invoke('edit_album', { id: albumId, name: newName, description: newDespription });
    console.log('edit_album', album);
    if (album) {
      return album;
    }
  } catch (error) {
    console.log('Failed to edit album:', error);
  }
  return null;
}

// remove an album
export async function removeAlbum(albumId) {
  try {
    const removedAlbum = await invoke('remove_album', { id: albumId });
    console.log('remove_album', removedAlbum);
    if (removedAlbum) {
      return removedAlbum;
    }
  } catch (error) {
    console.log('Failed to remove album:', error);
  }
  return null;
}

// set display order 
export async function setDisplayOrder(albumId, order) {
  try {
    const updatedAlbum = await invoke('set_album_display_order', { id: albumId, displayOrder: order });
    console.log('set_album_display_order', updatedAlbum);
    if (updatedAlbum) {
      return updatedAlbum;
    }
  } catch (error) {
    console.log('Failed to set display order:', error);
  }
  return null;
}

// set album cover
export async function setAlbumCover(albumId, fileId) {
  try {
    const result = await invoke('set_album_cover', { id: albumId, fileId });
    if (result) {
      return result;
    }
  } catch (error) {
    console.log('Failed to set album cover:', error);
  }
  return null;
}

// folders

// select a folder to an album
// add a folder to db
export async function selectFolder(albumId, folderPath) {
  try {
    const selectedFolder = await invoke('select_folder', { albumId, folderPath });
    if(selectedFolder) {
      return selectedFolder;
    };
  } catch (error) {
    console.log('Failed to select folder:', error);
  }
  return null;
}

// fetch folder and build a FileNode
export async function fetchFolder(path, isRecursive) {
  try {
    const folder = await invoke('fetch_folder', { path, isRecursive });
    if(folder) {
      // get folder children's favorite status
      for (let i = 0; i < folder.children.length; i++) {
        const child = folder.children[i];
        child.is_favorite = await getFolderFavorite(child.path);
      }
      console.log('fetchFolder:', folder);
      return folder;
    };
  } catch (error) {
    console.log('Failed to fetch folder:', error);
  }
  return null;
}

// expand the final folder path, return the final folder
export async function expandFinalFolder(rootFolder, finalPath) {
  let relativePath = finalPath.replace(rootFolder.path, '');

  const pathArray = relativePath.split(separator).filter(Boolean); // Split and remove empty strings
  
  // If there's no relative path, we're already at the target
  if (pathArray.length === 0) {
    return null;
  }
  
  // rootFolder.children is now [folderObject], start from that folderObject
  if (!rootFolder.children || rootFolder.children.length === 0) {
    return null;
  }
  
  // The first child is the root folder representation (e.g., folder1)
  let currentFolder = rootFolder.children[0];
  currentFolder.is_expanded = true;
  
  // Load its children if not already loaded
  if (!currentFolder.children) {
    const subFolders = await fetchFolder(currentFolder.path, false);
    if (subFolders) {
      currentFolder.children = subFolders.children;
    }
  }

  for (let i = 0; i < pathArray.length; i++) {
    if(currentFolder.children && currentFolder.children.length > 0) {
      for (let child of currentFolder.children) {
        if(child.name === pathArray[i]) {
          if( i < pathArray.length - 1) {
            child.is_expanded = true;
            // fetch sub-folders for this child
            const subFolders = await fetchFolder(child.path, false);
            if(subFolders) {
              child.children = subFolders.children;
            }
            currentFolder = child;
            break;
          } else {  // last folder
            return child;
          }
        }
      }
    }
  }
}

// recurse all files under the path(include all sub-folders), and count the number of files
export async function countFolder(path) {
  try {
    const result = await invoke('count_folder', { path });
    if(result) {
      return result;
    };
  } catch (error) {
    console.error('countFolder error:', error);
  }
  return null;
}

// create a folder
export async function createFolder(path, folderName) {
  try {
    const newFolder = await invoke('create_folder', { path, folderName });
    if(newFolder) {
      return newFolder;
    };
  } catch (error) {
    console.log('Failed to create folder:', error);
  }
  return null;
}

// rename a folder
export async function renameFolder(folderPath, newFolderName) {
  try {
    const renamedFolder = await invoke('rename_folder', { folderPath, newFolderName });
    if(renamedFolder) {
      return renamedFolder;
    };
  } catch (error) {
    console.log('Failed to rename folder:', error);
  }
  return null;
}

// move a folder, return new folder path
export async function moveFolder(folderPath, newAlbumId, newFolderPath) {
  try {
    const result = await invoke('move_folder', { folderPath, newAlbumId, newFolderPath });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to move folder:', error);
  }
  return null;
}

// copy a folder, return new folder path
export async function copyFolder(folderPath, newFolderPath) {
  try {
    const result = await invoke('copy_folder', { folderPath, newFolderPath });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to copy folder:', error);
  }
  return null;
}

// delete a folder
export async function deleteFolder(folderPath) {
  try {
    const result = await invoke('delete_folder', { folderPath });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to delete folder:', error);
  }
  return null;
};

/// reveal a folder in file explorer( or finder)
export async function revealFolder(folderPath) {
  try {
    const result = await invoke('reveal_folder', { folderPath });
    if(result) {
      return result;
    };
  } catch (error) {
    console.error('revealFolder error:', error);
  }
  return null;
};

// files

// get total files count and sum
export async function getTotalCountAndSum() {
  try {
    const result = await invoke('get_total_count_and_sum');
    if(result) {
      return result;
    };
  } catch (error) {
    console.error('getTotalCountAndSum error:', error);
  }
  return null;
}

/// get query files count and sum
export async function getQueryCountAndSum(params) {
  try {
    const result = await invoke('get_query_count_and_sum', { params });
    if(result) {
      return result;
    };
  } catch (error) {
    console.error('getQueryCountAndSum error:', error);
  }
  return null;
}

/// get query timeline markers
export async function getQueryTimeLine(params) {
  try {
    const result = await invoke('get_query_time_line', { params });
    if(result) {
      return result;
    };
  } catch (error) {
    console.error('getQueryTimeLine error:', error);
  }
  return null;
}

/// get query files from db (with pagination)
export async function getQueryFiles(params, offset, limit) {
  try {
    const files = await invoke('get_query_files', { 
      params,
      offset, 
      limit,
    });
    if(files) {
      return files;
    };
  } catch (error) {
    console.error('getQueryFiles error:', error);
  }
  return null;
}

// get all files from the folder (no pagination)
export async function getFolderFiles(folderId, folderPath, fromDbOnly) {
  try {
    let result = await invoke('get_folder_files', { 
      fileType: config.search.fileType,
      sortType: config.search.sortType,
      sortOrder: config.search.sortOrder,
      folderId, 
      folderPath,
      fromDbOnly: fromDbOnly || false,
    });
    if(result) {
      return result;
    };
  } catch (error) {
    console.error('getFolderFiles error:', error);
  }
  return [null, 0, 0];
};

// get the thumbnail count of the folder
export async function getFolderThumbCount(folderId) {
  try {
    let count = await invoke('get_folder_thumb_count', { 
      fileType: config.search.fileType,
      folderId, 
    });
    if(count) {
      return count;
    };
  } catch (error) {
    console.error('getFolderThumbCount error:', error);
  }
  return 0;
}

// edit an image
// params: { sourceFilePath, destFilePath, outputFormat, orientation, flipHorizontal, flipVertical, rotate, crop, resize, quality, filter, brightness, contrast, blur }
export async function editImage(params) {
  try {
    return await invoke('edit_image', { params });
  } catch (error) {
    console.log('Failed to edit image:', error);
    return false;
  }
}

// copy an edited image to clipboard
export async function copyEditedImage(params) {
  try {
    return await invoke('copy_edited_image', { params });
  } catch (error) {
    console.log('Failed to copy edited image to clipboard:', error);
    return false;
  }
}

// copy an image to clipboard
export async function copyImage(filePath) {
  try {
    return await invoke('copy_image', { filePath });
  } catch (error) {
    console.error('Failed to copy image to clipboard:', error);
    return false;
  }
}

// rename a file
export async function renameFile(fileId, filePath, newName) {
  try {
    const result = await invoke('rename_file', { fileId, filePath, newName });
    if (result) {
      return result;
    }
  } catch (error) {
    console.log('Failed to rename file:', error);
  }
  return null;
}

// move a file
export async function moveFile(fileId, filePath, newFolderId, newFolderPath) {
  try {
    const result = await invoke('move_file', { fileId, filePath, newFolderId, newFolderPath });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to move file:', error);
  }
  return null;
}

// copy a file
export async function copyFile(filePath, newFolderPath) {
  try {
    const result = await invoke('copy_file', { filePath, newFolderPath });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to copy file:', error);
  }
  return null;
}

// delete a file
export async function deleteFile(fileId, filePath) {
  try {
    return await invoke('delete_file', { fileId, filePath });
  } catch (error) {
    console.error('deleteFile error:', error);
    return null;
  }
}

// delete a file from db
export async function deleteDbFile(fileId) {
  try {
    return await invoke('delete_db_file', { fileId });
  } catch (error) {
    console.error('deleteDbFile error:', error);
  }
}

// edit file comment
export async function editFileComment(fileId, comment) {
  try {
    const result = await invoke('edit_file_comment', { fileId, comment });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to edit file comment:', error);
  }
  return null;
}

// get file thumb
export async function getFileThumb(fileId, filePath, fileType, orientation, thumbnailSize, forceRegenerate) {
  try {
    const result = await invoke('get_file_thumb', { fileId, filePath, fileType, orientation, thumbnailSize, forceRegenerate });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to get file thumb:', error);
  }
  return null;
}

// get file info
export async function getFileInfo(fileId) {
  try {
    const result = await invoke('get_file_info', { fileId });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to get file info:', error);
  }
  return null;
}

// update file info
export async function updateFileInfo(fileId, filePath) {
  try {
    const result = await invoke("update_file_info", { fileId, filePath });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to update file info:', error);
  }
  return null;
}

// get file image
export async function getFileImage(filePath) {
  try {
    const result = await invoke('get_file_image', { filePath });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to get file image:', error);
  }
  return null;
}

// check if file exists
export async function checkFileExists(filePath) {
  try {
    return await invoke('check_file_exists', { filePath });
  } catch (error) {
    console.error('Failed to check file exists:', error);
  }
  return false;
}

// set file rotate
export async function setFileRotate(fileId, fileRotate) {
  try {
    const result = await invoke('set_file_rotate', { fileId, rotate: fileRotate % 360 });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to set file rotate:', error);
  }
  return null;
}

// get file has_tags status
export async function getFileHasTags(fileId) {
  try {
    const result = await invoke('get_file_has_tags', { fileId });
    if (result) {
      return result;
    }
  } catch (error) {
    console.error('Failed to get file has_tags status:', error);
  }
  return false;
}

// favorites

// get favorite folders
export async function getFavoriteFolders() {
  try {
    const favoriteFolders = await invoke('get_favorite_folders');
    if (favoriteFolders) {
      // sort favorite folders by name in locale order 
      favoriteFolders.sort((a, b) => localeComp(config.settings.language, a.name, b.name));
      return favoriteFolders;
    }
  } catch (error) {
    console.error('Failed to get favorite folders:', error);
  }
  return null;
}

// get folder favorite
export async function getFolderFavorite(folderPath) {
  try {
    const is_favorite = await invoke('get_folder_favorite', { folderPath });
    if(is_favorite) {
      return is_favorite;
    };
  } catch (error) {
    console.log('Failed to get folder favorite:', error);
  }
  return false;
}

// set folder favorite
export async function setFolderFavorite(folderId, isFavorite) {
  try {
    const result = await invoke('set_folder_favorite', { folderId, isFavorite });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to set folder favorite:', error);
  }
  return null;
}

// set file favorite
export async function setFileFavorite(fileId, isFavorite) {
  try {
    const result = await invoke('set_file_favorite', { fileId, isFavorite });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to set file favorite:', error);
  }
  return null;
}

// tags

// get all tags
export async function getAllTags() {
  try {
    const tags = await invoke('get_all_tags');
    console.log('getAllTags:', tags);
    if (tags) {
      return tags;
    }
  } catch (error) {
    console.error('Failed to get all tags:', error);
  }
  return null;
}

// get tag name by id
export async function getTagName(tagId) {
  try {
    const tagName = await invoke('get_tag_name', { tagId });
    if (tagName) {
      return tagName;
    }
  } catch (error) {
    console.error('Failed to get tag name:', error);
  }
  return null;
}

// create a new tag
export async function createTag(name) {
  try {
    const result = await invoke('create_tag', { name });
    return result;
  } catch (error) {
    console.error('Failed to create tag:', error);
  }
  return null;
}

// rename a tag
export async function renameTag(tagId, newName) {
  try {
    const result = await invoke('rename_tag', { tagId, newName });
    return result;
  } catch (error) {
    console.error('Failed to rename tag:', error);
  }
  return null;
}

// delete a tag
export async function deleteTag(tagId) {
  try {
    const result = await invoke('delete_tag', { tagId });
    return result;
  } catch (error) {
    console.error('Failed to delete tag:', error);
  }
  return null;
}

// get tags for a file
export async function getTagsForFile(fileId) {
  try {
    const tags = await invoke('get_tags_for_file', { fileId });
    if (tags) {
      return tags;
    }
  } catch (error) {
    console.error('Failed to get tags for file:', error);
  }
  return null;
}

// add tag to file
export async function addTagToFile(fileId, tagId) {
  try {
    const result = await invoke('add_tag_to_file', { fileId, tagId });
    return result;
  } catch (error) {
    console.error('Failed to add tag to file:', error);
  }
  return null;
}

// remove tag from file
export async function removeTagFromFile(fileId, tagId) {
  try {
    const result = await invoke('remove_tag_from_file', { fileId, tagId });
    return result;
  } catch (error) {
    console.error('Failed to remove tag from file:', error);
  }
  return null;
}

// calendar

// get taken dates
export async function getTakenDates(ascending = true) {
  try {
    const taken_dates = await invoke('get_taken_dates', { ascending });
    if (taken_dates) {
      return taken_dates;
    }
  } catch (error) {
    console.error('Failed to get taken dates:', error);
  }
  return null;
}

// camera

// get camera info
export async function getCameraInfo() {
  try {
    const cameraInfo = await invoke('get_camera_info');
    if (cameraInfo) {
      return cameraInfo;
    }
  } catch (error) {
    console.error('Failed to get camera info:', error);
  }
  return null;
}

// location

// get location info
export async function getLocationInfo() {
  try {
    const locationInfo = await invoke('get_location_info');
    if (locationInfo) {
      return locationInfo;
    }
  } catch (error) {
    console.error('Failed to get location info:', error);
  }
  return null;
}

// print

// print image
export async function printImage(imagePath) {
  try {
    const result = await invoke('print_image', { imagePath });
    if (result) {
      return result;
    }
  } catch (error) {
    console.error('Failed to print image:', error);
  }
  return null;
}

// settings

// get package info
export async function getPackageInfo() {
  try {
    const packageInfo = await invoke('get_package_info');
    if (packageInfo) {
      return packageInfo;
    }
  } catch (error) {
    console.error('Failed to get package info:', error);
  }
  return null;
}

// get build time
export async function getBuildTime() {
  try {
    const unixTime = await invoke('get_build_time');
    console.log('get_build_time', unixTime);
    if (unixTime) {
      return new Date(unixTime * 1000).toLocaleString();;
    }
  } catch (error) {
    console.error('Failed to get build time:', error);
  }
  return null;
}

// get db file info
export async function getStorageFileInfo() {
  try {
    const dbFileInfo = await invoke('get_storage_file_info');
    if (dbFileInfo) {
      return dbFileInfo;
    }
  } catch (error) {
    console.error('Failed to get db file size:', error);
  }
  return null;
}

// ai

// check ai status
export async function checkAiStatus() {
  try {
    const status = await invoke('check_ai_status');
    return status;
  } catch (error) {
    console.error('checkAiStatus error:', error);
  }
  return 'Unknown';
}

// generate embedding
export async function generateEmbedding(fileId) {
  try {
    const result = await invoke('generate_embedding', { fileId });
    return result;
  } catch (error) {
    console.error('generateEmbedding error:', error);
  }
  return null;
}

// search similar images
export async function searchSimilarImages(params) {
  try {
    const results = await invoke('search_similar_images', { params });
    if (results) {
      return results;
    }
  } catch (error) {
    console.error('searchSimilarImages error:', error);
  }
  return [];
}

// indexing

// index album
export async function indexAlbum(albumId) {
  try {
    await invoke('index_album', { albumId, thumbnailSize: config.settings.thumbnailSize || 512 });
  } catch (error) {
    console.error('indexAlbum error:', error);
  }
}

// cancel indexing
export async function cancelIndexing(albumId) {
  try {
    await invoke('cancel_indexing', { albumId });
  } catch (error) {
    console.error('cancelIndexing error:', error);
  }
}

// listen index progress
export async function listenIndexProgress(callback) {
  return await listen('index_progress', callback);
}

// listen index finished
export async function listenIndexFinished(callback) {
  return await listen('index_finished', callback);
}
