import { describe, it, expect } from 'vitest'
import { settingsConfig, type SettingsCategory } from '@/views/Settings/config'

describe('Settings Config', () => {
  it('has exactly 7 categories', () => {
    expect(settingsConfig).toHaveLength(7)
  })

  const expectedIds = ['general', 'appearance', 'player', 'downloads', 'privacy', 'sync', 'advanced']

  it('has all expected category IDs', () => {
    const ids = settingsConfig.map(c => c.id)
    expect(ids).toEqual(expectedIds)
  })

  it('every category has a label, description, icon, and route', () => {
    for (const category of settingsConfig) {
      expect(category.label).toBeTruthy()
      expect(category.description).toBeTruthy()
      expect(category.icon).toBeTruthy()
      expect(category.route).toMatch(/^\/settings\//)
      expect(category.sections.length).toBeGreaterThan(0)
    }
  })

  it('every section has at least one item', () => {
    for (const category of settingsConfig) {
      for (const section of category.sections) {
        expect(section.items.length).toBeGreaterThan(0)
      }
    }
  })

  it('every setting item has required fields', () => {
    for (const category of settingsConfig) {
      for (const section of category.sections) {
        for (const item of section.items) {
          expect(item.key).toBeTruthy()
          expect(item.type).toMatch(/^(toggle|select|accordion|link|action|text)$/)
          expect(item.label).toBeTruthy()
          expect(item.description).toBeTruthy()
          expect(Array.isArray(item.synonyms)).toBe(true)
        }
      }
    }
  })

  it('has unique keys across all items', () => {
    const keys: string[] = []
    for (const category of settingsConfig) {
      for (const section of category.sections) {
        for (const item of section.items) {
          keys.push(item.key)
          if (item.children) {
            for (const child of item.children) {
              keys.push(child.key)
            }
          }
        }
      }
    }
    const uniqueKeys = new Set(keys)
    expect(uniqueKeys.size).toBe(keys.length)
  })

  it('select items have options array', () => {
    for (const category of settingsConfig) {
      for (const section of category.sections) {
        for (const item of section.items) {
          if (item.type === 'select') {
            expect(item.options).toBeTruthy()
            expect(item.options!.length).toBeGreaterThan(0)
          }
        }
      }
    }
  })

  it('accordion items have children', () => {
    for (const category of settingsConfig) {
      for (const section of category.sections) {
        for (const item of section.items) {
          if (item.type === 'accordion') {
            expect(item.children).toBeTruthy()
            expect(item.children!.length).toBeGreaterThan(0)
          }
        }
      }
    }
  })

  it('has no quickAccess items', () => {
    let quickAccessCount = 0
    for (const category of settingsConfig) {
      for (const section of category.sections) {
        for (const item of section.items) {
          if (item.quickAccess) quickAccessCount++
        }
      }
    }
    expect(quickAccessCount).toBe(0)
  })
})
