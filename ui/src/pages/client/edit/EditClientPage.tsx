import { ArrowLeft } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useParams } from 'react-router-dom'

import { buttonVariants } from '@/components/button'
import { RealmLink } from '@/entities/realm/lib/navigation'
import { EditClientForm } from '@/features/client/forms/EditClientForm.tsx'
import { cn } from '@/lib/utils'

export function EditClientPage() {
  const { t } = useTranslation('client')

  const { clientId } = useParams<{ clientId: string }>()

  if (!clientId) return null

  return (
    <div className="w-full py-6">
      <div className="mb-2">
        <RealmLink
          to="/clients"
          className={cn(
            buttonVariants({ variant: 'link', size: 'sm' }),
            'text-muted-foreground hover:text-foreground gap-2 pl-0',
          )}
        >
          <ArrowLeft className="h-4 w-4" />
          {t('FORMS.EDIT_CLIENT.BACK_TO_CLIENTS_BUTTON')}
        </RealmLink>
      </div>
      <EditClientForm clientId={clientId} />
    </div>
  )
}
