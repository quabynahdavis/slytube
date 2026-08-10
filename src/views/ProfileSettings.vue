<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { cn } from '@/lib/utils'
import { useProfilesStore } from '@/stores/profiles'

const profilesStore = useProfilesStore()

const isLoading = ref(true)
const showCreateForm = ref(false)
const editingProfileId = ref<string | null>(null)
const newProfileName = ref('')
const editProfileName = ref('')

const profiles = computed(() => profilesStore.getProfileList)
const activeProfile = computed(() => profilesStore.getActiveProfile)

onMounted(async () => {
  isLoading.value = true
  try {
    await new Promise((r) => setTimeout(r, 400))
  } finally {
    isLoading.value = false
  }
})

function createProfile() {
  if (!newProfileName.value.trim()) return
  profilesStore.createProfile(newProfileName.value.trim())
  newProfileName.value = ''
  showCreateForm.value = false
}

function startEdit(profile: { _id: string; name: string }) {
  editingProfileId.value = profile._id
  editProfileName.value = profile.name
}

function saveEdit(profileId: string) {
  const profile = profilesStore.profileById(profileId)
  if (profile && editProfileName.value.trim()) {
    profile.name = editProfileName.value.trim()
    profilesStore.sortProfiles()
  }
  editingProfileId.value = null
}

function deleteProfile(id: string) {
  profilesStore.deleteProfile(id)
}

function setActiveProfile(id: string) {
  profilesStore.setActiveProfile(id)
}
</script>

<template>
  <div class="container mx-auto max-w-4xl px-4 py-6">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold text-foreground">Profiles</h1>
        <p class="text-sm text-muted-foreground mt-1">Manage profiles and subscriptions</p>
      </div>
      <button class="inline-flex items-center gap-1 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors" @click="showCreateForm = true">
        <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        New Profile
      </button>
    </div>

    <div v-if="isLoading" class="space-y-3">
      <div v-for="n in 3" :key="n" class="animate-pulse rounded-lg border border-border p-4"><div class="h-5 w-48 rounded bg-muted"/></div>
    </div>

    <div v-else class="space-y-3">
      <div v-for="profile in profiles" :key="profile._id" :class="cn('rounded-lg border border-border bg-card p-4 transition-colors', activeProfile?._id === profile._id && 'border-primary/50 bg-primary/5')">
        <div class="flex items-center justify-between gap-4">
          <div class="flex items-center gap-3 min-w-0">
            <div class="size-10 rounded-full flex items-center justify-center text-white font-bold text-sm shrink-0" :style="{ backgroundColor: profile.bgColor }">
              {{ profile.name.charAt(0).toUpperCase() }}
            </div>
            <div class="min-w-0">
              <template v-if="editingProfileId === profile._id">
                <input v-model="editProfileName" class="h-8 rounded-md border border-input bg-background px-2 text-sm" @keyup.enter="saveEdit(profile._id)" @keyup.escape="editingProfileId = null"/>
              </template>
              <template v-else>
                <h3 class="text-sm font-medium text-foreground truncate">{{ profile.name }}</h3>
                <p class="text-xs text-muted-foreground">{{ profile.subscriptions.length }} subscriptions</p>
              </template>
            </div>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <button v-if="activeProfile?._id !== profile._id" class="h-8 rounded-md border border-border px-3 text-xs text-muted-foreground hover:bg-accent transition-colors" @click="setActiveProfile(profile._id)">Activate</button>
            <span v-else class="rounded-full bg-primary/10 px-2 py-0.5 text-xs text-primary font-medium">Active</span>
            <button v-if="editingProfileId === profile._id" class="h-8 rounded-md bg-primary px-3 text-xs text-primary-foreground" @click="saveEdit(profile._id)">Save</button>
            <button v-else class="size-8 rounded-md text-muted-foreground hover:bg-accent flex items-center justify-center" @click="startEdit(profile)"><svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg></button>
            <button v-if="profile._id !== 'allChannels'" class="size-8 rounded-md text-muted-foreground hover:bg-destructive/10 hover:text-destructive flex items-center justify-center" @click="deleteProfile(profile._id)"><svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg></button>
          </div>
        </div>
      </div>
    </div>

    <Teleport to="body">
      <div v-if="showCreateForm" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="showCreateForm = false">
        <div class="w-full max-w-sm rounded-lg bg-card border border-border p-6 shadow-xl">
          <h3 class="text-lg font-semibold text-foreground mb-4">Create Profile</h3>
          <div class="space-y-4">
            <div><label class="text-sm font-medium text-foreground">Profile Name</label><input v-model="newProfileName" type="text" placeholder="Enter profile name" class="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary" @keyup.enter="createProfile"/></div>
            <div class="flex justify-end gap-2">
              <button class="h-9 rounded-md border border-input bg-background px-4 text-sm font-medium text-foreground hover:bg-accent transition-colors" @click="showCreateForm = false">Cancel</button>
              <button class="h-9 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors" @click="createProfile">Create</button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
