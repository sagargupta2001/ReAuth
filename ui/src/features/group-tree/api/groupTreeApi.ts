import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { apiClient } from '@/shared/api/client'
import type { GroupTreeNode } from '@/features/group-tree/model/types'

export interface GroupTreeQueryParams {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: 'asc' | 'desc'
}

export interface MoveGroupPayload {
  parent_id?: string | null
  before_id?: string | null
  after_id?: string | null
}

export async function fetchGroupRoots(
  realm: string,
  params: GroupTreeQueryParams,
): Promise<PaginatedResponse<GroupTreeNode>> {
  const query = new URLSearchParams()
  query.set('page', String(params.page || 1))
  query.set('per_page', String(params.per_page || 200))
  if (params.q) query.set('q', params.q)
  if (params.sort_by) query.set('sort_by', params.sort_by)
  if (params.sort_dir) query.set('sort_dir', params.sort_dir)

  return apiClient.get<PaginatedResponse<GroupTreeNode>>(
    `/api/realms/${realm}/rbac/groups/tree?${query.toString()}`,
  )
}

export async function fetchGroupChildren(
  realm: string,
  groupId: string,
  params: GroupTreeQueryParams,
): Promise<PaginatedResponse<GroupTreeNode>> {
  const query = new URLSearchParams()
  query.set('page', String(params.page || 1))
  query.set('per_page', String(params.per_page || 200))
  if (params.q) query.set('q', params.q)
  if (params.sort_by) query.set('sort_by', params.sort_by)
  if (params.sort_dir) query.set('sort_dir', params.sort_dir)

  return apiClient.get<PaginatedResponse<GroupTreeNode>>(
    `/api/realms/${realm}/rbac/groups/${groupId}/children?${query.toString()}`,
  )
}

export async function moveGroup(
  realm: string,
  groupId: string,
  payload: MoveGroupPayload,
) {
  return apiClient.post(`/api/realms/${realm}/rbac/groups/${groupId}/move`, payload)
}
