declare module 'shaka-player/dist/shaka-player.compiled' {
  interface PlayerStatic {
    new (mediaElement?: HTMLMediaElement | null): any
    isBrowserSupported(): boolean
  }

  const shaka: {
    Player: PlayerStatic
    polyfill: {
      installAll(): void
    }
    extern: {
      Track: any
    }
  }
  export default shaka
}
