import { z } from 'zod'

const headerSchema = z
  .string()
  .max(4096, 'Maximum 4096 characters')
  .optional()
  .or(z.literal(''))

export const securityHeadersSchema = z.object({
  x_frame_options: headerSchema,
  content_security_policy: headerSchema,
  x_content_type_options: headerSchema,
  referrer_policy: headerSchema,
  strict_transport_security: headerSchema,
})

export type SecurityHeadersSchema = z.infer<typeof securityHeadersSchema>
