export const queryKeys = {
  setupStatus: () => ['setup-status'] as const,
  user: (userId: string) => ['user', userId] as const,
  users: (realm?: string, params?: unknown) =>
    realm ? (['users', realm, params] as const) : (['users'] as const),
  client: (realm: string, clientId: string) => ['client', realm, clientId] as const,
  clients: (realm?: string, params?: unknown) =>
    realm ? (['clients', realm, params] as const) : (['clients'] as const),
  role: (realm: string, roleId: string) => ['role', realm, roleId] as const,
  roles: (realm: string, params?: unknown) => ['roles', realm, params] as const,
  group: (realm: string, groupId: string) => ['group', realm, groupId] as const,
  groups: (realm?: string, params?: unknown) =>
    realm ? (['groups', realm, params] as const) : (['groups'] as const),
  groupChildren: (realm: string, groupId?: string, params?: unknown) =>
    groupId
      ? (['group-children', realm, groupId, params] as const)
      : (['group-children', realm] as const),
  groupMembers: (realm: string, groupId: string) => ['group-members', realm, groupId] as const,
  groupMemberList: (realm: string, groupId: string, params?: unknown) =>
    ['group-member-list', realm, groupId, params] as const,
  groupRoles: (realm: string, groupId: string, scope?: string) =>
    ['group-roles', realm, groupId, scope] as const,
  groupRoleList: (realm: string, groupId: string, params?: unknown) =>
    ['group-role-list', realm, groupId, params] as const,
  groupDeleteSummary: (realm: string, groupId: string) =>
    ['group-delete-summary', realm, groupId] as const,
  sessions: (realm?: string, params?: unknown) =>
    realm ? (['sessions', realm, params] as const) : (['sessions'] as const),
  userRoles: (realm: string, userId: string, scope?: string) =>
    ['user-roles', realm, userId, scope] as const,
  userRoleList: (realm: string, userId: string, params?: unknown) =>
    ['user-role-list', realm, userId, params] as const,
  roleMembers: (realm: string, roleId: string, scope?: string) =>
    ['role-members', realm, roleId, scope] as const,
  roleMemberList: (realm: string, roleId: string, params?: unknown) =>
    ['role-member-list', realm, roleId, params] as const,
  roleComposites: (realm: string, roleId: string, scope?: string) =>
    ['role-composites', realm, roleId, scope] as const,
  roleCompositeList: (realm: string, roleId: string, params?: unknown) =>
    ['role-composite-list', realm, roleId, params] as const,
  rolePermissions: (realm: string, roleId: string) => ['role-permissions', realm, roleId] as const,
  permissionsDefinitions: (realm: string, clientId?: string | null) =>
    ['permissions-definitions', realm, clientId ?? null] as const,
  flows: (realm?: string) => (realm ? (['flows', realm] as const) : (['flows'] as const)),
  flow: (flowId?: string) => (flowId ? (['flow', flowId] as const) : (['flow'] as const)),
  flowDraft: (realm?: string, flowId?: string) =>
    flowId ? (['flow-draft', realm, flowId] as const) : (['flow-draft'] as const),
  flowDrafts: (realm?: string) =>
    realm ? (['flow-drafts', realm] as const) : (['flow-drafts'] as const),
  flowVersions: (flowId?: string) =>
    flowId ? (['flow-versions', flowId] as const) : (['flow-versions'] as const),
  flowNodes: (realm?: string) =>
    realm ? (['flow-nodes', realm] as const) : (['flow-nodes'] as const),
  harborJobs: (realm: string, limit?: number) => ['harbor-jobs', realm, limit] as const,
  harborJobDetails: (realm: string, jobId: string) =>
    ['harbor-job-details', realm, jobId] as const,
  observabilityLogs: (params?: unknown) => ['observability-logs', params] as const,
  observabilityTraces: (params?: unknown) => ['observability-traces', params] as const,
  observabilityTraceSpans: (traceId: string) => ['observability-trace-spans', traceId] as const,
  observabilityMetrics: () => ['observability-metrics'] as const,
  observabilityCacheStats: (namespace?: string) =>
    ['observability-cache-stats', namespace ?? 'all'] as const,
  observabilityLogTargets: (params?: unknown) => ['observability-log-targets', params] as const,
  eventRoutingMetrics: (realm: string, windowHours: number) =>
    ['event-routing-metrics', realm, windowHours] as const,
  webhookDeliveries: (realm?: string, endpointId?: string, params?: unknown) =>
    realm && endpointId
      ? (['webhook-deliveries', realm, endpointId, params] as const)
      : (['webhook-deliveries'] as const),
  webhookDeliveryLogs: () => ['delivery-logs'] as const,
  themes: (realm: string, themeId?: string) =>
    themeId ? (['themes', realm, themeId] as const) : (['themes', realm] as const),
  themeVersions: (realm: string, themeId: string) =>
    ['themes', realm, themeId, 'versions'] as const,
  themeVersionSnapshot: (realm: string, themeId: string, versionId: string) =>
    ['themes', realm, themeId, 'versions', versionId, 'snapshot'] as const,
  themeAssets: (realm: string, themeId: string) =>
    ['themes', realm, themeId, 'assets'] as const,
  themeDraft: (realm: string, themeId: string) => ['themes', realm, themeId, 'draft'] as const,
  themePreview: (realm: string, themeId: string, pageKey?: string, nodeKey?: string) =>
    ['theme-preview', realm, themeId, pageKey, nodeKey] as const,
  themeSnapshot: (
    realm: string,
    params?: { pageKey?: string; nodeKey?: string; clientId?: string | null },
  ) => ['theme-snapshot', realm, params?.pageKey, params?.nodeKey, params?.clientId] as const,
  themePages: (realm: string, themeId: string) => ['theme-pages', realm, themeId] as const,
  themeBindings: (realm: string, themeId: string) =>
    ['theme-bindings', realm, themeId] as const,
  themeBindingClient: (realm: string, clientId: string) =>
    ['theme-bindings', 'client', realm, clientId] as const,
  themeTemplateGaps: (realm: string, themeId: string) =>
    ['theme-template-gaps', realm, themeId] as const,
  activeTheme: (realm: string) => ['active-theme', realm] as const,
  realms: () => ['realms'] as const,
  realm: (realmName?: string) => (realmName ? (['realm', realmName] as const) : (['realm'] as const)),
  realmEmailSettings: (realmId?: string) =>
    realmId ? (['realm-email-settings', realmId] as const) : (['realm-email-settings'] as const),
  realmRecoverySettings: (realmId?: string) =>
    realmId
      ? (['realm-recovery-settings', realmId] as const)
      : (['realm-recovery-settings'] as const),
  realmSecurityHeaders: (realmId?: string) =>
    realmId
      ? (['realm-security-headers', realmId] as const)
      : (['realm-security-headers'] as const),
  realmBindings: () => ['realm-bindings'] as const,
  omniSearch: (realm: string, query?: string, limit?: number) =>
    query !== undefined
      ? (['omni-search', realm, query, limit] as const)
      : (['omni-search', realm] as const),
  webhooks: (realm: string, params?: unknown) =>
    params ? (['webhooks', realm, params] as const) : (['webhooks', realm] as const),
  webhooksById: (realm: string, id: string) => ['webhooks', realm, id] as const,
}
