import * as React from 'react'

import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Label } from '@/components/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Switch } from '@/components/switch'
import { Textarea } from '@/components/textarea'

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

        return (
          <FieldRenderer
            key={key}
            name={key}
            schema={fieldSchema}
            value={value}
            onChange={(val) => handleChange(key, val)}
          />
        )
      })}
    </div>
  )
}

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
}) {
  const id = React.useId()
  const label = (schema.title as string) || name
  const description = schema.description as string | undefined
  const placeholder = schema.default ? `Default: ${schema.default}` : ''

  if (schema.enum && Array.isArray(schema.enum)) {
    return (
      <div className="space-y-2">
        <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">
          {label}
        </Label>
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

  if (schema.type === 'boolean') {
    return (
      <div className="flex items-center justify-between rounded-lg border p-3 shadow-sm">
        <div className="space-y-0.5">
          <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">
            {label}
          </Label>
          {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
        </div>
        <Switch id={id} checked={Boolean(value)} onCheckedChange={onChange} className="scale-75" />
      </div>
    )
  }

  if (schema.type === 'integer' || schema.type === 'number') {
    return (
      <div className="space-y-2">
        <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">
          {label}
        </Label>
        <Input
          id={id}
          type="number"
          className="h-8 text-xs"
          value={(value as number) ?? ''}
          min={schema.minimum as number | undefined}
          max={schema.maximum as number | undefined}
          onChange={(event) => {
            const next = event.target.value === '' ? undefined : Number(event.target.value)
            onChange(next)
          }}
        />
        {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
      </div>
    )
  }

  if (schema.type === 'array') {
    const items = Array.isArray(value) ? value : []

    return (
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">
            {label}
          </Label>
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="h-7 px-2 text-[10px]"
            onClick={() => onChange([...items, ''])}
          >
            Add item
          </Button>
        </div>
        <div className="space-y-2">
          {items.map((item, index) => (
            <div key={`${name}-${index}`} className="flex items-center gap-2">
              <Input
                id={index === 0 ? id : undefined}
                className="h-8 text-xs"
                value={String(item ?? '')}
                onChange={(event) => {
                  const next = [...items]
                  next[index] = event.target.value
                  onChange(next)
                }}
                placeholder={placeholder}
              />
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="h-8 px-2 text-[10px]"
                onClick={() => onChange(items.filter((_, itemIndex) => itemIndex !== index))}
              >
                Remove
              </Button>
            </div>
          ))}
        </div>
        {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
      </div>
    )
  }

  if (schema.format === 'textarea' || schema.format === 'multiline') {
    return (
      <div className="space-y-2">
        <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">
          {label}
        </Label>
        <Textarea
          id={id}
          className="min-h-24 text-xs"
          value={(value as string) || ''}
          onChange={(event) => onChange(event.target.value)}
          placeholder={placeholder}
        />
        {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
      </div>
    )
  }

  return (
    <div className="space-y-2">
      <Label htmlFor={id} className="text-foreground/80 text-xs font-semibold">
        {label}
      </Label>
      <Input
        id={id}
        className="h-8 text-xs"
        value={(value as string) || ''}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
      />
      {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
    </div>
  )
}
