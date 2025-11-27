import { ArrowLeft } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { RealmLink } from '@/entities/realm/lib/navigation.tsx'
import { CreateClientForm } from '@/features/client/create/CreateClientForm.tsx'
import { cn } from '@/lib/utils.ts'
import { buttonVariants } from '@/shared/ui/button.tsx'

export function CreateClientPage() {
  const { t } = useTranslation('client')
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
          {t('FORMS.CREATE_CLIENT.BACK_TO_CLIENTS_BUTTON')}
        </RealmLink>
      </div>
      <CreateClientForm />
    </div>
  )
}
