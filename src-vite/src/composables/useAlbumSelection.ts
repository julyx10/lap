import { provide, inject, computed, reactive, Ref } from 'vue';
import { libConfig } from '@/common/config';
import { useUIStore } from '@/stores/uiStore';
import { selectFolder as apiSelectFolder } from '@/common/api';
import { Album, Folder, AlbumSelectionContext, ALBUM_SELECTION_KEY } from '@/common/types';

export type SelectionSource = 'album' | 'destFolder' | 'local';

const folderFileCounts = reactive(new Map<string, number>());

export const setFolderFileCount = (path: string, count: number) => {
    folderFileCounts.set(path, count);
};

export const clearFolderFileCounts = () => {
    folderFileCounts.clear();
};

export const getFolderFileCount = (path: string) => folderFileCounts.get(path) || 0;

export const getCountedFolderPaths = () => Array.from(folderFileCounts.keys());

/**
 * Creates and provides an album selection context.
 * Call this in the root component (AlbumList) to provide selection state to all descendants.
 * 
 * @param source - 'album' for the main view, 'destFolder' for MoveTo, or 'local' for an independent picker
 * @param onExpandAndSelect - callback to expand album and select folder (implemented by AlbumList)
 */
export function useAlbumSelectionProvider(
    source: SelectionSource,
    onExpandAndSelect?: (albumId: number, folderPath: string) => Promise<void>
) {
    const uiStore = useUIStore();
    const markAlbumActivated = () => {
        if (source !== 'album') return;
        libConfig.activePane = 'main';
        libConfig.album.activateTick = Number(libConfig.album.activateTick || 0) + 1;
    };

    const localSelection = reactive({
        albumId: 0,
        folderId: null as number | null,
        folderPath: '',
        selected: false,
    });
    const folderSelection = computed(() => source === 'local' ? localSelection : libConfig.destFolder);

    // Create refs for the selected source.
    const albumId = computed({
        get: () => source === 'album' ? (libConfig.album.id ?? 0) : (folderSelection.value.albumId ?? 0),
        set: (val: number) => {
            if (source === 'album') {
                libConfig.album.id = val;
            } else {
                folderSelection.value.albumId = val;
            }
        }
    });

    const folderId = computed({
        get: () => source === 'album' ? libConfig.album.folderId : folderSelection.value.folderId,
        set: (val: number | null) => {
            if (source === 'album') {
                libConfig.album.folderId = val;
            } else {
                folderSelection.value.folderId = val;
            }
        }
    });

    const folderPath = computed({
        get: () => source === 'album' ? (libConfig.album.folderPath ?? '') : (folderSelection.value.folderPath ?? ''),
        set: (val: string) => {
            if (source === 'album') {
                libConfig.album.folderPath = val;
            } else {
                folderSelection.value.folderPath = val;
            }
        }
    });

    const selected = computed({
        get: () => source === 'album' ? (libConfig.album.selected ?? false) : (folderSelection.value.selected ?? false),
        set: (val: boolean) => {
            if (source === 'album') {
                libConfig.album.selected = val;
            } else {
                folderSelection.value.selected = val;
            }
        }
    });

    let folderRequestId = 0;
    const currentSelection = () => ({
        albumId: albumId.value,
        folderId: folderId.value,
        folderPath: folderPath.value,
        selected: selected.value,
    });
    let confirmedSelection = currentSelection();

    const updateFolderPath = (folder: Folder, oldPath: string, newPath: string) => {
        if (folder.path === oldPath) {
            folder.path = newPath;
            folder.name = newPath.split(/[\\/]/).filter(Boolean).pop() || folder.name;
        } else if (folder.path.startsWith(`${oldPath}${oldPath.includes('\\') ? '\\' : '/'}`)) {
            folder.path = `${newPath}${folder.path.slice(oldPath.length)}`;
        }
        for (const child of folder.children || []) {
            updateFolderPath(child, oldPath, newPath);
        }
    };

    /**
     * Select an album (shows all files in the album)
     */
    const selectAlbum = (album: Album) => {
        folderRequestId++;
        albumId.value = album.id;
        folderPath.value = album.path;
        selected.value = true;
        confirmedSelection = currentSelection();
        if (source === 'local') return;
        uiStore.requestCountUpdate({ source: 'album', id: Number(album.id || 0) });
        markAlbumActivated();
    };

    /**
     * Select a folder within an album
     */
    const selectFolder = async (albumIdVal: number, folder: Folder) => {
        markAlbumActivated();
        const requestId = ++folderRequestId;
        const selectedPath = folder.path;
        const isRestoringCurrentFolder =
            albumId.value === albumIdVal &&
            folderPath.value === selectedPath &&
            !selected.value;
        albumId.value = albumIdVal;
        // Folder-tree nodes are filesystem nodes and do not carry the database
        // folder id. When AlbumList remounts and restores the already selected
        // folder, keep its known id instead of changing it to null and then back
        // again after select_folder resolves.
        if (!isRestoringCurrentFolder) {
            folderId.value = Number(folder.id || 0) || null;
        }
        folderPath.value = selectedPath;
        selected.value = false;
        if (source === 'local') return;
        uiStore.requestCountUpdate({ source: 'album-folder', path: selectedPath });

        await new Promise<void>(resolve => {
            requestAnimationFrame(() => setTimeout(resolve, 0));
        });
        const result = await apiSelectFolder(albumIdVal, selectedPath);
        const selectionIsCurrent = requestId === folderRequestId;

        if (result && selectionIsCurrent) {
            folderId.value = result.id;
            folderPath.value = result.path;
            if (result.path !== selectedPath) {
                updateFolderPath(folder, selectedPath, result.path);
            }
            confirmedSelection = currentSelection();
        } else if (!result && selectionIsCurrent) {
            albumId.value = confirmedSelection.albumId;
            folderId.value = confirmedSelection.folderId;
            folderPath.value = confirmedSelection.folderPath;
            selected.value = confirmedSelection.selected;
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
        folderRequestId++;
        albumId.value = 0;
        folderId.value = null;
        folderPath.value = '';
        selected.value = false;
        confirmedSelection = currentSelection();
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
