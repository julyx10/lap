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
    total?: number;
    indexed?: number;
    created_at?: number;
    modified_at?: number;
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

/**
 * Represents a face bounding box
 */
export interface BBox {
    x: number;
    y: number;
    width: number;
    height: number;
}

/**
 * Represents a face record from the API/DB (raw data)
 */
export interface RawFace {
    id: number;
    file_id: number;
    person_id: number;
    person_name?: string;
    bbox: string; // JSON string
    created_at?: number;
    modified_at?: number;
}

/**
 * Represents a face record with parsed bounding box
 */
export interface Face extends Omit<RawFace, 'bbox'> {
    bbox: BBox;
}

