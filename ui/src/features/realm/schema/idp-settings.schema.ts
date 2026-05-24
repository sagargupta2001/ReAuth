import { z } from 'zod'

export const idpSettingsSchema = z.object({
  oauth_start_rate_limit_max: z.coerce
    .number()
    .int()
    .min(0, 'Use 0 to disable rate limits')
    .max(50),
  oauth_start_rate_limit_window_minutes: z.coerce
    .number()
    .int()
    .min(1, 'Minimum 1 minute')
    .max(120, 'Maximum 120 minutes'),
})

export type IdpSettingsSchema = z.infer<typeof idpSettingsSchema>
