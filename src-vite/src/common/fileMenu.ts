import { computed, Ref } from 'vue';
import {
  IconMonitor,
  IconHeart,
  IconStar,
  IconStarFilled,
  IconTag,
  IconRotate,
  IconCopy,
  IconRename,
  IconMoveTo,
  IconCopyTo,
  IconTrash,
  IconComment,
  IconPhotoSearch,
  IconFiles,
  IconFolderSearch,
  IconPersonSearch,
  IconCrop,
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
        disabled: f.file_type !== 1 && f.file_type !== 3,
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
        label: localeMsg.value.menu.file.transform,
        icon: IconCrop,
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
        shortcut: isMac ? '⌘R' : 'Ctrl+R',
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
        icon: IconHeart,
        shortcut: isMac ? '⌘F' : 'Ctrl+F',
        action: createAction('favorite')
      },
      {
        label: localeMsg.value.favorite.ratings,
        icon: IconStar,
        children: [
          {
            label: localeMsg.value.favorite.clear_rating,
            icon: IconStar,
            action: createAction('rating-0')
          },
          { label: '-', action: null },
          {
            label: localeMsg.value.favorite.five_stars,
            icon: Number(f.rating || 0) === 5 ? IconStarFilled : IconStar,
            shortcut: isMac ? '⌘5' : 'Ctrl+5',
            action: createAction('rating-5')
          },
          {
            label: localeMsg.value.favorite.four_stars,
            icon: Number(f.rating || 0) === 4 ? IconStarFilled : IconStar,
            shortcut: isMac ? '⌘4' : 'Ctrl+4',
            action: createAction('rating-4')
          },
          {
            label: localeMsg.value.favorite.three_stars,
            icon: Number(f.rating || 0) === 3 ? IconStarFilled : IconStar,
            shortcut: isMac ? '⌘3' : 'Ctrl+3',
            action: createAction('rating-3')
          },
          {
            label: localeMsg.value.favorite.two_stars,
            icon: Number(f.rating || 0) === 2 ? IconStarFilled : IconStar,
            shortcut: isMac ? '⌘2' : 'Ctrl+2',
            action: createAction('rating-2')
          },
          {
            label: localeMsg.value.favorite.one_star,
            icon: Number(f.rating || 0) === 1 ? IconStarFilled : IconStar,
            shortcut: isMac ? '⌘1' : 'Ctrl+1',
            action: createAction('rating-1')
          },
        ]
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
        // shortcut: isMac ? '⌘R' : 'Ctrl+R',
        action: createAction('rotate')
      },
    ];
  });
};
