import { json } from '@codemirror/lang-json'
import { oneDark } from '@codemirror/theme-one-dark'
import CodeMirror from '@uiw/react-codemirror'
import { Copy } from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/dialog'
import type { User } from '@/entities/user/model/types'
import { Separator } from '@/shared/ui/separator'

interface UserJsonDialogProps {
  user?: User
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function UserJsonDialog({ user, open, onOpenChange }: UserJsonDialogProps) {
  const value = JSON.stringify(user ?? {}, null, 2)

  const copyJson = () => {
    navigator.clipboard
      .writeText(value)
      .then(() => toast.success('User JSON copied.'))
      .catch(() => toast.error('Failed to copy user JSON.'))
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[720px]">
        <DialogHeader className="pt-6 pl-6">
          <DialogTitle>User JSON</DialogTitle>
        </DialogHeader>
        <Separator className="my-1" />

        <div className="px-6 pb-6">
          <div className="overflow-hidden rounded-lg border">
            <CodeMirror
              value={value}
              height="420px"
              theme={oneDark}
              extensions={[json()]}
              basicSetup={{ foldGutter: true, lineNumbers: true }}
              editable={false}
            />
          </div>
        </div>

        <DialogFooter className="gap-1 py-3 pr-3">
          <Button variant="outline" type="button" onClick={() => onOpenChange(false)}>
            Close
          </Button>
          <Button size="sm" onClick={copyJson}>
            <Copy className="mr-2 h-4 w-4" />
            Copy JSON
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
