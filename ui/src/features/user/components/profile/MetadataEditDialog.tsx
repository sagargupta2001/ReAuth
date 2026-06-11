import { useEffect, useMemo, useState } from 'react'

import { json } from '@codemirror/lang-json'
import { oneDark } from '@codemirror/theme-one-dark'
import CodeMirror from '@uiw/react-codemirror'

import { Button } from '@/components/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/dialog'
import type { JsonObject, UserMetadataVisibility } from '@/entities/user/model/types.ts'
import { useUpdateUserMetadata } from '@/features/user/api/useUserMetadata.ts'
import { Separator } from '@/shared/ui/separator'

interface MetadataEditDialogProps {
  userId: string
  visibility: UserMetadataVisibility
  title: string
  metadata: JsonObject
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function MetadataEditDialog({
  userId,
  visibility,
  title,
  metadata,
  open,
  onOpenChange,
}: MetadataEditDialogProps) {
  const [value, setValue] = useState(formatJson(metadata))
  const mutation = useUpdateUserMetadata(userId)

  useEffect(() => {
    if (open) setValue(formatJson(metadata))
  }, [metadata, open])

  const parsed = useMemo(() => parseJsonObject(value), [value])
  const hasError = Boolean(parsed.error)

  const handleSave = () => {
    if (parsed.error || !parsed.value) return

    mutation.mutate(
      { visibility, metadata: parsed.value },
      {
        onSuccess: () => onOpenChange(false),
      },
    )
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[720px]">
        <DialogHeader className="pt-6 pl-6">
          <DialogTitle>Edit {title}</DialogTitle>
        </DialogHeader>
        <Separator className="my-1" />

        <div className="space-y-3 px-6 pb-6">
          <div className="overflow-hidden rounded-lg border">
            <CodeMirror
              value={value}
              height="360px"
              theme={oneDark}
              extensions={[json()]}
              basicSetup={{ foldGutter: true, lineNumbers: true }}
              onChange={setValue}
            />
          </div>
          {parsed.error ? (
            <p className="text-destructive text-sm">{parsed.error}</p>
          ) : (
            <p className="text-muted-foreground text-sm">
              JSON is valid. Metadata must stay below 16 KiB after compact serialization.
            </p>
          )}
        </div>

        <DialogFooter className="gap-1 py-3 pr-3">
          <Button variant="outline" type="button" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button size="sm" onClick={handleSave} disabled={hasError || mutation.isPending}>
            {mutation.isPending ? 'Saving...' : 'Save metadata'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function formatJson(metadata: JsonObject) {
  return JSON.stringify(metadata ?? {}, null, 2)
}

function parseJsonObject(value: string): { value?: JsonObject; error?: string } {
  try {
    const parsed = JSON.parse(value)
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
      return { error: 'Metadata must be a JSON object.' }
    }
    return { value: parsed as JsonObject }
  } catch (error) {
    return { error: error instanceof Error ? error.message : 'Invalid JSON.' }
  }
}
