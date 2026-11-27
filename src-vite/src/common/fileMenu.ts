import { isOriginalStorageUnavailable, isOriginalUnavailable, requiresOriginalAction, requiresLocalContent } from '@/common/availability';
import { computed, markRaw, Ref } from 'vue';
import { config, libConfig } from '@/common/config';
import { SIDEBAR } from '@/common/constants';
import { DEFAULT_PLATFORM, getShortcutLabel, ShortcutActionId } from '@/common/shortcuts';
import {
  IconMonitor,
  IconSave,
  IconPrint,
  IconRefresh,
  IconCopy,
  IconRename,
  IconFileArrowRight,
  IconTrash,
  IconPhotoSearch,
  IconPersonSearch,
  IconFolderSearch,
  IconImageEdit,
  IconExternal,
  IconSplitOn,
  IconSplitOn4,
  IconCollage,
} from '@/common/icons';

import { isPreviewOnlyDocument } from '@/common/utils';

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
    selectionHasCloudOnly?: Ref<boolean>;
    selectionStorageUnavailable?: Ref<boolean>;
    selectionInCollection?: Ref<boolean>;
    inCollection?: Ref<boolean>;
    /** Hide "View in new window" instead of only disabling it. */
    hideViewInNewWindow?: boolean;
  }
) => {
  const requiresAvailableFile = (action: string) => action === 'copy' || requiresOriginalAction(action) || requiresLocalContent(action);
  const actionUnavailable = (action: string, target: any) => {
    const cloudOnly = options?.selectMode?.value ? options.selectionHasCloudOnly?.value : target?.cloud_only;
    if (cloudOnly && requiresLocalContent(action)) return true;
    if (options?.selectMode?.value) {
      return action === 'copy'
        ? (options.selectionStorageUnavailable?.value ?? options.selectionHasUnavailable?.value)
        : requiresOriginalAction(action) && options.selectionHasUnavailable?.value;
    }
    return action === 'copy'
      ? isOriginalStorageUnavailable(target)
      : requiresOriginalAction(action) && isOriginalUnavailable(target);
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
      'rename', 'copy', 'trash',
    ]);
    const batchLabels = new Set([
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
        disabled: kind !== 'image' || selectionCount < 2 || options?.selectionHasCloudOnly?.value,
        action: createAction('compare-selected-images'),
      },
      // A mixed image+video selection has no single external-app target, so
      // disable the entry rather than showing an empty app list.
      { ...externalAppMenu(externalAppKind), disabled: unavailable || options?.selectionHasCloudOnly?.value || kind === 'mixed' },
      {
        label: String(localeMsg.value.menu.file.create_montage || 'Create montage...'),
        icon: markRaw(IconCollage),
        disabled: unavailable || options?.selectionHasCloudOnly?.value || selectionCount < 2,
        action: createAction('create-montage'),
      },
      { label: '-', action: null },
      ...batchItems.filter(item => item.action?.actionName === 'rename'),
      ...batchItems.filter(item => item.action?.actionName === 'copy'),
      ...batchItems.filter(item => item.label === translate('menu.file.move_copy')),
      { label: '-', action: null },
      {
        label: localeMsg.value.menu.file.refresh_file_info,
        icon: markRaw(IconRefresh),
        disabled: unavailable || selectionCount === 0,
        action: createAction('refresh-file-info'),
      },
      {
        label: String(localeMsg.value.menu.file.save_xmp || 'Save Metadata to XMP'),
        icon: markRaw(IconSave),
        disabled: unavailable || options?.selectionHasCloudOnly?.value || selectionCount === 0,
        action: createAction('save-xmp'),
      },
      { label: '-', action: null },
      {
        label: localeMsg.value.menu.file.remove_from_collection,
        hidden: !options?.selectionInCollection?.value,
        disabled: selectionCount === 0,
        action: createAction('remove-selected-from-collection'),
        hideInSelectionPanel: true,
      },
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
        hidden: options?.hideViewInNewWindow === true,
        action: createAction('open')
      },
      { ...externalAppMenu(isVideo ? 'video' : 'image'), hidden: !isImage && !isVideo },
      {
        label: localeMsg.value.menu.file.edit_image,
        icon: markRaw(IconImageEdit),
        shortcut: shortcut('file.editImage'),
        disabled: !isImage || isPreviewOnlyDocument(f.file_path || ''),
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
      {
        label: localeMsg.value.menu.file.explore_same_folder,
        icon: markRaw(IconFolderSearch),
        disabled: !file.value?.album_id || !file.value?.file_path,
        action: createAction('explore-same-folder')
      },
      { label: "-", action: null },
      {
        label: localeMsg.value.menu.file.refresh_file_info,
        icon: markRaw(IconRefresh),
        action: createAction('refresh-file-info')
      },
      {
        label: String(localeMsg.value.menu.file.save_xmp || 'Save Metadata to XMP'),
        icon: markRaw(IconSave),
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
        label: localeMsg.value.menu.file.remove_from_collection,
        hidden: !options?.inCollection?.value,
        action: createAction('remove-from-collection'),
      },
      {
        label: translate(config.settings.fileDeleteBehavior === 'permanent' ? 'menu.file.delete' : 'msgbox.move_to_trash.title'),
        icon: markRaw(IconTrash),
        shortcut: shortcut(config.settings.fileDeleteBehavior === 'permanent' ? 'file.deletePermanently' : 'file.trash'),
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
          || (requiresAvailableFile(action) && actionUnavailable(action, f))
          || (unavailable && requiresAvailableGroup)
          || (item.label === menuLabel(OPEN_IN_APP_LABELS.generic)
            && (options?.selectMode?.value ? options.selectionHasCloudOnly?.value : f.cloud_only)),
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
