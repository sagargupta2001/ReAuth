export type RoleCompositeFilter = 'all' | 'direct' | 'effective' | 'unassigned'

export const roleCompositeFilterOptions = [
  { value: 'all', label: 'All' },
  { value: 'direct', label: 'Direct' },
  { value: 'effective', label: 'Inherited' },
  { value: 'unassigned', label: 'Unassigned' },
] satisfies Array<{ value: RoleCompositeFilter; label: string }>
