import { computed, Ref } from 'vue';
import {
  IconMonitor,
  IconImageEdit,
  IconFavorite,
  IconUnFavorite,
  IconTag,
  IconRotate,
  IconCopy,
  IconRename,
  IconMoveTo,
  IconCopyTo,
  IconTrash,
  IconGoto,
  IconComment,
  IconImageSearch,
  IconPhotoSearch,
  IconFiles,
  IconFolderSearch,
  IconPersonSearch,
} from '@/common/icons';

export const useFileMenuItems = (
  file: Ref<any>,
  localeMsg: Ref<any>,
  isMac: boolean,
  showFolderFiles: Ref<boolean>,
  onAction: (action: string) => void
) => {
  return computed(() => {
    const f = file.value;
    if (!f) return [];

    const createAction = (actionName: string) => () => onAction(actionName);

    return [
      {
        label: localeMsg.value.menu.file.view_in_new_window,
        icon: IconMonitor,
        shortcut: isMac ? '⌘⏎' : 'Ctrl+Enter',
        action: createAction('open')
      },
      {
        label: localeMsg.value.menu.file.find_similar_images,
        icon: IconPhotoSearch,
        shortcut: isMac ? '⌘S' : 'Ctrl+S',
        disabled: f.file_type !== 1 && f.file_type !== 3,
        action: createAction('search-similar')
      },
      {
        label: localeMsg.value.menu.file.find_person_images,
        icon: IconPersonSearch,
        action: createAction('find-person')
      },
      {
        label: localeMsg.value.menu.file.find_album_folder,
        disabled: showFolderFiles.value,
        icon: IconFolderSearch,
        action: createAction('album-folder')
      },
      {
        label: localeMsg.value.menu.file.set_album_cover,
        action: createAction('set-album-cover')
      },
      {
        label: "-",   // separator
        action: () => { }
      },
      {
        label: localeMsg.value.menu.file.edit,
        icon: IconImageEdit,
        shortcut: isMac ? '⌘E' : 'Ctrl+E',
        disabled: f.file_type !== 1 && f.file_type !== 3,
        action: createAction('edit')
      },
      {
        label: localeMsg.value.menu.file.copy,
        icon: IconCopy,
        shortcut: isMac ? '⌘C' : 'Ctrl+C',
        disabled: f.file_type !== 1 && f.file_type !== 3,
        action: createAction('copy')
      },
      {
        label: localeMsg.value.menu.file.rename,
        icon: IconRename,
        action: createAction('rename')
      },
      {
        label: localeMsg.value.menu.file.move_to,
        icon: IconMoveTo,
        action: createAction('move-to')
      },
      {
        label: localeMsg.value.menu.file.copy_to,
        icon: IconFiles,
        action: createAction('copy-to')
      },
      {
        label: isMac ? localeMsg.value.menu.file.move_to_trash : localeMsg.value.menu.file.delete,
        icon: IconTrash,
        shortcut: isMac ? '⌘⌫' : 'Del',
        action: createAction('trash')
      },
      {
        label: isMac ? localeMsg.value.menu.file.reveal_in_finder : localeMsg.value.menu.file.reveal_in_file_explorer,
        action: createAction('reveal')
      },
      { label: "-", action: null },
      {
        label: f.is_favorite ? localeMsg.value.menu.meta.unfavorite : localeMsg.value.menu.meta.favorite,
        icon: f.is_favorite ? IconUnFavorite : IconFavorite,
        shortcut: isMac ? '⌘F' : 'Ctrl+F',
        action: createAction('favorite')
      },
      {
        label: localeMsg.value.menu.meta.tag,
        icon: IconTag,
        shortcut: isMac ? '⌘T' : 'Ctrl+T',
        action: createAction('tag')
      },
      {
        label: localeMsg.value.menu.meta.comment,
        icon: IconComment,
        action: createAction('comment')
      },
      {
        label: localeMsg.value.menu.meta.rotate,
        icon: IconRotate,
        shortcut: isMac ? '⌘R' : 'Ctrl+R',
        action: createAction('rotate')
      },
    ];
  });
};
