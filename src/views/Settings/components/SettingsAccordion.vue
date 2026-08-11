<script setup lang="ts">
import { ref } from 'vue'
import { cn } from '@/lib/utils'
import SettingsToggle from './SettingsToggle.vue'
import SettingsSelect from './SettingsSelect.vue'
import type { SettingItem } from '../config'

defineProps<{
  item: SettingItem
}>()

const isOpen = ref(false)
</script>

<template>
  <div>
    <button
      class="flex w-full items-center justify-between gap-4 px-5 py-3.5 text-left hover:bg-accent/50 transition-colors"
      @click="isOpen = !isOpen"
    >
      <div class="min-w-0 flex-1">
        <p class="text-sm font-medium text-foreground">{{ item.label }}</p>
        <p class="text-xs text-muted-foreground mt-0.5">{{ item.description }}</p>
      </div>
      <svg
        :class="cn('size-4 text-muted-foreground transition-transform shrink-0', isOpen && 'rotate-180')"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M6 9l6 6 6-6" />
      </svg>
    </button>
    <div
      v-show="isOpen"
      class="border-t border-border bg-muted/20 px-4 py-2 space-y-1"
    >
      <template v-for="child in item.children" :key="child.key">
        <SettingsToggle
          v-if="child.type === 'toggle'"
          :item="child"
        />
        <SettingsSelect
          v-else-if="child.type === 'select'"
          :item="child"
        />
      </template>
    </div>
  </div>
</template>
