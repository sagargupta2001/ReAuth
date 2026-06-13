export type RoleMemberFilter = 'all' | 'direct' | 'effective' | 'unassigned'

export const roleMemberFilterOptions = [
  { value: 'all', label: 'All' },
  { value: 'direct', label: 'Direct' },
  { value: 'effective', label: 'Via Group' },
  { value: 'unassigned', label: 'Unassigned' },
] satisfies Array<{ value: RoleMemberFilter; label: string }>
