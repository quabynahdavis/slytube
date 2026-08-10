import { defineStore } from 'pinia'

export interface Subscription {
  id: string
  name: string
  thumbnail?: string
}

export interface Profile {
  _id: string
  name: string
  bgColor: string
  textColor: string
  subscriptions: Subscription[]
}

export interface ProfilesState {
  profileList: Profile[]
  activeProfile: string
}

const MAIN_PROFILE_ID = 'allChannels'

export const useProfilesStore = defineStore('profiles', {
  state: (): ProfilesState => ({
    profileList: [
      {
        _id: MAIN_PROFILE_ID,
        name: 'All Channels',
        bgColor: '#000000',
        textColor: '#FFFFFF',
        subscriptions: [],
      },
    ],
    activeProfile: MAIN_PROFILE_ID,
  }),

  getters: {
    getProfileList: (state) => state.profileList,

    getActiveProfile: (state) => {
      return state.profileList.find((profile) => profile._id === state.activeProfile)
    },

    profileById: (state) => (id: string) => {
      return state.profileList.find((p) => p._id === id)
    },

    getSubscribedChannelIdSet: (state) => {
      const mainProfile = state.profileList[0]
      return mainProfile.subscriptions.reduce(
        (set, channel) => set.add(channel.id),
        new Set<string>()
      )
    },

    getSubscribedChannelsById: (state) => {
      const mainProfile = state.profileList[0]
      return new Map(mainProfile.subscriptions.map((channel) => [channel.id, channel]))
    },
  },

  actions: {
    loadProfiles() {
      // Placeholder for DB integration
    },

    createProfile(name: string) {
      const profile: Profile = {
        _id: `profile--${Date.now()}-${Math.floor(Math.random() * 10000)}`,
        name,
        bgColor: '#000000',
        textColor: '#FFFFFF',
        subscriptions: [],
      }
      this.profileList.push(profile)
      this.sortProfiles()
      return profile
    },

    deleteProfile(id: string) {
      const index = this.profileList.findIndex((p) => p._id === id)
      if (index !== -1) {
        this.profileList.splice(index, 1)
      }
    },

    setActiveProfile(id: string) {
      this.activeProfile = id
    },

    addSubscription(profileId: string, channel: Subscription) {
      const profile = this.profileList.find((p) => p._id === profileId)
      if (profile && !profile.subscriptions.some((sub) => sub.id === channel.id)) {
        profile.subscriptions.push(channel)
      }
    },

    removeSubscription(profileId: string, channelId: string) {
      const profile = this.profileList.find((p) => p._id === profileId)
      if (profile) {
        profile.subscriptions = profile.subscriptions.filter((ch) => ch.id !== channelId)
      }
    },

    sortProfiles() {
      const collator = new Intl.Collator(undefined, {
        usage: 'sort',
        caseFirst: 'upper',
        sensitivity: 'case',
        numeric: true,
      })

      this.profileList.sort((a, b) => {
        if (a._id === MAIN_PROFILE_ID) return -1
        if (b._id === MAIN_PROFILE_ID) return 1
        return collator.compare(a.name.normalize('NFC'), b.name.normalize('NFC'))
      })
    },
  },
})
