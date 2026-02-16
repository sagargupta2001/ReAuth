import { create } from 'zustand'

interface GroupTreeStoreState {
  expandedByRealm: Record<string, string[]>
  setExpanded: (realm: string, ids: string[]) => void
  toggleExpanded: (realm: string, id: string) => void
  resetExpanded: (realm: string) => void
}

export const useGroupTreeStore = create<GroupTreeStoreState>((set, get) => ({
  expandedByRealm: {},
  setExpanded: (realm, ids) =>
    set((state) => ({
      expandedByRealm: { ...state.expandedByRealm, [realm]: ids },
    })),
  toggleExpanded: (realm, id) => {
    const current = new Set(get().expandedByRealm[realm] ?? [])
    if (current.has(id)) {
      current.delete(id)
    } else {
      current.add(id)
    }
    set((state) => ({
      expandedByRealm: { ...state.expandedByRealm, [realm]: Array.from(current) },
    }))
  },
  resetExpanded: (realm) =>
    set((state) => ({
      expandedByRealm: { ...state.expandedByRealm, [realm]: [] },
    })),
}))
