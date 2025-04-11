import { invoke } from '@tauri-apps/api/core';
import { useConfigStore } from '@/stores/configStore';
import { openFolderDialog, localeComp } from '@/common/utils';

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

// get album
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

// expand folder
export async function expandFolder(path, isRecursive) {
  try {
    const folder = await invoke('expand_folder', { path, isRecursive });
    if(folder) {
      // get folder children's favorite status
      for (let i = 0; i < folder.children.length; i++) {
        const child = folder.children[i];
        child.is_favorite = await getFolderFavorite(child.path);
      }
      // sort subfolders by name in locale order
      folder.children.sort((a, b) => localeComp(config.language, a.name, b.name));
      console.log('expandFolder:', folder);
      return folder;
    };
  } catch (error) {
    console.log('Failed to expand folder:', error);
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