import { invoke } from '@tauri-apps/api/core';
import { useConfigStore } from '@/stores/configStore';
import { separator, openFolderDialog, localeComp } from '@/common/utils';

const config = useConfigStore();

// album

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
      return albums;
    } 
  } catch (error) {
    console.error('getAlbums...', error);
  }
  return null;
};

// get one album
export async function getAlbum(albumId) {
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

// add an album
export async function addAlbum() {
  try {
    const folderPath = await openFolderDialog();
    if (folderPath) {
      const newAlbum = await invoke('add_album', { folderPath });
      console.log('add_album', newAlbum);
      if(newAlbum) {
        return {
          ...newAlbum,
          is_expanded: false,
          children: null,
        };
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

// folder

// select a folder
export async function selectFolder(albumId, parentId, folderPath) {
  try {
    const selectedFolder = await invoke('select_folder', { albumId, parentId, folderPath });
    if(selectedFolder) {
      return selectedFolder;
    };
  } catch (error) {
    console.log('Failed to select folder:', error);
  }
  return null;
}

// fetch folder
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
  let currentFolder = rootFolder;

  for (let i = 0; i < pathArray.length; i++) {
    // fetch sub-folders
    const subFolders = await fetchFolder(currentFolder.path, false);
    if(subFolders) {
      currentFolder.children = subFolders.children;

      if(currentFolder.children && currentFolder.children.length > 0) {
        for (let child of currentFolder.children) {
          if(child.name === pathArray[i]) {
            if( i < pathArray.length - 1) {
              child.is_expanded = true;
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
}

// recurse all files under the path, and count the number of files
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
    if (result) {
      return true;
    };
  } catch (error) {
    console.log('Failed to delete folder:', error);
  }
  return false;
}

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

// file

/// get all files from db (with pagination)
/// return [files, totalCount, totalSum] when offset is 0
/// return files when offset is not 0
export async function getDbFiles(startDate, endDate, make, model, isFavorite, tagId, isDeleted, offset) {
  try {
    const files = await invoke('get_db_files', {
      searchText: config.searchText, 
      searchFileType: config.searchFileType,
      sortType: config.sortType,
      sortOrder: config.sortOrder,
      startDate, 
      endDate,
      make, 
      model,
      isFavorite, 
      tagId,
      isDeleted,
      offset, 
      pageSize: config.fileListPageSize
    });
    if(files) {
      if(offset === 0) {
        const [totalCount, totalSum] = await getDbCountAndSum(startDate, endDate, make, model, isFavorite, tagId, isDeleted);
        return [files, totalCount, totalSum];
      } else {
        return files;
      }
    };
  } catch (error) {
    console.error('getAllFiles error:', error);
  }
  return null;
}

/// get all db files' count and sum(without pagination)
export async function getDbCountAndSum(startDate, endDate, make, model, isFavorite, tagId, isDeleted) {
  try {
    const result = await invoke('get_db_count_and_sum', {
      searchText: config.searchText, 
      searchFileType: config.searchFileType,
      startDate, 
      endDate,
      make, 
      model,
      isFavorite, 
      tagId,
      isDeleted,
    });
    if(result) {
      return result;
    };
  } catch (error) {
    console.error('getDbCountAndSum error:', error);
  }
  return null
}

// get all files from the folder (no pagination)
// return [files, totalCount, totalSum]
export async function getFolderFiles(folderId, folderPath) {
  try {
    const files = await invoke('get_folder_files', { 
      searchText: config.searchText, 
      searchFileType: config.searchFileType,
      sortType: config.sortType,
      sortOrder: config.sortOrder,
      folderId, 
      folderPath, 
    });
    if(files) {
      const totalCount = files.length;
      const totalSum = files.reduce((acc, file) => acc + file.size, 0);
      return [files, totalCount, totalSum];
    };
  } catch (error) {
    console.error('getFolderFiles error:', error);
  }
  return [null, null, null];
};

// copy an image to clipboard
export async function copyImage(filePath) {
  try {
    await invoke('copy_image_to_clipboard', { filePath });
  } catch (error) {
    console.error('copyImageToClipboard error:', error);
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
    const result = await invoke('delete_file', { fileId, filePath });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to delete file:', error);
  }
  return null;
};

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
export async function getFileThumb(fileId, filePath, fileType, orientation, thumbnailSize) {
  try {
    const result = await invoke('get_file_thumb', { fileId, filePath, fileType, orientation, thumbnailSize });
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

// set file delete
export async function setFileDelete(fileId, deletedAt) {
  try {
    const result = await invoke('set_file_delete', { fileId, deletedAt });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to set file delete:', error);
  }
  return null;
}

// favorite

// get favorite folders
export async function getFavoriteFolders() {
  try {
    const favoriteFolders = await invoke('get_favorite_folders');
    if (favoriteFolders) {
      // sort favorite folders by name in locale order 
      favoriteFolders.sort((a, b) => localeComp(config.language, a.name, b.name));
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

// tag

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

// setting

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