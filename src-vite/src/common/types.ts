import { Ref } from 'vue';

/**
 * Represents a folder in an album's folder tree
 */
export interface Folder {
    id: number;
    name: string;
    path: string;
    is_expanded?: boolean;
    is_favorite?: boolean;
    children?: Folder[];
}

/**
 * Represents an album with its folder hierarchy
 */
export interface Album {
    id: number;
    name: string;
    path: string;
    description?: string;
    cover_file_id?: number;
    is_expanded?: boolean;
    is_favorite?: boolean;
    is_hidden?: boolean;
    total?: number;
    scanned?: number;
    created_at?: string;
    modified_at?: string;
    children?: Folder[];
}

/**
 * Selection context provided to album/folder components via inject
 */
export interface AlbumSelectionContext {
    // Current selection state
    albumId: Ref<number>;
    folderId: Ref<number | null>;
    folderPath: Ref<string>;
    selected: Ref<boolean>;  // true = album selected, false = folder selected

    // Selection actions
    selectAlbum: (album: Album) => void;
    selectFolder: (albumId: number, folder: Folder) => Promise<void>;

    // For navigating to a specific folder path (e.g., after folder move)
    expandAndSelectFolder: (albumId: number, folderPath: string) => Promise<void>;

    // For resetting selection (e.g., show all files)
    resetSelection: () => void;
}

/**
 * Injection key for album selection context
 */
export const ALBUM_SELECTION_KEY = Symbol('albumSelection');
