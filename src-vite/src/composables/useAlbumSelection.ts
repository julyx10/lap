import { provide, inject, computed, Ref } from 'vue';
import { libConfig } from '@/common/config';
import { selectFolder as apiSelectFolder } from '@/common/api';
import { Album, Folder, AlbumSelectionContext, ALBUM_SELECTION_KEY } from '@/common/types';

export type SelectionSource = 'album' | 'destFolder';

/**
 * Creates and provides an album selection context.
 * Call this in the root component (AlbumList) to provide selection state to all descendants.
 * 
 * @param source - 'album' for main album view, 'destFolder' for MoveTo dialog
 * @param onExpandAndSelect - callback to expand album and select folder (implemented by AlbumList)
 */
export function useAlbumSelectionProvider(
    source: SelectionSource,
    onExpandAndSelect?: (albumId: number, folderPath: string) => Promise<void>
) {
    // Create refs that stay in sync with libConfig
    const albumId = computed({
        get: () => source === 'album' ? (libConfig.album.id ?? 0) : (libConfig.destFolder.albumId ?? 0),
        set: (val: number) => {
            if (source === 'album') {
                libConfig.album.id = val;
            } else {
                libConfig.destFolder.albumId = val;
            }
        }
    });

    const folderId = computed({
        get: () => source === 'album' ? libConfig.album.folderId : libConfig.destFolder.folderId,
        set: (val: number | null) => {
            if (source === 'album') {
                libConfig.album.folderId = val;
            } else {
                libConfig.destFolder.folderId = val;
            }
        }
    });

    const folderPath = computed({
        get: () => source === 'album' ? (libConfig.album.folderPath ?? '') : (libConfig.destFolder.folderPath ?? ''),
        set: (val: string) => {
            if (source === 'album') {
                libConfig.album.folderPath = val;
            } else {
                libConfig.destFolder.folderPath = val;
            }
        }
    });

    const selected = computed({
        get: () => source === 'album' ? (libConfig.album.selected ?? false) : (libConfig.destFolder.selected ?? false),
        set: (val: boolean) => {
            if (source === 'album') {
                libConfig.album.selected = val;
            } else {
                libConfig.destFolder.selected = val;
            }
        }
    });

    /**
     * Select an album (shows all files in the album)
     */
    const selectAlbum = (album: Album) => {
        albumId.value = album.id;
        folderPath.value = album.path;
        selected.value = true;
    };

    /**
     * Select a folder within an album
     */
    const selectFolder = async (albumIdVal: number, folder: Folder) => {
        const result = await apiSelectFolder(albumIdVal, folder.path);
        if (result) {
            albumId.value = albumIdVal;
            folderId.value = result.id;
            folderPath.value = result.path;
            selected.value = false;
        }
    };

    /**
     * Expand the album tree to a specific folder and select it
     * Used after folder move operations
     */
    const expandAndSelectFolder = async (albumIdVal: number, targetFolderPath: string) => {
        if (onExpandAndSelect) {
            await onExpandAndSelect(albumIdVal, targetFolderPath);
        }
    };

    /**
     * Reset selection to show all files
     */
    const resetSelection = () => {
        albumId.value = 0;
        folderId.value = null;
        folderPath.value = '';
        selected.value = false;
    };

    // Create the context object
    const context: AlbumSelectionContext = {
        albumId: albumId as unknown as Ref<number>,
        folderId: folderId as unknown as Ref<number | null>,
        folderPath: folderPath as unknown as Ref<string>,
        selected: selected as unknown as Ref<boolean>,
        selectAlbum,
        selectFolder,
        expandAndSelectFolder,
        resetSelection,
    };

    // Provide the context to all descendants
    provide(ALBUM_SELECTION_KEY, context);

    return context;
}

/**
 * Injects the album selection context.
 * Call this in child components (AlbumFolder) to access the selection state.
 */
export function useAlbumSelection(): AlbumSelectionContext {
    const context = inject<AlbumSelectionContext>(ALBUM_SELECTION_KEY);
    if (!context) {
        throw new Error('useAlbumSelection must be used within a component that provides AlbumSelectionContext');
    }
    return context;
}
