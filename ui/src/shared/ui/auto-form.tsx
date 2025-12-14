import { Input } from '@/components/input'
import { Label } from '@/components/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Switch } from '@/components/switch'

interface AutoFormProps {
  schema: Record<string, any>
  values: Record<string, any>
  onChange: (newValues: Record<string, any>) => void
}

export function AutoForm({ schema, values = {}, onChange }: AutoFormProps) {
  if (!schema || !schema.properties) return null

  const handleChange = (key: string, value: any) => {
    onChange({
      ...values,
      [key]: value,
    })
  }

  return (
    <div className="grid gap-4">
      {Object.entries(schema.properties).map(([key, fieldSchema]: [string, any]) => {
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
  schema: any
  value: any
  onChange: (val: any) => void
  error?: string | null
}) {
  const label = schema.title || name
  const description = schema.description

  // 1. ENUM (Select)
  if (schema.enum) {
    return (
      <div className="space-y-2">
        <Label className="text-foreground/80 text-xs font-semibold">{label}</Label>
        <Select value={value} onValueChange={onChange}>
          <SelectTrigger className="h-8 text-xs">
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
          <Label className="text-foreground/80 text-xs font-semibold">{label}</Label>
          {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
        </div>
        <Switch checked={value} onCheckedChange={onChange} className="scale-75" />
      </div>
    )
  }

  // 3. INTEGER / NUMBER
  if (schema.type === 'integer' || schema.type === 'number') {
    return (
      <div className="space-y-2">
        <Label className="text-foreground/80 text-xs font-semibold">{label}</Label>
        <Input
          type="number"
          className="h-8 text-xs"
          value={value}
          min={schema.minimum}
          max={schema.maximum}
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
      <Label className="text-foreground/80 text-xs font-semibold">{label}</Label>
      <Input
        className="h-8 text-xs"
        value={value || ''}
        onChange={(e) => onChange(e.target.value)}
        placeholder={schema.default ? `Default: ${schema.default}` : ''}
      />
      {description && <p className="text-muted-foreground text-[10px]">{description}</p>}
    </div>
  )
}
