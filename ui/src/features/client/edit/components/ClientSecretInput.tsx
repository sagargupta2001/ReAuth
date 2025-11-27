import { useState } from 'react'

import { Check, Copy, Eye, EyeOff } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Label } from '@/components/label'

export function ClientSecretInput({ secret }: { secret?: string | null }) {
  const { t } = useTranslation('client')

  const [show, setShow] = useState(false)
  const [copied, setCopied] = useState(false)

  const handleCopy = () => {
    if (!secret) return
    void navigator.clipboard.writeText(secret)
    setCopied(true)
    toast.success('Client secret copied to clipboard')

    setTimeout(() => setCopied(false), 1000)
  }

  if (!secret) return null

  return (
    <div className="space-y-2">
      <Label>{t('FORMS.EDIT_CLIENT.FIELDS.CLIENT_SECRET')}</Label>

      <div className="flex gap-2">
        <div className="relative flex-1">
          <Input
            readOnly
            type={show ? 'text' : 'password'}
            value={secret}
            className="pr-10 font-mono text-sm"
          />

          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="text-muted-foreground hover:text-foreground absolute top-0 right-0 h-full px-3"
            onClick={() => setShow(!show)}
          >
            {show ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            <span className="sr-only">Toggle secret visibility</span>
          </Button>
        </div>

        <Button
          variant="outline"
          size="icon"
          type="button"
          onClick={handleCopy}
          disabled={copied}
          className={`relative transition-all ${!copied ? 'hover:bg-accent hover:scale-105' : ''} ${copied ? 'animate-copyPulse theme-copy' : ''} `}
        >
          {copied ? (
            <Check className="text-accent-foreground h-4 w-4" />
          ) : (
            <Copy className="h-4 w-4" />
          )}
          <span className="sr-only">Copy secret</span>
        </Button>
      </div>

      <p className="text-muted-foreground text-[0.8rem]">
        {t('FORMS.EDIT_CLIENT.FIELDS.CLIENT_SECRET_HELPER_TEXT')}
      </p>
    </div>
  )
}
