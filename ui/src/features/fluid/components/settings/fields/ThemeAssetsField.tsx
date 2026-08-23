import type { ChangeEvent } from 'react'
import { useRef } from 'react'

import { Image, UploadCloud } from 'lucide-react'

import type { ThemeAsset } from '@/entities/theme/model/types'
import { useThemeSettings } from '@/features/fluid/components/settings/themeSettingsContext'

const BYTES_PER_KB = 1024
const UPLOAD_HINT = 'PNG, JPG, SVG'
const IMAGE_MIME_PREFIX = 'image/'

/** Asset uploader plus the list of assets already attached to the theme. */
export function ThemeAssetsField() {
  const { assets, onUploadAsset, isUploading } = useThemeSettings()

  return (
    <div className="space-y-3">
      <AssetUploadButton onSelectFile={onUploadAsset} isUploading={isUploading} />
      <div className="space-y-2">
        {assets.length === 0 ? (
          <p className="text-muted-foreground text-xs">No assets uploaded yet.</p>
        ) : (
          assets.map((asset) => <AssetListItem key={asset.id} asset={asset} />)
        )}
      </div>
    </div>
  )
}

function AssetUploadButton({
  onSelectFile,
  isUploading,
}: {
  onSelectFile: (file: File) => void
  isUploading: boolean
}) {
  const fileInputRef = useRef<HTMLInputElement | null>(null)

  const handleChange = (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (file) {
      onSelectFile(file)
    }
    // Reset so re-picking the same file still fires a change event.
    event.target.value = ''
  }

  return (
    <>
      <input ref={fileInputRef} type="file" className="hidden" onChange={handleChange} />
      <button
        type="button"
        className="border-border hover:border-primary/60 hover:bg-muted/40 flex w-full items-center justify-between rounded-lg border px-3 py-2 text-left text-xs transition-colors"
        onClick={() => fileInputRef.current?.click()}
      >
        <span className="flex items-center gap-2">
          <UploadCloud className="text-muted-foreground h-4 w-4" />
          Upload asset
        </span>
        <span className="text-muted-foreground">
          {isUploading ? 'Uploading...' : UPLOAD_HINT}
        </span>
      </button>
    </>
  )
}

function AssetListItem({ asset }: { asset: ThemeAsset }) {
  const isImage = asset.mime_type.startsWith(IMAGE_MIME_PREFIX)

  return (
    <div className="bg-background flex items-center gap-3 rounded-md border px-3 py-2 text-xs">
      {isImage ? (
        <img
          src={asset.url}
          alt={asset.filename}
          className="h-10 w-10 rounded-md border object-cover"
        />
      ) : (
        <div className="bg-muted flex h-10 w-10 items-center justify-center rounded-md border">
          <Image className="text-muted-foreground h-4 w-4" />
        </div>
      )}
      <div className="flex flex-1 flex-col">
        <span className="font-medium">{asset.filename}</span>
        <span className="text-muted-foreground text-[10px]">
          {(asset.byte_size / BYTES_PER_KB).toFixed(1)} KB · {asset.asset_type}
        </span>
      </div>
    </div>
  )
}
