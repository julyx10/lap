<template>
  <div class="h-6 flex items-center gap-0.5">
    <button
      class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0 mr-1 relative"
      :title="withShortcut(favoriteTitle, 'meta.favorite')"
      :aria-label="withMixedState(favoriteTitle, favorite === null)"
      :disabled="disabled"
      @click.stop="emit('favorite', favorite === true ? false : true)"
    >
      <component
        :is="favorite === true ? IconHeartFilled : IconHeart"
        class="w-3.5 h-3.5"
        :class="favoriteIconClass"
      />
      <IconHeartFilled v-if="favorite === null" aria-hidden="true"
        class="absolute w-3.5 h-3.5 [clip-path:inset(0_50%_0_0)]"
        :class="disabled ? 'text-base-content/30' : 'text-error'" />
    </button>
    <div class="w-px h-4 shrink-0 bg-base-content/10 mx-1"></div>
    <button
      v-for="value in [1, 2, 3, 4, 5]"
      :key="value"
      class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0 relative"
      :title="withShortcut(getRatingLabel(value), ratingShortcuts[value - 1])"
      :aria-label="withMixedState(getRatingLabel(value), rating === null)"
      :disabled="disabled"
      @click.stop="emit('rating', rating === value ? 0 : value)"
    >
      <component
        :is="Number(rating || 0) >= value ? IconStarFilled : IconStar"
        class="w-3.5 h-3.5"
        :class="getRatingIconClass(value)"
      />
    </button>
    <span v-if="rating === null && !disabled"
      class="ml-1 shrink-0 rounded bg-base-content/5 px-1 text-[10px] text-base-content/60"
      :title="$t('rating.title')">
      {{ $t('rating.mixed') }}
    </span>
    <div class="w-px h-4 shrink-0 bg-base-content/10 mx-1"></div>
    <button
      class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0"
      :title="withShortcut($t('culling.picks'), 'meta.culling.pick')"
      :aria-label="getCullingLabel('picks')"
      :disabled="disabled"
      @click.stop="emit('culling', 1)"
    >
      <component :is="IconFlagFilled" class="w-3.5 h-3.5" :class="getCullingIconClass(1)" />
    </button>
    <button
      class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0"
      :title="withShortcut($t('culling.rejected'), 'meta.culling.reject')"
      :aria-label="getCullingLabel('rejected')"
      :disabled="disabled"
      @click.stop="emit('culling', 2)"
    >
      <component :is="IconFlagOff" class="w-3.5 h-3.5" :class="getCullingIconClass(2)" />
    </button>
    <button
      class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0"
      :title="withShortcut($t('culling.unreviewed'), 'meta.culling.unreviewed')"
      :aria-label="getCullingLabel('unreviewed')"
      :disabled="disabled"
      @click.stop="emit('culling', 0)"
    >
      <component :is="IconFlag" class="w-3.5 h-3.5" :class="getCullingIconClass(0)" />
    </button>
    <span v-if="culling === null && !disabled"
      class="ml-1 shrink-0 rounded bg-base-content/5 px-1 text-[10px] text-base-content/60"
      :title="$t('culling.title')">
      {{ $t('rating.mixed') }}
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { getShortcutLabel, type ShortcutActionId } from '@/common/shortcuts';
import { IconFlag, IconFlagFilled, IconFlagOff, IconHeart, IconHeartFilled, IconStar, IconStarFilled } from '@/common/icons';

const props = withDefaults(defineProps<{
  favorite: boolean | null;
  rating: number | null;
  culling: number | null;
  disabled?: boolean;
  favoriteLabel?: string;
  unfavoriteLabel?: string;
  labelClass?: string;
  inactiveRatingClass?: string;
}>(), {
  disabled: false,
  favoriteLabel: '',
  unfavoriteLabel: '',
  labelClass: 'text-base-content/50',
  inactiveRatingClass: 'text-base-content/30',
});

const emit = defineEmits<{
  favorite: [value: boolean];
  rating: [value: number];
  culling: [value: number];
}>();

const { locale, messages, t } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const favoriteTitle = computed(() => {
  if (props.favorite) {
    return props.unfavoriteLabel || t('menu.meta.unfavorite');
  }
  return props.favoriteLabel || t('menu.meta.favorite');
});

const favoriteIconClass = computed(() => {
  if (props.disabled) return 'text-base-content/30';
  if (props.favorite === true) return 'text-error';
  return 'text-base-content/70 hover:text-error/70';
});

const ratingShortcuts: ShortcutActionId[] = [
  'meta.rating.one', 'meta.rating.two', 'meta.rating.three', 'meta.rating.four', 'meta.rating.five',
];

function withShortcut(label: string, action: ShortcutActionId) {
  const shortcut = !props.disabled && getShortcutLabel(action);
  return shortcut ? `${label} (${shortcut})` : label;
}

function withMixedState(label: string, mixed: boolean) {
  return mixed ? `${label} (${t('rating.mixed')})` : label;
}

function getRatingLabel(rating: number) {
  const keys: Record<number, string> = {
    5: 'five_stars',
    4: 'four_stars',
    3: 'three_stars',
    2: 'two_stars',
    1: 'one_star',
  };
  const key = keys[rating];
  return localeMsg.value.rating?.[key] || `${rating}★`;
}

function getRatingIconClass(value: number) {
  if (props.disabled) return 'text-base-content/30';
  if (props.rating === null) return 'text-base-content/70 hover:text-warning/70';
  if (Number(props.rating || 0) >= value) return 'text-warning';
  return `${props.inactiveRatingClass} hover:text-warning/70`;
}

function getCullingLabel(state: 'picks' | 'rejected' | 'unreviewed') {
  const label = t(`culling.${state}`);
  return withMixedState(label, props.culling === null);
}

function getCullingIconClass(value: number) {
  if (props.disabled) return 'text-base-content/30';
  if (props.culling === null) return 'text-base-content/70 hover:text-primary/70';
  if (Number(props.culling) === value) {
    return value === 2 ? 'text-error' : 'text-primary';
  }
  return value === 2 ? 'text-base-content/30 hover:text-error/70' : 'text-base-content/30 hover:text-primary/70';
}
</script>
