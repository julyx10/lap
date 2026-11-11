import { computed, onScopeDispose, ref, watch } from 'vue';
import { useConfigStore } from '@/stores/configStore';
import { setTheme } from '@/common/utils';

// Keep the saved mode (0: light, 1: dark, 2: system) separate from the displayed mode.
export function useAppearance() {
  const config = useConfigStore();
  const media = window.matchMedia('(prefers-color-scheme: dark)');
  const systemDark = ref(media.matches);
  const onChange = (event: MediaQueryListEvent) => {
    systemDark.value = event.matches;
  };

  media.addEventListener('change', onChange);
  onScopeDispose(() => media.removeEventListener('change', onChange));

  return computed(() => config.settings.appearance === 2
    ? (systemDark.value ? 1 : 0)
    : config.settings.appearance);
}

// App owns theme application for every window, including its loading screen.
export function useAppTheme() {
  const config = useConfigStore();
  const appearance = useAppearance();

  watch(
    [appearance, () => config.settings.lightTheme, () => config.settings.darkTheme],
    ([mode, lightTheme, darkTheme]) => {
      setTheme(mode, mode === 0 ? lightTheme : darkTheme);
    },
    { immediate: true },
  );
}
