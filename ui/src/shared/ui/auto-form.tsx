import * as React from 'react'
import { Input } from '@/components/input'
import { Label } from '@/components/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Switch } from '@/components/switch'

interface AutoFormProps {
  schema: Record<string, unknown>
  values: Record<string, unknown>
  onChange: (newValues: Record<string, unknown>) => void
}

export function AutoForm({ schema, values = {}, onChange }: AutoFormProps) {
  if (!schema || !schema.properties) return null

  const properties = schema.properties as Record<string, Record<string, unknown>>

  const handleChange = (key: string, value: unknown) => {
    onChange({
      ...values,
      [key]: value,
    })
  }

  return (
    <div className="grid gap-4">
      {Object.entries(properties).map(([key, fieldSchema]) => {
        const value = values[key] ?? fieldSchema.default
        const error = null // todo: Integrate Zod validation errors here

        return (
          <FieldRenderer
            key={key}
            name={key}
            schema={fieldSchema}
            value={value}
            onChange={(val) => handleChange(key, val)}
            error={error}
          />
        )
      })}
    </div>
  )
}

/**
 * Dispatches the correct input component based on the schema type
 */
function FieldRenderer({
  name,
  schema,
  value,
  onChange,
}: {
  name: string
  schema: Record<string, unknown>
  value: unknown
  onChange: (val: unknown) => void
  error?: string | null
}) {
  const id = React.useId()
  const label = (schema.title as string) || name
  const description = schema.description as string | undefined

  // 1. ENUM (Select)
  if (schema.enum && Array.isArray(schema.enum)) {
    return (
      <div className="space-y-2">
        <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">{label}</Label>
        <Select value={value as string} onValueChange={onChange}>
          <SelectTrigger id={id} className="h-8 text-xs">
            <SelectValue placeholder="Select..." />
          </SelectTrigger>
          <SelectContent>
            {schema.enum.map((option: string) => (
              <SelectItem key={option} value={option} className="text-xs">
                {option}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
      </div>
    )
  }

  // 2. BOOLEAN (Switch)
  if (schema.type === 'boolean') {
    return (
      <div className="flex items-center justify-between rounded-lg border p-3 shadow-sm">
        <div className="space-y-0.5">
          <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">{label}</Label>
          {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
        </div>
        <Switch id={id} checked={value as boolean} onCheckedChange={onChange} className="scale-75" />
      </div>
    )
  }

  // 3. INTEGER / NUMBER
  if (schema.type === 'integer' || schema.type === 'number') {
    return (
      <div className="space-y-2">
        <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">{label}</Label>
        <Input
          id={id}
          type="number"
          className="h-8 text-xs"
          value={(value as number) ?? ''}
          min={schema.minimum as number | undefined}
          max={schema.maximum as number | undefined}
          onChange={(e) => {
            const val = e.target.value === '' ? undefined : Number(e.target.value)
            onChange(val)
          }}
        />
        {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
      </div>
    )
  }

  // 4. STRING (Default)
  return (
    <div className="space-y-2">
      <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">{label}</Label>
      <Input
        id={id}
        className="h-8 text-xs"
        value={(value as string) || ''}
        onChange={(e) => onChange(e.target.value)}
        placeholder={schema.default ? `Default: ${schema.default}` : ''}
      />
      {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
    </div>
  )
}
