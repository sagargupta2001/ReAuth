export interface User {
  id: string
  username: string
  email?: string | null
  created_at?: string
  last_sign_in_at?: string | null
}
