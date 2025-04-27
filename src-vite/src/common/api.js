import { invoke } from '@tauri-apps/api/core';
import { useConfigStore } from '@/stores/configStore';
import { separator, openFolderDialog, localeComp } from '@/common/utils';
import { format } from 'date-fns';

const config = useConfigStore();

/// get all albums
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

/// add an album
export async function addAlbum() {
  try {
    const folderPath = await openFolderDialog();
    if (folderPath) {
      const newAlbum = await invoke('add_album', { folderPath: folderPath });
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

// rename an album
export async function renameAlbum(albumId, newName) {
  try {
    const renamedAlbum = await invoke('rename_album', { id: albumId, name: newName });
    console.log('rename_album', renamedAlbum);
    if (renamedAlbum) {
      return renamedAlbum;
    }
  } catch (error) {
    console.log('Failed to rename album:', error);
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
      // sort subfolders by name in locale order
      folder.children.sort((a, b) => localeComp(config.language, a.name, b.name));
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

// move files, return moved files
export async function moveFiles(files, newFolderPath) {
  try {
    const result = await invoke('move_files', { files, newFolderPath });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to move files:', error);
  }
  return null;
}

// copy files, return copied files
export async function copyFiles(files, newFolderPath) {
  try {
    const result = await invoke('copy_files', { files, newFolderPath });
    if(result) {
      return result;
    };
  } catch (error) {
    console.log('Failed to copy files:', error);
  }
  return null;
}

/// get all files under the path
export async function getFolderFiles(folderId, folderPath) {
  try {
    const result = await invoke('get_folder_files', { folderId: folderId, path: folderPath });
    if(result) {
      return result;
    };
  } catch (error) {
    console.error('getFolderFiles error:', error);
  }
  return null;
};

/// get all files (only get favorite files if isFavorite is true)
export async function getAllFiles(isFavorite = false, offset = 0, pageSize = config.fileListPageSize) {
  try {
    const result = await invoke('get_all_files', { isFavorite, offset, pageSize });
    if(result) {
      return result;
    };
  } catch (error) {
    console.error('getAllFiles error:', error);
  }
  return null
}

/// get all files of calendar
export async function getCalendarFiles(year, month, date) {
  try {
    if (date === -1) { // -1 means selecting a month
      // get the first and last days of the month.
      let startDate = format(new Date(year, month - 1, 1), 'yyyy-MM-dd');
      let endDate = format(new Date(year, month, 0), 'yyyy-MM-dd');
      const result = await invoke('get_files_by_date_range', { startDate, endDate });
      if(result) {
        return result;
      };
    } else {  // otherwise, get files by date
      let dateStr = format(new Date(year, month - 1, date), 'yyyy-MM-dd');
      const result = await invoke('get_files_by_date', { date: dateStr });
      if(result) {
        return result;
      };
    }
  } catch (error) {
    console.error('getCalendarFiles error:', error);
  }
  return null;
}

/// get all files under the camera make and model
export async function getCameraFiles(make, model) {
  try {
    const result = await invoke('get_camera_files', { make, model });
    if(result) {
      return result;
    };
  } catch (error) {
    console.error('getCameraFiles error:', error);
  }
  return null
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

// get taken dates
export async function getTakenDates() {
  try {
    const taken_dates = await invoke('get_taken_dates');
    if (taken_dates) {
      return taken_dates;
    }
  } catch (error) {
    console.error('Failed to get taken dates:', error);
  }
  return null;
}

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