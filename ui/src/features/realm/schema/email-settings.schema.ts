import { z } from 'zod'

export const emailSettingsSchema = z.object({
  enabled: z.boolean(),
  from_address: z.string().email('Enter a valid email address').optional().or(z.literal('')),
  from_name: z.string().optional().or(z.literal('')),
  reply_to_address: z.string().email('Enter a valid email address').optional().or(z.literal('')),
  smtp_host: z.string().optional().or(z.literal('')),
  smtp_port: z.coerce.number().int().min(1).max(65535).optional(),
  smtp_username: z.string().optional().or(z.literal('')),
  smtp_password: z.string().optional().or(z.literal('')),
  smtp_security: z.enum(['starttls', 'tls', 'none']),
})

export type EmailSettingsSchema = z.infer<typeof emailSettingsSchema>
