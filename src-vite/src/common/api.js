import { invoke } from '@tauri-apps/api/core';
import { openFolderDialog, openShellFolder } from '@/common/utils';

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

// add folder
export async function addFolder(albumId, parentId, folderPath) {
  try {
    const newFolder = await invoke('add_folder', { albumId, parentId, folderPath });
    console.log('add_folder', newFolder);
    if(newFolder) {
      return newFolder;
    };
  } catch (error) {
    console.log('Failed to add folder:', error);
  }
  return null;
}

// expand folder
export async function expandFolder(path, isRecursive) {
  try {
    const subFolders = await invoke('expand_folder', { path, isRecursive });
    console.log('expand_folder', subFolders);
    if(subFolders) {
      return subFolders;
    };
  } catch (error) {
    console.log('Failed to expand folder:', error);
  }
  return null;
}

// get taken dates
export async function getTakenDates() {
  try {
    const taken_dates = await invoke('get_taken_dates');
    console.log('get_taken_dates, taken_dates');
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
    console.log('get_camera_info', cameraInfo);
    if (cameraInfo) {
      return cameraInfo;
    }
  } catch (error) {
    console.error('Failed to get camera info:', error);
  }
  return null;
}