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
} from '@/common/icons';

export const useFileMenuItems = (
  file: Ref<any>,
  localeMsg: Ref<any>,
  isMac: boolean,
  translate: (key: string) => string,
  onAction: (action: string) => void
) => {
  return computed(() => {
    const f = file.value;
    if (!f) return [];

    const createAction = (actionName: string) => () => onAction(actionName);
    const imageAppName = String(config.settings.externalImageAppName || '');
    const videoAppName = String(config.settings.externalVideoAppName || '');
    const isImage = f.file_type === 1 || f.file_type === 3;
    const isVideo = f.file_type === 2;
    const shortcut = (actionId: ShortcutActionId) => getShortcutLabel(actionId, DEFAULT_PLATFORM);

    return [
      {
        label: localeMsg.value.menu.file.view_in_new_window,
        icon: markRaw(IconMonitor),
        shortcut: shortcut('file.openNewWindow'),
        action: createAction('open')
      },
      {
        label: (
          isVideo
            ? (localeMsg.value.menu.file.open_video_in_app || 'Open video in {app}...')
            : (localeMsg.value.menu.file.open_image_in_app || 'Open image in {app}...')
        ).replace('{app}', isVideo ? videoAppName : imageAppName),
        hidden: !((isImage && imageAppName) || (isVideo && videoAppName)),
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
        label: localeMsg.value.favorite.ratings,
        icon: markRaw(Number(f.rating || 0) > 0 ? IconStarFilled : IconStar),
        submenuOpenDelay: 200,
        children: [
          {
            label: localeMsg.value.favorite.clear_rating,
            icon: markRaw(IconStar),
            shortcut: shortcut('meta.rating.clear'),
            action: createAction('rating-0')
          },
          { label: '-', action: null },
          {
            label: localeMsg.value.favorite.five_stars,
            icon: markRaw(Number(f.rating || 0) === 5 ? IconStarFilled : IconStar),
            shortcut: shortcut('meta.rating.five'),
            action: createAction('rating-5')
          },
          {
            label: localeMsg.value.favorite.four_stars,
            icon: markRaw(Number(f.rating || 0) === 4 ? IconStarFilled : IconStar),
            shortcut: shortcut('meta.rating.four'),
            action: createAction('rating-4')
          },
          {
            label: localeMsg.value.favorite.three_stars,
            icon: markRaw(Number(f.rating || 0) === 3 ? IconStarFilled : IconStar),
            shortcut: shortcut('meta.rating.three'),
            action: createAction('rating-3')
          },
          {
            label: localeMsg.value.favorite.two_stars,
            icon: markRaw(Number(f.rating || 0) === 2 ? IconStarFilled : IconStar),
            shortcut: shortcut('meta.rating.two'),
            action: createAction('rating-2')
          },
          {
            label: localeMsg.value.favorite.one_star,
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
        label: localeMsg.value.menu.file.move_to_trash,
        icon: markRaw(IconTrash),
        shortcut: shortcut('file.trash'),
        action: createAction('trash')
      },
    ];
  });
};
