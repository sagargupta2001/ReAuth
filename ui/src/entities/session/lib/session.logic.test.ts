import { describe, expect, it } from 'vitest'

import {
  deriveSessionStatus,
  deriveSessionType,
  parseUserAgent,
  sessionTypeLabel,
} from '@/entities/session/lib/session.logic'
import type { Session } from '@/entities/session/model/types'

const NOW = new Date('2026-06-13T12:00:00Z').getTime()

function makeSession(overrides: Partial<Session> = {}): Session {
  return {
    id: 'sess-1',
    user_id: 'user-1',
    realm_id: 'realm-1',
    created_at: new Date(NOW - 60 * 60 * 1000).toISOString(),
    last_used_at: new Date(NOW - 60 * 1000).toISOString(),
    expires_at: new Date(NOW + 7 * 24 * 60 * 60 * 1000).toISOString(),
    ...overrides,
  }
}

describe('deriveSessionType', () => {
  it('treats a null client_id as a browser session', () => {
    expect(deriveSessionType(makeSession())).toBe('browser')
    expect(sessionTypeLabel(makeSession())).toBe('Browser')
  })

  it('treats a client_id as an OAuth client session', () => {
    expect(deriveSessionType(makeSession({ client_id: 'app' }))).toBe('oauth')
    expect(sessionTypeLabel(makeSession({ client_id: 'app' }))).toBe('OAuth Client')
  })
})

describe('deriveSessionStatus', () => {
  it('marks the caller current session', () => {
    expect(deriveSessionStatus(makeSession(), 'sess-1', NOW)).toBe('current')
  })

  it('prioritizes reauth_pending when step_up_at is set (non-current)', () => {
    const s = makeSession({ step_up_at: new Date(NOW).toISOString() })
    expect(deriveSessionStatus(s, 'other', NOW)).toBe('reauth_pending')
  })

  it('flags expiring_soon within one hour', () => {
    const s = makeSession({ expires_at: new Date(NOW + 30 * 60 * 1000).toISOString() })
    expect(deriveSessionStatus(s, 'other', NOW)).toBe('expiring_soon')
  })

  it('flags idle when not used within 24h', () => {
    const s = makeSession({ last_used_at: new Date(NOW - 25 * 60 * 60 * 1000).toISOString() })
    expect(deriveSessionStatus(s, 'other', NOW)).toBe('idle')
  })

  it('falls back to active', () => {
    expect(deriveSessionStatus(makeSession(), 'other', NOW)).toBe('active')
  })
})

describe('parseUserAgent', () => {
  it('returns Unknown device for missing UA', () => {
    expect(parseUserAgent(undefined).label).toBe('Unknown device')
    expect(parseUserAgent('').label).toBe('Unknown device')
  })

  it('parses macOS + Chrome', () => {
    const ua =
      'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36'
    const info = parseUserAgent(ua)
    expect(info.os).toBe('macOS')
    expect(info.browser).toBe('Chrome')
    expect(info.label).toBe('macOS • Chrome')
  })

  it('parses iOS + Safari', () => {
    const ua =
      'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1'
    const info = parseUserAgent(ua)
    expect(info.os).toBe('iOS')
    expect(info.browser).toBe('Safari')
  })

  it('detects Edge over Chrome', () => {
    const ua =
      'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36 Edg/124.0'
    const info = parseUserAgent(ua)
    expect(info.os).toBe('Windows')
    expect(info.browser).toBe('Edge')
  })
})
