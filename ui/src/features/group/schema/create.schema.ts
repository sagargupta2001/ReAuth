import { z } from 'zod'

export const groupSchema = z.object({
  name: z.string().min(2, 'Group name must be at least 2 characters'),
  description: z.string().optional(),
})

export type GroupFormValues = z.infer<typeof groupSchema>
