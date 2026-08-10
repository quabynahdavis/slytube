import { defineStore } from 'pinia'

export interface InvidiousState {
  currentInvidiousInstance: string
  currentInvidiousInstanceAuthorization: string | null
  currentInvidiousInstanceUrl: string
  invidiousInstancesList: string[] | null
}

export const useInvidiousStore = defineStore('invidious', {
  state: (): InvidiousState => ({
    currentInvidiousInstance: '',
    currentInvidiousInstanceAuthorization: null,
    currentInvidiousInstanceUrl: '',
    invidiousInstancesList: null,
  }),

  getters: {
    getCurrentInvidiousInstance: (state) => state.currentInvidiousInstance,
    getCurrentInvidiousInstanceUrl: (state) => state.currentInvidiousInstanceUrl,
    getCurrentInvidiousInstanceAuthorization: (state) => state.currentInvidiousInstanceAuthorization,
    getInvidiousInstancesList: (state) => state.invidiousInstancesList,
  },

  actions: {
    loadInstances() {
      // Placeholder for fetching instances from file/API
    },

    setInstance(url: string) {
      this.currentInvidiousInstance = url

      let parsedUrl: URL | undefined
      try {
        parsedUrl = new URL(url)
      } catch {
        // invalid URL
      }

      let authorization: string | null = null

      if (parsedUrl && (parsedUrl.username.length > 0 || parsedUrl.password.length > 0)) {
        authorization = `Basic ${btoa(`${parsedUrl.username}:${parsedUrl.password}`)}`
      }

      this.currentInvidiousInstanceAuthorization = authorization

      let instanceUrl: string

      if (parsedUrl && authorization) {
        parsedUrl.username = ''
        parsedUrl.password = ''
        instanceUrl = parsedUrl.toString().replace(/\/$/, '')
      } else {
        instanceUrl = url
      }

      this.currentInvidiousInstanceUrl = instanceUrl
    },

    async testInstance(url: string): Promise<boolean> {
      try {
        const response = await fetch(`${url}/api/v1/stats`)
        return response.ok
      } catch {
        return false
      }
    },

    setInstancesList(instances: string[]) {
      this.invidiousInstancesList = instances
    },
  },
})
