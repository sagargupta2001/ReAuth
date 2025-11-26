import { z } from 'zod'

export const createClientSchema = z.object({
  client_id: z
    .string()
    .min(3, 'Client ID must be at least 3 characters')
    .regex(/^[a-z0-9-]+$/, 'Only lowercase letters, numbers, and hyphens allowed'),

  // We use an array of objects for the form state (better for useFieldArray)
  redirect_uris: z
    .array(
      z.object({
        value: z.string().url({ message: 'Must be a valid URL (http/https)' }),
      }),
    )
    .min(1, { message: 'At least one redirect URI is required' }),
})

export type CreateClientSchema = z.infer<typeof createClientSchema>
