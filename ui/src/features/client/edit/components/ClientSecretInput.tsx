import { useState } from 'react'

import { Copy, Eye, EyeOff } from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Label } from '@/components/label'

export function ClientSecretInput({ secret }: { secret?: string | null }) {
  const [show, setShow] = useState(false)

  const handleCopy = () => {
    if (!secret) return
    void navigator.clipboard.writeText(secret)
    toast.success('Client secret copied to clipboard')
  }

  // If public client (no secret)
  if (!secret) return null

  return (
    <div className="space-y-2">
      <Label>Client Secret</Label>
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
        <Button variant="outline" size="icon" type="button" onClick={handleCopy}>
          <Copy className="h-4 w-4" />
          <span className="sr-only">Copy secret</span>
        </Button>
      </div>
      <p className="text-muted-foreground text-[0.8rem]">
        Keep this secret secure. Do not share it with public applications.
      </p>
    </div>
  )
}
