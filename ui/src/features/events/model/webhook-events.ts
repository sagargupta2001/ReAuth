export const WEBHOOK_EVENT_GROUPS = [
  {
    id: 'users',
    label: 'Users',
    description: 'Authentication and lifecycle changes',
    events: [
      'user.created',
      'user.updated',
      'user.disabled',
      'user.deleted',
      'user.assigned',
      'user.removed',
    ],
  },
  {
    id: 'roles',
    label: 'Roles',
    description: 'Role assignments and permission changes',
    events: ['role.created', 'role.updated', 'role.assigned', 'role.removed', 'role.deleted'],
  },
  {
    id: 'groups',
    label: 'Groups',
    description: 'Group membership changes',
    events: ['group.created', 'group.updated', 'group.assigned', 'group.removed', 'group.deleted'],
  },
] as const

export const DEFAULT_WEBHOOK_EVENTS = ['user.created', 'user.updated']
