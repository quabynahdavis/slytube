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
import { PhDownloadSimple, PhUploadSimple, PhTrash } from '@phosphor-icons/vue'
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
    <!-- Export -->
    <Button variant="ghost" size="sm" class="gap-1.5 h-7 text-xs" @click="handleExport">
      <PhDownloadSimple :size="14" />
      {{ t('settings.importExport.export') }}
    </Button>

    <!-- Import -->
    <Dialog v-model:open="importDialogOpen">
      <DialogTrigger as-child>
        <Button variant="ghost" size="sm" class="gap-1.5 h-7 text-xs">
          <PhUploadSimple :size="14" />
          {{ t('settings.importExport.import') }}
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{{ t('settings.importExport.importTitle') }}</DialogTitle>
          <DialogDescription>{{ t('settings.importExport.importDesc') }}</DialogDescription>
        </DialogHeader>
        <div class="py-4">
          <Button variant="outline" class="w-full" @click="triggerImport">
            <PhUploadSimple :size="16" class="mr-2" />
            {{ t('settings.importExport.chooseFile') }}
          </Button>
          <input ref="fileInput" type="file" accept=".json" class="hidden" @change="handleFileSelect" />
        </div>
        <DialogFooter>
          <Button variant="outline" @click="importDialogOpen = false">
            {{ t('settings.common.cancel') }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Delete All -->
    <Dialog>
      <DialogTrigger as-child>
        <Button variant="ghost" size="sm" class="gap-1.5 h-7 text-xs text-destructive hover:text-destructive">
          <PhTrash :size="14" />
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

    <!-- Import Preview -->
    <Dialog v-model:open="importPreviewOpen">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{{ t('settings.importExport.importPreview') }}</DialogTitle>
          <DialogDescription>
            {{ t('settings.importExport.importChanges', { count: importChangesCount }) }}
          </DialogDescription>
        </DialogHeader>
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
