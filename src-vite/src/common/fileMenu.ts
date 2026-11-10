import { isOriginalStorageUnavailable, isOriginalUnavailable, requiresOriginalAction } from '@/common/availability';
import { computed, markRaw, Ref } from 'vue';
import { config, libConfig } from '@/common/config';
import { SIDEBAR } from '@/common/constants';
import { DEFAULT_PLATFORM, getShortcutLabel, ShortcutActionId } from '@/common/shortcuts';
import {
  IconMonitor,
  IconBrackets,
  IconPrint,
  IconCloud,
  IconRefresh,
  IconHeart,
  IconStar,
  IconStarFilled,
  IconTag,
  IconRotate,
  IconCopy,
  IconRename,
  IconFileArrowRight,
  IconTrash,
  IconComment,
  IconPhotoSearch,
  IconPersonSearch,
  IconImageEdit,
  IconExternal,
  IconHeartFilled,
  IconFlag,
  IconFlagFilled,
  IconFlagOff,
  IconCollection,
  IconSplitOn,
  IconSplitOn4,
  IconCollage,
} from '@/common/icons';
import { isWin } from '@/common/utils';

const OPEN_IN_APP_LABELS = {
  generic: ['open_in_app', 'Open in external app...'],
} as const;

type ExternalAppKind = 'image' | 'video';

export const useFileMenuItems = (
  file: Ref<any>,
  localeMsg: Ref<any>,
  isMac: boolean,
  translate: (key: string) => string,
  onAction: (action: string) => void,
  options?: {
    selectMode?: Ref<boolean>;
    selectionMediaKind?: Ref<'image' | 'video' | 'mixed' | 'empty'>;
    selectionCount?: Ref<number>;
    selectionHasUnavailable?: Ref<boolean>;
    selectionStorageUnavailable?: Ref<boolean>;
    selectionHasCloudOnly?: Ref<boolean>;
    selectionInCollection?: Ref<boolean>;
    selectionFirstFavorite?: Ref<boolean>;
  }
) => {
  const requiresAvailableFile = (action: string) => action === 'copy' || requiresOriginalAction(action);
  const actionUnavailable = (action: string, target: any) => {
    if (options?.selectMode?.value) {
      return action === 'copy'
        ? (options.selectionStorageUnavailable?.value ?? options.selectionHasUnavailable?.value)
        : options.selectionHasUnavailable?.value;
    }
    return action === 'copy'
      ? isOriginalStorageUnavailable(target)
      : isOriginalUnavailable(target);
  };
  const createAction = (actionName: string) => Object.assign(() => {
    if (requiresAvailableFile(actionName) && actionUnavailable(actionName, file.value)) return;
    onAction(actionName);
  }, { actionName });
  const shortcut = (actionId: ShortcutActionId) => getShortcutLabel(actionId, DEFAULT_PLATFORM);

  // Resolves a label string against the current locale, or returns the fallback if none was found.
  const menuLabel = ([key, fallback]: readonly [string, string]) =>
    String(localeMsg.value.menu.file[key] || fallback);

  const externalAppMenu = (kind?: ExternalAppKind) => {
    const apps = kind ? config.externalAppsFor(kind) : [];
    const defaultApp = kind ? config.defaultExternalApp(kind) : null;
    return {
      label: menuLabel(OPEN_IN_APP_LABELS.generic),
      icon: markRaw(IconExternal),
      children: [
        ...apps.map((app: any) => ({
          label: app.name || app.path,
          shortcut: app.id === defaultApp?.id ? shortcut('file.openExternalApp') : undefined,
          action: createAction(`open-external-app:${app.id}`),
        })),
        ...(apps.length ? [{ label: '-', action: null }] : []),
        {
          label: String(localeMsg.value.menu.file.manage_app || 'Manage app...'),
          action: createAction('manage-external-apps'),
        },
      ],
    };
  };

  // Creates a context menu for multi-select mode.
  const buildSelectionMenu = () => {
    const kind = options?.selectionMediaKind?.value ?? 'empty';
    const externalAppKind = kind === 'image' || kind === 'video' ? kind : undefined;
    const selectionCount = options?.selectionCount?.value ?? 0;
    const unavailable = options?.selectionHasUnavailable?.value === true;
    const batchActions = new Set([
      'tag', 'add-to-collection', 'comment', 'rotate', 'rename', 'copy', 'trash',
    ]);
    const batchLabels = new Set([
      localeMsg.value.rating.title,
      localeMsg.value.culling.title,
      translate('menu.file.move_copy'),
    ]);
    const batchItems = buildSingleFileMenu({}).filter(item =>
      batchActions.has(item.action?.actionName) || batchLabels.has(item.label)
    ).map(item => ({
      ...item,
      disabled: selectionCount === 0
        || (requiresAvailableFile(item.action?.actionName || '') && actionUnavailable(item.action.actionName, null))
        || (unavailable && item.label === translate('menu.file.move_copy')),
      hideInSelectionPanel: true,
    }));

    return [
      {
        label: String(localeMsg.value.menu.file.compare_selected_images || 'Compare selected images...'),
        icon: markRaw(selectionCount >= 3 ? IconSplitOn4 : IconSplitOn),
        disabled: kind !== 'image' || selectionCount < 2,
        action: createAction('compare-selected-images'),
      },
      // A mixed image+video selection has no single external-app target, so
      // disable the entry rather than showing an empty app list.
      { ...externalAppMenu(externalAppKind), disabled: unavailable || kind === 'mixed' },
      {
        label: String(localeMsg.value.menu.file.create_montage || 'Create montage...'),
        icon: markRaw(IconCollage),
        disabled: unavailable || selectionCount < 2,
        action: createAction('create-montage'),
      },
      { label: '-', action: null },
      {
        label: options?.selectionFirstFavorite?.value
          ? localeMsg.value.menu.meta.unfavorite
          : localeMsg.value.menu.meta.favorite,
        icon: markRaw(options?.selectionFirstFavorite?.value ? IconHeartFilled : IconHeart),
        shortcut: shortcut('meta.favorite'),
        disabled: selectionCount === 0,
        action: createAction(options?.selectionFirstFavorite?.value ? 'unfavorite-all' : 'favorite-all'),
        hideInSelectionPanel: true,
      },
      ...batchItems.filter(item =>
        !['rename', 'copy', 'trash'].includes(item.action?.actionName)
        && item.label !== translate('menu.file.move_copy')
      ),
      { label: '-', action: null },
      ...batchItems.filter(item => item.action?.actionName === 'rename'),
      ...batchItems.filter(item => item.action?.actionName === 'copy'),
      ...batchItems.filter(item => item.label === translate('menu.file.move_copy')),
      {
        label: localeMsg.value.menu.file.remove_from_collection,
        hidden: !options?.selectionInCollection?.value,
        disabled: selectionCount === 0,
        action: createAction('remove-selected-from-collection'),
        hideInSelectionPanel: true,
      },
      { label: '-', action: null },
      {
        label: localeMsg.value.menu.file.refresh_file_info,
        icon: markRaw(IconRefresh),
        disabled: unavailable || selectionCount === 0,
        action: createAction('refresh-file-info'),
      },
      {
        label: menuLabel(['index_cloud_files', 'Download, index, then free up space']),
        icon: markRaw(IconCloud),
        // Freeing up space is a Windows (OneDrive) operation.
        hidden: !isWin,
        disabled: unavailable || !options?.selectionHasCloudOnly?.value,
        action: createAction('index-cloud-files'),
      },
      {
        label: String(localeMsg.value.menu.file.save_xmp || 'Save Metadata to XMP'),
        icon: markRaw(IconBrackets),
        disabled: unavailable || selectionCount === 0,
        action: createAction('save-xmp'),
      },
      { label: '-', action: null },
      ...batchItems.filter(item => item.action?.actionName === 'trash'),
    ];
  };

  // Creates a context menu for a single focused file.
  const buildSingleFileMenu = (f: any) => {
    const isImage = f.file_type === 1 || f.file_type === 3;
    const isVideo = f.file_type === 2;
    // Grouped/query rows can omit album_id; the active album is still the
    // correct target, matching the action handler in Content.vue.
    const albumId = Number(libConfig.album.id || f.album_id);
    const canSetAlbumCover = config.main.sidebarIndex === SIDEBAR.ALBUM
      && libConfig.activePane !== 'collection'
      && isImage
      && albumId > 0;
    const canSetDesktopWallpaper = f.file_type === 1
      || (f.file_type === 3 && f.media_subtype === 'raw_jpeg_pair' && f.live_photo_video_path);
    const unavailable = isOriginalUnavailable(f);
    const items: any[] = [
      {
        label: localeMsg.value.menu.file.view_in_new_window,
        icon: markRaw(IconMonitor),
        shortcut: shortcut('file.openNewWindow'),
        action: createAction('open')
      },
      { ...externalAppMenu(isVideo ? 'video' : 'image'), hidden: !isImage && !isVideo },
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
        label: localeMsg.value.culling.title,
        icon: markRaw(
          Number(f.culling_flag ?? f.cullingFlag ?? 0) === 1
            ? IconFlagFilled
            : Number(f.culling_flag ?? f.cullingFlag ?? 0) === 2
              ? IconFlagOff
              : IconFlag,
        ),
        submenuOpenDelay: 200,
        children: [
          {
            label: localeMsg.value.culling.picks,
            icon: markRaw(IconFlagFilled),
            shortcut: shortcut('meta.culling.pick'),
            action: createAction('culling-pick'),
          },
          {
            label: localeMsg.value.culling.rejected,
            icon: markRaw(IconFlagOff),
            shortcut: shortcut('meta.culling.reject'),
            action: createAction('culling-reject'),
          },
          {
            label: localeMsg.value.culling.unreviewed,
            icon: markRaw(IconFlag),
            shortcut: shortcut('meta.culling.unreviewed'),
            action: createAction('culling-unreviewed'),
          },
        ],
      },
      {
        label: localeMsg.value.menu.meta.tag,
        icon: markRaw(IconTag),
        shortcut: shortcut('meta.tag'),
        action: createAction('tag')
      },
      {
        label: localeMsg.value.menu.meta.collection,
        icon: markRaw(IconCollection),
        shortcut: shortcut('meta.collection'),
        action: createAction('add-to-collection'),
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
            icon: markRaw(IconFileArrowRight),
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
        label: String(localeMsg.value.menu.file.save_xmp || 'Save Metadata to XMP'),
        icon: markRaw(IconBrackets),
        action: createAction('save-xmp'),
      },
      {
        label: localeMsg.value.menu.file.set_as,
        hidden: !canSetAlbumCover && !canSetDesktopWallpaper,
        children: [
          {
            label: localeMsg.value.menu.file.set_album_cover,
            hidden: !canSetAlbumCover,
            action: createAction('set-album-cover'),
          },
          {
            label: localeMsg.value.menu.file.set_desktop_wallpaper,
            hidden: !canSetDesktopWallpaper,
            action: createAction('set-desktop-wallpaper'),
          },
        ],
      },
      { label: "-", action: null },
      {
        label: localeMsg.value.menu.file.delete,
        icon: markRaw(IconTrash),
        shortcut: shortcut('file.trash'),
        action: createAction('trash')
      },
    ];
    const applyAvailability = (item: any): any => {
      const action = item.action?.actionName || '';
      const requiresAvailableGroup = item.label === menuLabel(OPEN_IN_APP_LABELS.generic)
        || item.label === translate('menu.file.move_copy');
      return {
        ...item,
        disabled: !!item.disabled
          || (action === 'copy'
            ? actionUnavailable(action, f)
            : unavailable && (requiresOriginalAction(action) || requiresAvailableGroup)),
        children: item.children?.map(applyAvailability),
      };
    };
    return items.map(applyAvailability);
  };

  return computed(() => {
    if (options?.selectMode?.value) return buildSelectionMenu();
    const f = file.value;
    if (!f) return [];
    return buildSingleFileMenu(f);
  });
};
