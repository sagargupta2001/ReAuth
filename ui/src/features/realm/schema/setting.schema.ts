import { z } from 'zod'

export const tokenSettingsSchema = z.object({
  // We coerce strings to numbers because HTML inputs return strings by default
  access_token_ttl_secs: z.coerce
    .number()
    .min(60, 'Must be at least 60 seconds (1 minute)')
    .max(86400, 'Max 24 hours'),
  refresh_token_ttl_secs: z.coerce.number().min(3600, 'Must be at least 3600 seconds (1 hour)'),
  pkce_required_public_clients: z.boolean(),
  lockout_threshold: z.coerce
    .number()
    .min(0, 'Use 0 to disable lockout.')
    .max(50, 'Must be 50 or less'),
  lockout_duration_secs: z.coerce
    .number()
    .min(0, 'Use 0 to disable lockout.')
    .max(86400, 'Max 24 hours'),
})

export const generalSettingsSchema = z.object({
  name: z
    .string()
    .min(3, { message: 'Realm name must be at least 3 characters.' })
    .max(30, { message: 'Realm name must be less than 30 characters.' })
    .regex(/^[a-z0-9-]+$/, {
      message: 'Only lowercase letters, numbers, and hyphens allowed.',
    }),
})

export type GeneralSettingsSchema = z.infer<typeof generalSettingsSchema>
export type TokenSettingsSchema = z.infer<typeof tokenSettingsSchema>
