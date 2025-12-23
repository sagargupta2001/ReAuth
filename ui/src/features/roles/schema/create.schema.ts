import { z } from 'zod'

export const roleSchema = z.object({
  name: z
    .string()
    .min(2, 'Role name must be at least 2 characters')
    .regex(/^[a-z0-9_]+$/, 'Role name must be lowercase, numbers, or underscores'),
  description: z.string().optional(),
})

export type RoleFormValues = z.infer<typeof roleSchema>
