# Harbor Bundle (.reauth) Spec

## Overview
`.reauth` is a portable archive for Harbor import/export. It is a **zip** (preferred) or **tar.gz**
container with a stable, versioned manifest and resource data split from binary assets.

The API representation (`HarborBundle` JSON) is stored in `data/bundle.json`.
This spec defines the on-disk archive layout and mapping to the API payload.

## Archive layout
```
bundle.reauth
├── manifest.json
├── data/
│   └── bundle.json
└── assets/
    └── theme/
        ├── <asset_id>__<filename>
        └── ...
```

### `manifest.json`
Required. Mirrors `HarborManifest` from the API.
```
{
  "version": "1.0",
  "schema_version": 1,
  "exported_at": "2026-03-05T10:00:00Z",
  "source_realm": "acme-corp",
  "type": "full_realm",
  "selection": ["client", "flow", "theme"]
}
```
Rules:
- `exported_at` must be RFC3339.
- `source_realm` must be non-empty.
- `selection` is optional and only present for full realm exports.

### `data/bundle.json`
Required. The exact JSON payload used by the Harbor API:
```
{
  "manifest": { ...same as manifest.json... },
  "resources": [
    { "key": "client", "data": { ... }, "assets": [] },
    { "key": "flow", "data": { ... }, "assets": [] },
    { "key": "theme", "data": { ... }, "assets": [ ... ] }
  ]
}
```

The JSON schema for this file is validated at import time (see `docs/schemas/harbor/`).

### `assets/`
Optional. Binary assets can be stored on disk for portability. The current API schema expects
`assets[].data_base64` in `bundle.json`. When assets are present on disk, the export process:
1. Writes asset files into `assets/` following the naming convention above.
2. Keeps `assets[].data_base64` in `bundle.json` for backward compatibility.

Future compatibility: when Harbor supports file-backed assets, `assets[].data_base64` may be omitted
and replaced with a path reference. Until then, treat `assets/` as a convenience mirror.

## Versioning & compatibility
- `version`: human-readable bundle format version.
- `schema_version`: machine-readable schema version, used for up-converters.
- Importers must support **N-2** schema versions.

## Naming conventions
- Asset filenames: `<asset_id>__<filename>` to avoid collisions.
- Resource keys: `theme`, `client`, `flow`.

## Determinism (recommended)
- Stable JSON key ordering during export.
- Stable resource ordering within `bundle.json`.
