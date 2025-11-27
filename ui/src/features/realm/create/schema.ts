import { z } from 'zod'

export const formSchema = z.object({
  name: z
    .string()
    .min(3, { message: 'Realm name must be at least 3 characters.' })
    .max(30, { message: 'Realm name must be less than 30 characters.' })
    .regex(/^[a-z0-9-]+$/, {
      message: 'Only lowercase letters, numbers, and hyphens allowed.',
    }),
})

export type FormValues = z.infer<typeof formSchema>
