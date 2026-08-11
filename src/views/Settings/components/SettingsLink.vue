<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

const { t } = useI18n()
const router = useRouter()

const props = defineProps<{
  item: {
    label: string
    description: string
    crossLink?: { category: string; label: string }
  }
}>()

function navigate() {
  if (props.item.crossLink) {
    router.push(`/settings/${props.item.crossLink.category}`)
  }
}
</script>

<template>
  <div class="px-5 py-3 border-t border-border">
    <p class="text-xs text-muted-foreground">
      {{ t('settings.common.lookingFor') }}
      <button
        class="text-primary font-medium hover:underline"
        @click="navigate"
      >
        {{ t(item.crossLink?.label || '') }}
      </button>
    </p>
  </div>
</template>
