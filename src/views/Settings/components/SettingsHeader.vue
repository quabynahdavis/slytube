<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui/button'

const { t } = useI18n()

defineProps<{
  categoryLabel: string
}>()

const emit = defineEmits<{
  back: []
}>()
</script>

<template>
  <header class="sticky top-0 z-40 flex items-center gap-3 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 px-4 py-3">
    <Button variant="ghost" size="icon" class="shrink-0" @click="emit('back')">
      <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M19 12H5M12 19l-7-7 7-7" />
      </svg>
    </Button>
    <div class="flex items-center gap-2 text-sm text-muted-foreground">
      <router-link to="/settings" class="hover:text-foreground transition-colors">
        {{ t('settings.title') }}
      </router-link>
      <svg class="size-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M9 18l6-6-6-6" />
      </svg>
      <span class="font-medium text-foreground">{{ categoryLabel }}</span>
    </div>
  </header>
</template>
