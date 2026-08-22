import { ColorTokenField } from '@/features/fluid/components/settings/fields/ColorTokenField'
import { LayoutShellField } from '@/features/fluid/components/settings/fields/LayoutShellField'
import { NumberTokenField } from '@/features/fluid/components/settings/fields/NumberTokenField'
import { SelectTokenField } from '@/features/fluid/components/settings/fields/SelectTokenField'
import { StaticSettingsFieldView } from '@/features/fluid/components/settings/fields/StaticSettingsFieldView'
import { TextTokenField } from '@/features/fluid/components/settings/fields/TextTokenField'
import { ThemeAssetsField } from '@/features/fluid/components/settings/fields/ThemeAssetsField'
import {
  SettingsFieldKind,
  type SettingsField,
} from '@/features/fluid/model/settingsFields'

/**
 * Maps a schema field descriptor to its control.
 *
 * The union is exhaustive, so adding a `SettingsFieldKind` without handling it
 * here fails to compile.
 */
export function ThemeSettingsField({ field }: { field: SettingsField }) {
  switch (field.kind) {
    case SettingsFieldKind.Color:
      return <ColorTokenField field={field} />
    case SettingsFieldKind.Text:
      return <TextTokenField field={field} />
    case SettingsFieldKind.Number:
      return <NumberTokenField field={field} />
    case SettingsFieldKind.Select:
      return <SelectTokenField field={field} />
    case SettingsFieldKind.Static:
      return <StaticSettingsFieldView field={field} />
    case SettingsFieldKind.LayoutShell:
      return <LayoutShellField />
    case SettingsFieldKind.Assets:
      return <ThemeAssetsField />
    default:
      return assertNever(field)
  }
}

function assertNever(field: never): never {
  throw new Error(`Unhandled settings field: ${JSON.stringify(field)}`)
}
