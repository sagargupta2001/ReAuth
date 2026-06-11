import { useState } from 'react'

import { Braces, Edit3, Eye, Lock, ShieldAlert } from 'lucide-react'

import { Button } from '@/components/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import type { JsonObject, UserMetadataVisibility } from '@/entities/user/model/types.ts'
import { useUserMetadata } from '@/features/user/api/useUserMetadata.ts'
import { Skeleton } from '@/shared/ui/skeleton.tsx'
import { cn } from '@/lib/utils'

import { MetadataEditDialog } from './MetadataEditDialog.tsx'

interface MetadataSectionProps {
  userId: string
}

const metadataBuckets: Array<{
  visibility: UserMetadataVisibility
  title: string
  description: string
  icon: typeof Eye
}> = [
  {
    visibility: 'public',
    title: 'Public metadata',
    description: 'Readable by authenticated frontend and backend APIs.',
    icon: Eye,
  },
  {
    visibility: 'private',
    title: 'Private metadata',
    description: 'Backend-only data for internal integrations and admin workflows.',
    icon: Lock,
  },
  {
    visibility: 'unsafe',
    title: 'Unsafe metadata',
    description: 'Readable and writable by authenticated frontend and backend APIs.',
    icon: ShieldAlert,
  },
]

export function MetadataSection({ userId }: MetadataSectionProps) {
  const { data, isLoading } = useUserMetadata(userId)
  const [editing, setEditing] = useState<UserMetadataVisibility | null>(null)

  if (isLoading)
    return (
      <Card>
        <CardContent className="space-y-3 pt-6">
          <Skeleton className="h-20" />
          <Skeleton className="h-20" />
          <Skeleton className="h-20" />
        </CardContent>
      </Card>
    )

  const metadata = {
    public: data?.public_metadata ?? {},
    private: data?.private_metadata ?? {},
    unsafe: data?.unsafe_metadata ?? {},
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Metadata</CardTitle>
      </CardHeader>
      <CardContent>
        {metadataBuckets.map((bucket, index) => (
          <MetadataBucketRow
            key={bucket.visibility}
            bucket={bucket}
            metadata={metadata[bucket.visibility]}
            isFirst={index === 0}
            isLast={index === metadataBuckets.length - 1}
            onEdit={() => setEditing(bucket.visibility)}
          />
        ))}
      </CardContent>

      {metadataBuckets.map((bucket) => (
        <MetadataEditDialog
          key={bucket.visibility}
          userId={userId}
          visibility={bucket.visibility}
          title={bucket.title}
          metadata={metadata[bucket.visibility]}
          open={editing === bucket.visibility}
          onOpenChange={(open) => setEditing(open ? bucket.visibility : null)}
        />
      ))}
    </Card>
  )
}

function MetadataBucketRow({
  bucket,
  metadata,
  isFirst,
  isLast,
  onEdit,
}: {
  bucket: (typeof metadataBuckets)[number]
  metadata: JsonObject
  isFirst: boolean
  isLast: boolean
  onEdit: () => void
}) {
  const Icon = bucket.icon
  const isEmpty = Object.keys(metadata ?? {}).length === 0

  return (
    <div
      className={cn(
        'bg-primary-foreground p-4',
        isFirst && 'rounded-t-2xl',
        isLast && 'rounded-b-2xl',
        !isLast && 'border-b-0',
      )}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="flex min-w-0 items-start gap-3">
          <Icon className="text-muted-foreground mt-0.5 size-4 shrink-0" />
          <div className="min-w-0 space-y-1">
            <div className="flex flex-wrap items-center gap-2">
              <h3 className="text-sm font-medium">{bucket.title}</h3>
              <span className="bg-background text-muted-foreground inline-flex items-center gap-1 rounded-sm border px-1.5 py-0.5 text-xs">
                <Braces className="size-3" />
                {isEmpty ? 'Empty object' : `${Object.keys(metadata).length} keys`}
              </span>
            </div>
            <p className="text-muted-foreground text-sm">{bucket.description}</p>
          </div>
        </div>

        <Button variant="outline" size="sm" className="shrink-0 gap-1" onClick={onEdit}>
          <Edit3 className="size-3.5" />
          Edit
        </Button>
      </div>

      <pre className="bg-background text-muted-foreground mt-4 max-h-44 overflow-auto rounded-lg border p-3 text-xs leading-5">
        <code>{JSON.stringify(metadata ?? {}, null, 2)}</code>
      </pre>
    </div>
  )
}
