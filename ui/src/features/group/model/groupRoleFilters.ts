export type GroupRoleFilter = 'all' | 'direct' | 'effective' | 'unassigned'

export const groupRoleFilterOptions = [
  { value: 'all', label: 'All' },
  { value: 'direct', label: 'Direct' },
  { value: 'effective', label: 'Composite' },
  { value: 'unassigned', label: 'Unassigned' },
] satisfies Array<{ value: GroupRoleFilter; label: string }>
