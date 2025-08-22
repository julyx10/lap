# Changelog

## 2025-08-19: Refactor Trash and Deletion Functionality

**Objective:** The primary goal of this update was to refactor the item deletion functionality. The previous system, which used a `deleted_at` flag (a "soft delete"), was replaced with a more robust "virtual trash folder" model. This new approach treats the trash as a special album, and "deleting" an item is now a `move` operation. This resolves inconsistencies, especially with the app's hybrid model of reading from both the filesystem and the database, and provides a more stable foundation for future features like restoring items.

---

### Step 1: Database Schema Redesign

*   **File Updated:** `src-tauri/src/t_sqlite.rs`
*   **Changes:**
    *   Modified the `CREATE TABLE` statement for the `afolders` table:
        *   Removed the `parent_id` column as it was identified as unused.
        *   Removed the `deleted_at` column.
        *   Added `original_album_id INTEGER` to store the folder's original album ID when it's moved to the trash.
        *   Added `name_in_trash TEXT` to handle potential naming conflicts inside the trash.
        *   Added `trashed_at INTEGER` to timestamp when the folder was deleted.
    *   Modified the `CREATE TABLE` statement for the `afiles` table:
        *   Removed the `deleted_at` column.
        *   Added `original_folder_id INTEGER` to store the file's original folder ID.
        *   Added `name_in_trash TEXT` for naming conflicts in the trash.
        *   Added `trashed_at INTEGER` for the deletion timestamp.
    *   Updated the corresponding Rust structs (`AFolder`, `AFile`) and their associated functions (`from_row`, `insert`, `get_files`, etc.) to reflect these schema changes.
*   **Comment:** This new schema provides a clear and robust way to track an item's state and original location once it has been moved to the trash, which is essential for the restore functionality.

---

### Step 2: Backend Logic Implementation

*   **Files Updated:** `src-tauri/src/t_cmds.rs`, `src-tauri/src/t_sqlite.rs`, `src-tauri/src/t_utils.rs`
*   **Changes:**
    *   In `t_utils.rs`, added the `get_trash_path()` function to define a consistent physical location for the trash folder.
    *   In `t_sqlite.rs`, added the `ensure_trash_album_exists()` function and called it from `create_db()` to automatically create the "Trash" album and its corresponding folder on startup if they don't exist.
    *   In `t_cmds.rs`, removed the old and now obsolete `set_file_delete` and `set_folder_delete` commands.
    *   In `t_cmds.rs`, introduced and implemented two new primary commands:
        *   `trash_items(file_ids, folder_ids)`: Handles moving files and folders from their original location to the trash. This includes updating their paths and album/folder IDs in the database and recording their original location.
        *   `restore_items(file_ids, folder_ids)`: Handles the reverse operation, moving items from the trash back to their original locations and clearing the trash-related fields in the database.
    *   Corrected the `select_folder` command to remove the unused `parent_id` parameter.
*   **Comment:** This centralizes the core logic for the trash functionality into a few clear commands, ensuring that filesystem operations and database records remain synchronized.

---

### Step 3: Frontend API Integration

*   **File Updated:** `src-vite/src/common/api.js`
*   **Changes:**
    *   Removed the old `setFileDelete` and `setFolderDelete` functions.
    *   Added and exported new `trashItems` and `restoreItems` functions that invoke the new backend commands.
    *   Updated the `getDbFiles` function to use an `isTrashed` parameter instead of `isDeleted` to align with the new backend logic.
    *   Removed the client-side `deleted_at` filter from `getFolderFiles` as this logic is now handled by the physical move of the file.
    *   Updated `selectFolder` to remove the `parentId` parameter.
*   **Comment:** These changes ensure the frontend UI components communicate with the new, correct backend commands.

---

### Step 4: User Interface Updates

*   **Files Updated:** `src-vite/src/components/Trash.vue`, `src-vite/src/components/Content.vue`
*   **Changes:**
    *   In `Trash.vue`, the context menu actions for items were updated. The "Restore" option now correctly calls `restoreItems`.
    *   In `Content.vue`, the `clickMoveToTrash` function was refactored to call the new `trashItems` function, and the obsolete `clickRestoreFromTrash` function was removed.
    *   The logic for displaying content when the trash is selected (`sidebarIndex === 7`) was simplified to use the new `isTrashed` flag.
*   **Comment:** These updates connect user actions directly to the new, robust trash functionality.

---

### Step 5: General Code Refinement

*   **File Updated:** `src-vite/src/components/Content.vue`
*   **Changes:**
    *   Refactored the title bar component to correctly handle text truncation with an ellipsis (`...`) when the title text overflows.
    *   The previous implementation using a `v-for` loop was replaced with a simpler and more direct approach using Tailwind CSS's `truncate` utility.
*   **Comment:** This was a separate code quality improvement to fix a UI bug and enhance maintainability.
