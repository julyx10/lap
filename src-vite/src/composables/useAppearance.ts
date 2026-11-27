import { computed, onScopeDispose, ref, watch } from 'vue';
import { useConfigStore } from '@/stores/configStore';
import { applyTheme, findTheme, sanitizeCustomTheme, type CustomTheme, type ThemeAppearance } from '@/common/themes';

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

  const resolved = computed<ThemeAppearance>(() => appearance.value === 0 ? 'light' : 'dark');
  const customThemes = computed<CustomTheme[]>(() => {
    const list = Array.isArray(config.settings.customThemes) ? config.settings.customThemes : [];
    return list
      .map(theme => sanitizeCustomTheme(theme))
      .filter((theme): theme is CustomTheme => Boolean(theme));
  });

  watch(
    [
      resolved,
      () => config.settings.lightThemeId,
      () => config.settings.darkThemeId,
      customThemes,
    ],
    ([mode, lightThemeId, darkThemeId, themes]) => {
      const selectedId = mode === 'light' ? lightThemeId : darkThemeId;
      applyTheme(findTheme(selectedId, mode, themes));
    },
    { immediate: true, deep: true },
  );
}
