import { z } from 'zod'

export const createClientSchema = z.object({
  client_id: z
    .string()
    .min(3, 'Client ID must be at least 3 characters')
    .regex(/^[a-z0-9-]+$/, 'Only lowercase letters, numbers, and hyphens allowed'),

  // We accept a string from the Textarea, but validate it as a list of URLs
  redirect_uris: z.string().refine(
    (val) => {
      const urls = val
        .split('\n')
        .map((s) => s.trim())
        .filter(Boolean)
      if (urls.length === 0) return false
      // Simple URL validation
      return urls.every((u) => u.startsWith('http'))
    },
    {
      message: 'Must provide at least one valid URL (one per line), starting with http/https',
    },
  ),
})

export type CreateClientSchema = z.infer<typeof createClientSchema>
