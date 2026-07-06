import { computed, markRaw, Ref } from 'vue';
import { config } from '@/common/config';
import { DEFAULT_PLATFORM, getShortcutLabel, ShortcutActionId } from '@/common/shortcuts';
import {
  IconMonitor,
  IconPrint,
  IconRefresh,
  IconHeart,
  IconStar,
  IconStarFilled,
  IconTag,
  IconRotate,
  IconCopy,
  IconRename,
  IconMove,
  IconTrash,
  IconComment,
  IconPhotoSearch,
  IconPersonSearch,
  IconImageEdit,
  IconExternal,
  IconHeartFilled,
  IconBookmarkOff,
} from '@/common/icons';

// Label lookup table for "open in external app" command based on media type
// and item count.
const OPEN_IN_APP_LABELS = {
  image: {
    one: ['open_image_in_app', 'Open image in {app}...'],
    many: ['open_selected_images_in_app', 'Open selected images in {app}...'],
  },
  video: {
    one: ['open_video_in_app', 'Open video in {app}...'],
    many: ['open_selected_videos_in_app', 'Open selected videos in {app}...'],
  },
  generic: ['open_in_app', 'Open in external app...'],
} as const;

export const useFileMenuItems = (
  file: Ref<any>,
  localeMsg: Ref<any>,
  isMac: boolean,
  translate: (key: string) => string,
  onAction: (action: string) => void,
  options?: {
    isCollectionView?: Ref<boolean>;
    selectMode?: Ref<boolean>;
    selectionMediaKind?: Ref<'image' | 'video' | 'mixed' | 'empty'>;
    selectionCount?: Ref<number>;
  }
) => {
  const createAction = (actionName: string) => () => onAction(actionName);
  const shortcut = (actionId: ShortcutActionId) => getShortcutLabel(actionId, DEFAULT_PLATFORM);

  // Resolves a label string against the current locale, or returns the fallback if none was found.
  const menuLabel = ([key, fallback]: readonly [string, string]) =>
    String(localeMsg.value.menu.file[key] || fallback);

  // Constructs a label for the "open in external app" entry, depending on media type
  // and item count.
  const openInAppLabel = (kind: 'image' | 'video' | 'mixed' | 'empty', count = 1) => {
    if (kind !== 'image' && kind !== 'video') return menuLabel(OPEN_IN_APP_LABELS.generic);
    const name = String(
      (kind === 'video' ? config.settings.externalVideoAppName : config.settings.externalImageAppName) || '',
    );
    if (!name) return menuLabel(OPEN_IN_APP_LABELS.generic);
    const variant = OPEN_IN_APP_LABELS[kind][count > 1 ? 'many' : 'one'];
    return menuLabel(variant).replace('{app}', name);
  };

  // Creates a context menu for multi-select mode. Currently only shows the "open in external app" entry.
  //
  // If more entries are added in the future, we could refactor this to filter entries based on
  // selection instead of having two completely separate menus.
  const buildSelectionMenu = () => {
    const kind = options?.selectionMediaKind?.value ?? 'empty';
    const selectionIsVideo = kind === 'video';
    const selectionIsMixed = kind === 'mixed';
    // Gate on the app *path* (what actually launches), not the display name.
    const appPath = String(
      (selectionIsVideo ? config.settings.externalVideoAppPath : config.settings.externalImageAppPath) || '',
    );
    return [
      {
        label: openInAppLabel(kind, options?.selectionCount?.value ?? 0),
        icon: markRaw(IconExternal),
        hidden: kind === 'empty',
        disabled: selectionIsMixed || !appPath,
        shortcut: shortcut('file.openExternalApp'),
        action: createAction('open-external-app'),
      },
    ];
  };

  // Creates a context menu for a single focused file.
  const buildSingleFileMenu = (f: any) => {
    const isImage = f.file_type === 1 || f.file_type === 3;
    const isVideo = f.file_type === 2;
    const imageAppPath = String(config.settings.externalImageAppPath || '');
    const videoAppPath = String(config.settings.externalVideoAppPath || '');
    return [
      {
        label: localeMsg.value.menu.file.view_in_new_window,
        icon: markRaw(IconMonitor),
        shortcut: shortcut('file.openNewWindow'),
        action: createAction('open')
      },
      {
        label: openInAppLabel(isVideo ? 'video' : 'image'),
        hidden: !isImage && !isVideo,
        disabled: !((isImage && imageAppPath) || (isVideo && videoAppPath)),
        icon: markRaw(IconExternal),
        shortcut: shortcut('file.openExternalApp'),
        action: createAction('open-external-app')
      },
      {
        label: localeMsg.value.menu.file.edit_image,
        icon: markRaw(IconImageEdit),
        shortcut: shortcut('file.editImage'),
        disabled: !isImage,
        action: createAction('edit')
      },
      {
        label: localeMsg.value.menu.file.print,
        icon: markRaw(IconPrint),
        disabled: !isImage,
        action: createAction('print')
      },
      { label: "-", action: null },
      {
        label: f.is_favorite ? localeMsg.value.menu.meta.unfavorite : localeMsg.value.menu.meta.favorite,
        icon: markRaw(Number(f.is_favorite) ? IconHeartFilled : IconHeart),
        shortcut: shortcut('meta.favorite'),
        action: createAction('favorite')
      },
      {
        label: localeMsg.value.rating.title,
        icon: markRaw(Number(f.rating || 0) > 0 ? IconStarFilled : IconStar),
        submenuOpenDelay: 200,
        children: [
          {
            label: localeMsg.value.rating.clear_rating,
            icon: markRaw(IconStar),
            shortcut: shortcut('meta.rating.clear'),
            action: createAction('rating-0')
          },
          { label: '-', action: null },
          {
            label: localeMsg.value.rating.five_stars,
            icon: markRaw(Number(f.rating || 0) === 5 ? IconStarFilled : IconStar),
            shortcut: shortcut('meta.rating.five'),
            action: createAction('rating-5')
          },
          {
            label: localeMsg.value.rating.four_stars,
            icon: markRaw(Number(f.rating || 0) === 4 ? IconStarFilled : IconStar),
            shortcut: shortcut('meta.rating.four'),
            action: createAction('rating-4')
          },
          {
            label: localeMsg.value.rating.three_stars,
            icon: markRaw(Number(f.rating || 0) === 3 ? IconStarFilled : IconStar),
            shortcut: shortcut('meta.rating.three'),
            action: createAction('rating-3')
          },
          {
            label: localeMsg.value.rating.two_stars,
            icon: markRaw(Number(f.rating || 0) === 2 ? IconStarFilled : IconStar),
            shortcut: shortcut('meta.rating.two'),
            action: createAction('rating-2')
          },
          {
            label: localeMsg.value.rating.one_star,
            icon: markRaw(Number(f.rating || 0) === 1 ? IconStarFilled : IconStar),
            shortcut: shortcut('meta.rating.one'),
            action: createAction('rating-1')
          },
        ]
      },
      {
        label: localeMsg.value.menu.meta.tag,
        icon: markRaw(IconTag),
        shortcut: shortcut('meta.tag'),
        action: createAction('tag')
      },
      {
        label: localeMsg.value.menu.meta.comment,
        icon: markRaw(IconComment),
        shortcut: shortcut('meta.comment'),
        action: createAction('comment')
      },
      {
        label: localeMsg.value.menu.meta.rotate,
        icon: markRaw(IconRotate),
        shortcut: shortcut('meta.rotate'),
        action: createAction('rotate')
      },
      { label: "-", action: null },
      {
        label: localeMsg.value.menu.file.rename,
        icon: markRaw(IconRename),
        shortcut: shortcut('file.rename'),
        action: createAction('rename')
      },
      {
        label: localeMsg.value.menu.file.copy,
        icon: markRaw(IconCopy),
        shortcut: shortcut('file.copy'),
        action: createAction('copy')
      },

      {
        label: translate('menu.file.move_copy'),
        children: [
          {
            label: translate('menu.file.move_within_library'),
            icon: markRaw(IconMove),
            shortcut: shortcut('file.moveTo'),
            action: createAction('move-within-library')
          },
          {
            label: translate('menu.file.move_to_folder'),
            shortcut: shortcut('file.moveToFolder'),
            action: createAction('move-to-folder')
          },
          {
            label: translate('menu.file.copy_to_folder'),
            action: createAction('copy-to-folder')
          },
        ]
      },
      {
        label: isMac ? localeMsg.value.menu.file.reveal_in_finder : localeMsg.value.menu.file.reveal_in_file_explorer,
        shortcut: shortcut('file.reveal'),
        action: createAction('reveal')
      },
      { label: "-", action: null },
      {
        label: localeMsg.value.menu.file.find_similar_images,
        icon: markRaw(IconPhotoSearch),
        shortcut: shortcut('file.searchSimilar'),
        disabled: !isImage,
        action: createAction('search-similar')
      },
      {
        label: localeMsg.value.menu.file.find_person_images,
        icon: markRaw(IconPersonSearch),
        hidden: !config.settings.face.enabled,
        disabled: !isImage,
        action: createAction('find-person')
      },
      { label: "-", action: null },
      {
        label: localeMsg.value.menu.file.refresh_file_info,
        icon: markRaw(IconRefresh),
        action: createAction('refresh-file-info')
      },
      {
        label: localeMsg.value.menu.file.set_album_cover,
        hidden: !isImage || !Number(f.album_id),
        action: createAction('set-album-cover')
      },
      { label: "-", action: null },
      {
        label: options?.isCollectionView?.value
          ? translate('menu.file.remove_from_collection')
          : localeMsg.value.menu.file.move_to_trash,
        icon: markRaw(options?.isCollectionView?.value ? IconBookmarkOff : IconTrash),
        shortcut: shortcut('file.trash'),
        action: createAction(options?.isCollectionView?.value ? 'remove-from-collection' : 'trash')
      },
    ];
  };

  return computed(() => {
    if (options?.selectMode?.value) return buildSelectionMenu();
    const f = file.value;
    if (!f) return [];
    return buildSingleFileMenu(f);
  });
};
