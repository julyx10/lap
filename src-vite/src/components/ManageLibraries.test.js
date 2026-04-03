import { ref } from 'vue';
import { shallowMount } from '@vue/test-utils';

const getAppConfigMock = vi.fn();
const getLibraryInfoMock = vi.fn();

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key) => key,
    locale: ref('en'),
    messages: ref({ en: { msgbox: { manage_libraries: {} } } }),
  }),
}));

vi.mock('@/stores/uiStore', () => ({
  useUIStore: () => ({
    pushInputHandler: vi.fn(),
    popInputHandler: vi.fn(),
  }),
}));

vi.mock('@/common/config', () => ({
  config: {
    main: {
      maxLibraryCount: 10,
    },
  },
}));

vi.mock('@/common/api', () => ({
  getAppConfig: (...args) => getAppConfigMock(...args),
  getLibraryInfo: (...args) => getLibraryInfoMock(...args),
  addLibrary: vi.fn(),
  editLibrary: vi.fn(),
  removeLibrary: vi.fn(),
  hideLibrary: vi.fn(),
  reorderLibraries: vi.fn(),
  switchLibrary: vi.fn(),
}));

vi.mock('@/common/utils', () => ({
  formatTimestamp: () => '',
  isValidFileName: () => true,
  formatFileSize: () => '',
  openFolderDialog: vi.fn(),
}));

import ManageLibraries from './ManageLibraries.vue';

const flushPromises = async () => {
  await Promise.resolve();
  await new Promise((resolve) => setTimeout(resolve, 0));
};

function createWrapper() {
  return shallowMount(ManageLibraries, {
    global: {
      mocks: {
        $t: (key) => key,
      },
      stubs: {
        ModalDialog: { template: '<div><slot /></div>' },
        MessageBox: true,
        ProgressBar: true,
        VueDraggable: { template: '<div><slot /></div>' },
        TButton: {
          props: ['disabled', 'tooltip'],
          template: '<button :data-tooltip="tooltip" :data-disabled="String(disabled)"><slot /></button>',
        },
      },
    },
  });
}

describe('ManageLibraries', () => {
  it('disables delete for disconnected libraries that still have metadata', async () => {
    getAppConfigMock.mockResolvedValue({
      current_library_id: 'default',
      libraries: [
        { id: 'default', name: 'Default Library', created_at: 1, hidden: false },
        { id: 'blocked', name: 'Blocked Library', created_at: 2, hidden: false },
        { id: 'healthy', name: 'Healthy Library', created_at: 3, hidden: false },
      ],
    });

    getLibraryInfoMock.mockImplementation(async (id) => {
      if (id === 'blocked') {
        return {
          dbFileSize: 0,
          fileCount: 0,
          totalSize: 0,
          storageAvailable: false,
          metadataInitialized: true,
          usesDefaultStorage: false,
          storageDir: '/Volumes/External/blocked',
          dbFilePath: '/Volumes/External/blocked.db',
        };
      }
      return {
        dbFileSize: 1024,
        fileCount: 12,
        totalSize: 2048,
        storageAvailable: true,
        metadataInitialized: true,
        usesDefaultStorage: true,
        storageDir: '/tmp',
        dbFilePath: `/tmp/${id}.db`,
      };
    });

    const wrapper = createWrapper();
    await vi.waitFor(() => {
      expect(getLibraryInfoMock).toHaveBeenCalledTimes(3);
    });
    await flushPromises();

    const deleteButtons = wrapper
      .findAll('button[data-tooltip="msgbox.manage_libraries.delete"]')
      .map((button) => button.attributes('data-disabled'));

    expect(deleteButtons).toEqual(['true', 'true', 'false']);
  });
});
