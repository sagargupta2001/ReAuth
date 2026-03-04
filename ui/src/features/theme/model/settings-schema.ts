import { z } from 'zod'

export const themeSettingsSchema = z.object({
  name: z.string().min(1, 'Theme name is required'),
  description: z.string().optional(),
})

export type ThemeSettingsSchema = z.infer<typeof themeSettingsSchema>
