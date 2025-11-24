import { create } from 'zustand'

interface SidebarState {
  activeItemId: string | null // The title of the item expanding the secondary bar
  setActiveItem: (id: string | null) => void
}

export const useSidebarStore = create<SidebarState>((set) => ({
  activeItemId: null, // Default to closed (or set 'Settings' to default open)
  setActiveItem: (id) => set({ activeItemId: id }),
}))
