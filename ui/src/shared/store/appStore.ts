import { create } from 'zustand'

interface AppState {
  hasHydrated: boolean
  setHydrated: (v: boolean) => void
  i18nReady: boolean
  setI18nReady: (v: boolean) => void
}

export const useAppStore = create<AppState>((set) => ({
  hasHydrated: false,
  setHydrated: (v) => set({ hasHydrated: v }),
  i18nReady: false,
  setI18nReady: (v) => set({ i18nReady: v }),
}))
