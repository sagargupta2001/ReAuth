import { jwtDecode } from 'jwt-decode'
import { create } from 'zustand'

// This is the shape of the *payload* inside your Access Token (JWT)
// We get this by decoding the token
export interface AuthUser {
  sub: string // The User ID
  sid: string // The Session ID
  perms: string[] // The user's permissions
  exp: number
}

interface AuthState {
  user: AuthUser | null
  accessToken: string | null

  // Action to set the full session (called after login)
  setSession: (token: string) => void

  // Action to clear the session (called on logout)
  clearSession: () => void
}

export const useSessionStore = create<AuthState>((set) => ({
  user: null,
  accessToken: null,

  setSession: (token: string) => {
    try {
      // Decode the JWT to get the user payload
      const user = jwtDecode<AuthUser>(token)
      set({ accessToken: token, user })
    } catch (e) {
      console.error('Failed to decode JWT:', e)
      set({ accessToken: null, user: null })
    }
  },

  clearSession: () => {
    set({ accessToken: null, user: null })
  },
}))
