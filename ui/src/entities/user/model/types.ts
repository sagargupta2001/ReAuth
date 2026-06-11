export interface User {
  id: string
  username: string
  first_name?: string | null
  last_name?: string | null
  email?: string | null
  emails?: UserEmail[]
  phone_number?: string | null
  phone_numbers?: UserPhoneNumber[]
  created_at?: string
  updated_at?: string | null
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

export interface UserPhoneNumber {
  id: string
  user_id: string
  realm_id: string
  phone_number: string
  phone_number_normalized: string
  is_primary: boolean
  is_verified: boolean
  created_at: string
  updated_at: string
}
