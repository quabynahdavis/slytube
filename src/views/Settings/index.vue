<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { settingsConfig } from './config'
import SettingsSearch from './components/SettingsSearch.vue'
import QuickAccess from './components/QuickAccess.vue'
import CategoryCard from './components/CategoryCard.vue'

const { t } = useI18n()
</script>

<template>
  <div class="min-h-screen bg-background">
    <!-- Header -->
    <header class="sticky top-0 z-40 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 px-4 py-4">
      <h1 class="text-xl font-bold text-foreground mb-3">{{ t('settings.title') }}</h1>
      <SettingsSearch />
    </header>

    <!-- Child Route View -->
    <div v-if="$route.name !== 'settings'" class="min-h-screen">
      <router-view />
    </div>

    <!-- Content -->
    <div v-else class="max-w-3xl mx-auto px-4 py-6 space-y-8">
      <!-- Quick Access -->
      <QuickAccess />

      <!-- Categories -->
      <div class="space-y-3">
        <h2 class="text-sm font-semibold text-foreground">{{ t('settings.title') }}</h2>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <CategoryCard
            v-for="category in settingsConfig"
            :key="category.id"
            :category="category"
          />
        </div>
      </div>
    </div>
  </div>
</template>
