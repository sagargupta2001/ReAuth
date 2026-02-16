import { create } from 'zustand'

import type { GroupTreeNode } from '@/features/group-tree/model/types'

interface GroupTreeStoreState {
  expandedByRealm: Record<string, string[]>
  treeByRealm: Record<string, GroupTreeNode[]>
  setExpanded: (realm: string, ids: string[]) => void
  toggleExpanded: (realm: string, id: string) => void
  resetExpanded: (realm: string) => void
  setTree: (realm: string, tree: GroupTreeNode[]) => void
  updateTree: (realm: string, updater: (prev: GroupTreeNode[]) => GroupTreeNode[]) => void
  resetTree: (realm: string) => void
}

export const useGroupTreeStore = create<GroupTreeStoreState>((set, get) => ({
  expandedByRealm: {},
  treeByRealm: {},
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
  setTree: (realm, tree) =>
    set((state) => ({
      treeByRealm: { ...state.treeByRealm, [realm]: tree },
    })),
  updateTree: (realm, updater) =>
    set((state) => {
      const prev = state.treeByRealm[realm] ?? []
      const next = updater(prev)
      return {
        treeByRealm: { ...state.treeByRealm, [realm]: next },
      }
    }),
  resetTree: (realm) =>
    set((state) => ({
      treeByRealm: { ...state.treeByRealm, [realm]: [] },
    })),
}))
