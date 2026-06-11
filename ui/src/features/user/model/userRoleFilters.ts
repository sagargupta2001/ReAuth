import type { UserRoleFilter } from '@/features/user/api/useUserRoles'

export const userRoleFilterOptions = [
  { value: 'all', label: 'All' },
  { value: 'direct', label: 'Direct' },
  { value: 'effective', label: 'Effective' },
  { value: 'unassigned', label: 'Unassigned' },
] satisfies Array<{ value: UserRoleFilter; label: string }>
