<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settings'
import { useToast } from '@/composables/useToast'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import type { SettingsState } from '@/stores/settings'

const { t } = useI18n()
const settingsStore = useSettingsStore()
const toast = useToast()

const importDialogOpen = ref(false)
const importPreviewOpen = ref(false)
const importData = ref<Partial<SettingsState> | null>(null)
const importChangesCount = ref(0)
const fileInput = ref<HTMLInputElement | null>(null)

function handleExport() {
  const data = settingsStore.exportSettings()
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'slytube-settings.json'
  a.click()
  URL.revokeObjectURL(url)
  toast.success(t('settings.importExport.exportSuccess'))
}

function triggerImport() {
  fileInput.value?.click()
}

function handleFileSelect(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return

  const reader = new FileReader()
  reader.onload = (event) => {
    try {
      const data = JSON.parse(event.target?.result as string)
      if (typeof data !== 'object' || data === null) {
        throw new Error('Invalid format')
      }
      importData.value = data
      importChangesCount.value = Object.keys(data).length
      importPreviewOpen.value = true
    } catch {
      toast.error(t('settings.importExport.invalidFile'))
    }
  }
  reader.readAsText(file)
  ;(e.target as HTMLInputElement).value = ''
}

async function applyImport() {
  if (!importData.value) return
  await settingsStore.importSettings(importData.value)
  importPreviewOpen.value = false
  importDialogOpen.value = false
  importData.value = null
  toast.success(t('settings.importExport.importSuccess'))
}

async function handleDeleteAll() {
  await settingsStore.clearAllData()
  toast.success(t('settings.importExport.dataDeleted'))
}


</script>

<template>
  <div class="flex items-center gap-1">
    <!-- Import Button -->
    <Dialog v-model:open="importDialogOpen">
      <DialogTrigger as-child>
        <Button variant="ghost" size="sm" class="gap-2">
          <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3" />
          </svg>
          <span class="hidden sm:inline">{{ t('settings.importExport.import') }}</span>
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{{ t('settings.importExport.importTitle') }}</DialogTitle>
          <DialogDescription>{{ t('settings.importExport.importDesc') }}</DialogDescription>
        </DialogHeader>
        <div class="py-4">
          <Button variant="outline" class="w-full" @click="triggerImport">
            <svg class="size-4 mr-2" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
              <polyline points="14,2 14,8 20,8" />
            </svg>
            {{ t('settings.importExport.chooseFile') }}
          </Button>
          <input
            ref="fileInput"
            type="file"
            accept=".json"
            class="hidden"
            @change="handleFileSelect"
          />
        </div>
        <DialogFooter>
          <Button variant="outline" @click="importDialogOpen = false">
            {{ t('settings.common.cancel') }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Export Button -->
    <Button variant="ghost" size="sm" class="gap-2" @click="handleExport">
      <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M18 8l-5-5-5 5M12 3v12" />
      </svg>
      <span class="hidden sm:inline">{{ t('settings.importExport.export') }}</span>
    </Button>

    <!-- Delete All Data Button -->
    <Dialog>
      <DialogTrigger as-child>
        <Button variant="ghost" size="sm" class="gap-2 text-destructive hover:text-destructive">
          <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="3 6 5 6 21 6" />
            <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" />
          </svg>
          <span class="hidden sm:inline">{{ t('settings.importExport.deleteAll') }}</span>
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{{ t('settings.importExport.deleteTitle') }}</DialogTitle>
          <DialogDescription>{{ t('settings.importExport.deleteDesc') }}</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline">{{ t('settings.common.cancel') }}</Button>
          <Button variant="destructive" @click="handleDeleteAll">
            {{ t('settings.importExport.confirmDelete') }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Import Preview Dialog -->
    <Dialog v-model:open="importPreviewOpen">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{{ t('settings.importExport.importPreview') }}</DialogTitle>
          <DialogDescription>
            {{ t('settings.importExport.importChanges', { count: importChangesCount }) }}
          </DialogDescription>
        </DialogHeader>
        <div class="py-2 max-h-48 overflow-y-auto rounded-md border border-border bg-muted/30 px-3">
          <p class="text-xs text-muted-foreground py-2">
            {{ t('settings.importExport.importWarning') }}
          </p>
        </div>
        <DialogFooter>
          <Button variant="outline" @click="importPreviewOpen = false">
            {{ t('settings.common.cancel') }}
          </Button>
          <Button @click="applyImport">
            {{ t('settings.importExport.apply') }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
