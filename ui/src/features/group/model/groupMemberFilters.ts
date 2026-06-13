export type GroupMemberFilter = 'all' | 'members' | 'non-members'

export const groupMemberFilterOptions = [
  { value: 'all', label: 'All' },
  { value: 'members', label: 'Members' },
  { value: 'non-members', label: 'Not Members' },
] satisfies Array<{ value: GroupMemberFilter; label: string }>
