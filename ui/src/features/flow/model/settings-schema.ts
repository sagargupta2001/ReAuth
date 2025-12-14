import { z } from 'zod'

export const flowSettingsSchema = z.object({
  name: z
    .string()
    .min(3, { message: 'Flow name must be at least 3 characters.' })
    .max(50, { message: 'Flow name cannot exceed 50 characters.' })
    .regex(/^[a-zA-Z0-9-\s]+$/, {
      message: 'Name can only contain letters, numbers, spaces, and hyphens.',
    }),
  description: z.string().max(255).optional(),
})

export type FlowSettingsSchema = z.infer<typeof flowSettingsSchema>
