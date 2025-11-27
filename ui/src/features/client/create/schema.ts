// schema.ts
import i18n from 'i18next'
import { z } from 'zod'

export const createClientSchema = () =>
  z.object({
    client_id: z
      .string()
      .min(3, {
        message: i18n.t('client:FORMS.CREATE_CLIENT.VALIDATIONS.CLIENT_ID_MIN_THREE_CHARS'),
      })
      .regex(/^[a-z0-9-]+$/, {
        message: i18n.t('client:FORMS.CREATE_CLIENT.VALIDATIONS.CLIENT_ID_REGEX'),
      }),

    redirect_uris: z
      .array(
        z.object({
          value: z.string().url({
            message: i18n.t('client:FORMS.CREATE_CLIENT.VALIDATIONS.VALID_REDIRECT_URI'),
          }),
        }),
      )
      .min(1, {
        message: i18n.t('client:FORMS.CREATE_CLIENT.VALIDATIONS.VALID_REDIRECT_URI_MIN_COUNT'),
      }),
  })

export type CreateClientSchema = z.infer<ReturnType<typeof createClientSchema>>
