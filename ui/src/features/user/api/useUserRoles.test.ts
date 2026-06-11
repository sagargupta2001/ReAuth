import { describe, expect, it } from 'vitest'

import type { PaginatedResponse } from '@/entities/oidc/model/types'
import { getNextUserRolesPageParam, type UserRoleRow } from './useUserRoles'

function page(pageNumber: number, totalPages: number): PaginatedResponse<UserRoleRow> {
  return {
    data: [],
    meta: {
      total: 100,
      page: pageNumber,
      per_page: 25,
      total_pages: totalPages,
    },
  }
}

describe('getNextUserRolesPageParam', () => {
  it('returns the next page when more role pages exist', () => {
    expect(getNextUserRolesPageParam(page(2, 4))).toBe(3)
  })

  it('returns undefined on the last role page', () => {
    expect(getNextUserRolesPageParam(page(4, 4))).toBeUndefined()
  })
})
