import type { Session, SessionStatus, SessionType } from '@/entities/session/model/types'

// Thresholds for the derived status badges (presentational only; see spec).
export const EXPIRING_SOON_MS = 60 * 60 * 1000 // 1 hour
export const IDLE_MS = 24 * 60 * 60 * 1000 // 24 hours

/**
 * A session is a Browser (cookie/SSO root) session when it has no OAuth client,
 * otherwise it is an OAuth client session.
 */
export function deriveSessionType(session: Pick<Session, 'client_id'>): SessionType {
  return session.client_id ? 'oauth' : 'browser'
}

export function sessionTypeLabel(session: Pick<Session, 'client_id'>): string {
  return deriveSessionType(session) === 'oauth' ? 'OAuth Client' : 'Browser'
}

/**
 * Classify a session for the status badge. Precedence (highest first):
 * current > reauth_pending > expiring_soon > idle > active.
 */
export function deriveSessionStatus(
  session: Session,
  currentSessionId: string | undefined,
  now: number = Date.now(),
): SessionStatus {
  if (currentSessionId && session.id === currentSessionId) return 'current'
  if (session.step_up_at) return 'reauth_pending'

  const expiresAt = new Date(session.expires_at).getTime()
  if (!Number.isNaN(expiresAt) && expiresAt - now <= EXPIRING_SOON_MS) {
    return 'expiring_soon'
  }

  const lastUsed = new Date(session.last_used_at).getTime()
  if (!Number.isNaN(lastUsed) && now - lastUsed >= IDLE_MS) {
    return 'idle'
  }

  return 'active'
}

export interface StatusBadge {
  label: string
  variant: 'info' | 'warning' | 'muted' | 'success' | 'orange'
  /** Tooltip clarifying what the status means so it isn't misread. */
  hint: string
}

export function statusBadge(status: SessionStatus): StatusBadge {
  switch (status) {
    case 'current':
      return { label: 'Current', variant: 'info', hint: 'The session you are currently using.' }
    case 'reauth_pending':
      return {
        label: 'Re-auth pending',
        variant: 'orange',
        hint: 'Forced re-authentication requested. The session must verify again on its next refresh.',
      }
    case 'expiring_soon':
      return {
        label: 'Expiring soon',
        variant: 'warning',
        hint: 'This session expires within the next hour.',
      }
    case 'idle':
      return {
        label: 'Idle',
        variant: 'muted',
        hint: 'No activity in the last 24 hours.',
      }
    case 'active':
    default:
      return { label: 'Active', variant: 'success', hint: 'Active session with recent activity.' }
  }
}

export interface DeviceInfo {
  os: string
  browser: string
  /** e.g. "macOS • Chrome" or "Unknown device" */
  label: string
}

/**
 * Best-effort parse of a User-Agent string into OS + browser. Intentionally
 * lightweight (no dependency); falls back to "Unknown device" when unparseable.
 */
export function parseUserAgent(ua: string | undefined | null): DeviceInfo {
  if (!ua || !ua.trim()) {
    return { os: 'Unknown', browser: 'Unknown', label: 'Unknown device' }
  }

  const os = detectOs(ua)
  const browser = detectBrowser(ua)

  if (os === 'Unknown' && browser === 'Unknown') {
    return { os, browser, label: 'Unknown device' }
  }
  if (browser === 'Unknown') return { os, browser, label: os }
  if (os === 'Unknown') return { os, browser, label: browser }
  return { os, browser, label: `${os} • ${browser}` }
}

function detectOs(ua: string): string {
  if (/windows nt/i.test(ua)) return 'Windows'
  if (/iphone|ipad|ipod/i.test(ua)) return 'iOS'
  if (/mac os x|macintosh/i.test(ua)) return 'macOS'
  if (/android/i.test(ua)) return 'Android'
  if (/cros/i.test(ua)) return 'ChromeOS'
  if (/linux/i.test(ua)) return 'Linux'
  return 'Unknown'
}

function detectBrowser(ua: string): string {
  // Order matters: Edge/Opera/Brave masquerade as Chrome; check them first.
  if (/edg(e|a|ios)?\//i.test(ua)) return 'Edge'
  if (/opr\/|opera/i.test(ua)) return 'Opera'
  if (/firefox\/|fxios\//i.test(ua)) return 'Firefox'
  if (/chrome\/|crios\//i.test(ua)) return 'Chrome'
  if (/safari\//i.test(ua) && /version\//i.test(ua)) return 'Safari'
  if (/reauth|curl|wget|postman|insomnia|python-requests|go-http/i.test(ua)) return 'CLI / Tool'
  return 'Unknown'
}

export function isMobileUserAgent(ua: string | undefined | null): boolean {
  if (!ua) return false
  return /mobile|android|iphone|ipod/i.test(ua)
}
