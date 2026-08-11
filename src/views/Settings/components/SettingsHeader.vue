<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import ImportExport from './ImportExport.vue'
import { useSettingsStore } from '@/stores/settings'
import { computed, ref } from 'vue'
import type { SettingsState } from '@/stores/settings'

const { t } = useI18n()
const settingsStore = useSettingsStore()

const props = defineProps<{
  categoryLabel: string
  categoryKeys?: (keyof SettingsState)[]
}>()

const emit = defineEmits<{
  back: []
}>()

const resetDialogOpen = ref(false)

const changedCount = computed(() => {
  if (!props.categoryKeys) return 0
  return props.categoryKeys.filter(key => settingsStore.isSettingChanged(key)).length
})

async function resetCategory() {
  if (!props.categoryKeys) return
  await settingsStore.resetCategoryToDefaults(props.categoryKeys)
  resetDialogOpen.value = false
}
</script>

<template>
  <header class="sticky top-0 z-40 flex items-center gap-3 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 px-4 py-3">
    <Button variant="ghost" size="icon" class="shrink-0" @click="emit('back')">
      <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M19 12H5M12 19l-7-7 7-7" />
      </svg>
    </Button>
    <div class="flex items-center gap-2 text-sm text-muted-foreground flex-1 min-w-0">
      <router-link to="/settings" class="hover:text-foreground transition-colors shrink-0">
        {{ t('settings.title') }}
      </router-link>
      <svg class="size-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M9 18l6-6-6-6" />
      </svg>
      <span class="font-medium text-foreground truncate">{{ categoryLabel }}</span>
      <span v-if="changedCount > 0" class="text-xs text-yellow-500 font-medium shrink-0">
        ({{ changedCount }} changed)
      </span>
    </div>
    <div class="flex items-center gap-2 shrink-0">
      <Dialog v-model:open="resetDialogOpen">
        <DialogTrigger as-child>
          <Button
            v-if="categoryKeys?.length"
            variant="ghost"
            size="sm"
            class="gap-1.5 text-muted-foreground"
          >
            <svg class="size-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 12a9 9 0 109-9 9.75 9.75 0 00-6.74 2.74L3 8" />
              <path d="M3 3v5h5" />
            </svg>
            <span class="hidden sm:inline">{{ t('settings.common.resetCategory') }}</span>
          </Button>
        </DialogTrigger>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{{ t('settings.common.resetCategoryTitle') }}</DialogTitle>
            <DialogDescription>{{ t('settings.common.resetCategoryDesc') }}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" @click="resetDialogOpen = false">
              {{ t('settings.common.cancel') }}
            </Button>
            <Button variant="destructive" @click="resetCategory">
              {{ t('settings.common.reset') }}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <ImportExport />
    </div>
  </header>
</template>
