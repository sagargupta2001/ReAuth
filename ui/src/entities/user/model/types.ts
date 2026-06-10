export interface User {
  id: string
  username: string
  email?: string | null
  emails?: UserEmail[]
  created_at?: string
  last_sign_in_at?: string | null
}

export interface UserEmail {
  id: string
  user_id: string
  realm_id: string
  email: string
  email_normalized: string
  is_primary: boolean
  is_verified: boolean
  created_at: string
  updated_at: string
}
