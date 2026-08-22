import { useState } from 'react'

import { Button } from '@/components/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/select'
import { ColorPicker } from '@/features/fluid/components/controls/ColorPicker'
import {
  DESIGN_COLOR_TOKENS,
  isDesignTokenValue,
} from '@/features/fluid/model/designTokens'
import { resolveCssColor } from '@/lib/colorUtils'

const ColorSource = {
  Token: 'token',
  Custom: 'custom',
} as const

type ColorSource = (typeof ColorSource)[keyof typeof ColorSource]

const SOURCE_OPTIONS: readonly { value: ColorSource; label: string }[] = [
  { value: ColorSource.Token, label: 'Design token' },
  { value: ColorSource.Custom, label: 'Custom' },
]

interface ThemeColorControlProps {
  id: string
  value: string
  onChange: (value: string) => void
  /** Swatch colour used while the value is empty or unparseable. */
  fallback: string
  ariaLabel?: string
}

/**
 * Colour control that makes "inherit a design token" and "pin a literal colour"
 * two explicit modes.
 *
 * Seeded themes store `var(--primary)`, which a plain colour input cannot
 * represent — the swatch would show an unrelated fallback while the text field
 * showed the token. Here the mode is visible and the token's real colour is
 * previewed.
 */
export function ThemeColorControl({
  id,
  value,
  onChange,
  fallback,
  ariaLabel,
}: ThemeColorControlProps) {
  const isToken = isDesignTokenValue(value)
  // Remembered so toggling Custom -> Token -> Custom does not lose the hex.
  const [lastCustom, setLastCustom] = useState(() => (isToken ? '' : value))

  const handleSourceChange = (source: ColorSource) => {
    if (source === ColorSource.Token) {
      if (!isToken) setLastCustom(value)
      onChange(DESIGN_COLOR_TOKENS[0].value)
      return
    }
    if (!isToken) return
    // Seed the picker with the token's actual colour so the swatch is truthful.
    onChange(lastCustom || resolveCssColor(value) || fallback)
  }

  return (
    <div className="space-y-2">
      <div className="flex gap-1" role="group" aria-label={`${ariaLabel ?? 'Color'} source`}>
        {SOURCE_OPTIONS.map((option) => {
          const isActive = (isToken ? ColorSource.Token : ColorSource.Custom) === option.value
          return (
            <Button
              key={option.value}
              type="button"
              size="sm"
              variant={isActive ? 'default' : 'outline'}
              aria-pressed={isActive}
              className="h-7 flex-1 px-2 text-[11px]"
              onClick={() => handleSourceChange(option.value)}
            >
              {option.label}
            </Button>
          )
        })}
      </div>

      {isToken ? (
        <Select value={value} onValueChange={onChange}>
          <SelectTrigger id={id} className="bg-background h-9 text-xs">
            <SelectValue placeholder="Select a token" />
          </SelectTrigger>
          <SelectContent>
            {DESIGN_COLOR_TOKENS.map((token) => (
              <SelectItem key={token.value} value={token.value}>
                <span className="flex items-center gap-2">
                  <TokenSwatch value={token.value} />
                  {token.label}
                </span>
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      ) : (
        <ColorPicker
          id={id}
          value={value}
          fallback={fallback}
          ariaLabel={ariaLabel ? `${ariaLabel} color` : undefined}
          onChange={onChange}
        />
      )}
    </div>
  )
}

function TokenSwatch({ value }: { value: string }) {
  return (
    <span
      aria-hidden="true"
      className="h-3.5 w-3.5 shrink-0 rounded-full border"
      style={{ backgroundColor: resolveCssColor(value) ?? 'transparent' }}
    />
  )
}
