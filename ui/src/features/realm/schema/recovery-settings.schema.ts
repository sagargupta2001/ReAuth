import { z } from 'zod'

export const recoverySettingsSchema = z.object({
  token_ttl_minutes: z.coerce
    .number()
    .int()
    .min(5, 'Minimum 5 minutes')
    .max(1440, 'Maximum 1440 minutes (24h)'),
  rate_limit_max: z.coerce.number().int().min(0, 'Use 0 to disable rate limits').max(50),
  rate_limit_window_minutes: z.coerce
    .number()
    .int()
    .min(1, 'Minimum 1 minute')
    .max(120, 'Maximum 120 minutes'),
  revoke_sessions_on_reset: z.boolean(),
  email_subject: z.string().optional().or(z.literal('')),
  email_body: z.string().optional().or(z.literal('')),
})

export type RecoverySettingsSchema = z.infer<typeof recoverySettingsSchema>
