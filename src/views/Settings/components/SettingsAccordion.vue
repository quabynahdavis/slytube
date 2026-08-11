<script setup lang="ts">
import SettingsToggle from './SettingsToggle.vue'
import SettingsSelect from './SettingsSelect.vue'
import type { SettingItem } from '../config'
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from '@/components/ui/accordion'

defineProps<{
  item: SettingItem
}>()

function getChildItems(children: SettingItem[] = []): SettingItem[] {
  return children.filter(c => c.type === 'toggle' || c.type === 'select')
}
</script>

<template>
  <Accordion type="single" collapsible class="w-full">
    <AccordionItem :value="item.key" class="border-b-0">
      <AccordionTrigger class="px-5 py-3.5 hover:bg-accent/50 no-underline hover:no-underline">
        <div class="flex-1 text-left">
          <p class="text-sm font-medium text-foreground">{{ item.label }}</p>
          <p class="text-xs text-muted-foreground mt-0.5 font-normal">{{ item.description }}</p>
        </div>
      </AccordionTrigger>
      <AccordionContent class="px-4 py-2 space-y-1 !pb-3">
        <template v-for="child in getChildItems(item.children)" :key="child.key">
          <SettingsToggle
            v-if="child.type === 'toggle'"
            :item="child"
          />
          <SettingsSelect
            v-else-if="child.type === 'select'"
            :item="child"
          />
        </template>
      </AccordionContent>
    </AccordionItem>
  </Accordion>
</template>
