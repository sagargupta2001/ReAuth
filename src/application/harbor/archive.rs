use crate::application::harbor::types::{HarborBundle, HarborManifest};
use crate::error::{Error, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use tar::{Archive, Builder, Header};
use uuid::Uuid;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

const MANIFEST_PATH: &str = "manifest.json";
const BUNDLE_PATH: &str = "data/bundle.json";
const ASSETS_PREFIX: &str = "assets/";

pub fn read_bundle_from_path(path: &Path) -> Result<HarborBundle> {
    if is_tar_archive(path) {
        read_bundle_from_tar(path)
    } else {
        read_bundle_from_zip(path)
    }
}

pub fn write_bundle_to_path(bundle: &HarborBundle, path: &Path) -> Result<()> {
    if is_tar_archive(path) {
        write_bundle_to_tar(bundle, path)
    } else {
        write_bundle_to_zip(bundle, path)
    }
}

fn read_bundle_from_zip(path: &Path) -> Result<HarborBundle> {
    let file = File::open(path).map_err(|e| Error::Unexpected(e.into()))?;
    let mut archive = ZipArchive::new(file).map_err(|e| Error::Unexpected(e.into()))?;

    let mut manifest: Option<HarborManifest> = None;
    let mut bundle: Option<HarborBundle> = None;
    let mut asset_map: HashMap<String, Vec<u8>> = HashMap::new();

    for idx in 0..archive.len() {
        let mut file = archive
            .by_index(idx)
            .map_err(|e| Error::Unexpected(e.into()))?;
        let name = file.name().to_string();

        if name.ends_with('/') {
            continue;
        }

        if name == MANIFEST_PATH {
            let mut buf = String::new();
            file.read_to_string(&mut buf)
                .map_err(|e| Error::Unexpected(e.into()))?;
            let parsed: HarborManifest =
                serde_json::from_str(&buf).map_err(|e| Error::Validation(e.to_string()))?;
            manifest = Some(parsed);
            continue;
        }

        if name == BUNDLE_PATH {
            let mut buf = String::new();
            file.read_to_string(&mut buf)
                .map_err(|e| Error::Unexpected(e.into()))?;
            let parsed: HarborBundle =
                serde_json::from_str(&buf).map_err(|e| Error::Validation(e.to_string()))?;
            bundle = Some(parsed);
            continue;
        }

        if name.starts_with(ASSETS_PREFIX) {
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)
                .map_err(|e| Error::Unexpected(e.into()))?;
            if let Some(asset_id) = parse_asset_id_from_path(&name) {
                asset_map.insert(asset_id, bytes);
            }
        }
    }

    let mut bundle =
        bundle.ok_or_else(|| Error::Validation("Bundle missing data/bundle.json".to_string()))?;

    if let Some(manifest) = manifest {
        if manifest != bundle.manifest {
            return Err(Error::Validation(
                "manifest.json does not match bundle manifest".to_string(),
            ));
        }
    }

    hydrate_assets_from_map(&mut bundle, &asset_map)?;
    Ok(bundle)
}

fn read_bundle_from_tar(path: &Path) -> Result<HarborBundle> {
    let file = File::open(path).map_err(|e| Error::Unexpected(e.into()))?;
    let reader: Box<dyn Read> = if is_tar_gz(path) {
        Box::new(GzDecoder::new(file))
    } else {
        Box::new(file)
    };
    let mut archive = Archive::new(reader);

    let mut manifest: Option<HarborManifest> = None;
    let mut bundle: Option<HarborBundle> = None;
    let mut asset_map: HashMap<String, Vec<u8>> = HashMap::new();

    for entry in archive.entries().map_err(|e| Error::Unexpected(e.into()))? {
        let mut entry = entry.map_err(|e| Error::Unexpected(e.into()))?;
        let path = entry
            .path()
            .map_err(|e| Error::Unexpected(e.into()))?
            .to_string_lossy()
            .to_string();

        if path.ends_with('/') {
            continue;
        }

        if path == MANIFEST_PATH {
            let mut buf = String::new();
            entry
                .read_to_string(&mut buf)
                .map_err(|e| Error::Unexpected(e.into()))?;
            let parsed: HarborManifest =
                serde_json::from_str(&buf).map_err(|e| Error::Validation(e.to_string()))?;
            manifest = Some(parsed);
            continue;
        }

        if path == BUNDLE_PATH {
            let mut buf = String::new();
            entry
                .read_to_string(&mut buf)
                .map_err(|e| Error::Unexpected(e.into()))?;
            let parsed: HarborBundle =
                serde_json::from_str(&buf).map_err(|e| Error::Validation(e.to_string()))?;
            bundle = Some(parsed);
            continue;
        }

        if path.starts_with(ASSETS_PREFIX) {
            let mut bytes = Vec::new();
            entry
                .read_to_end(&mut bytes)
                .map_err(|e| Error::Unexpected(e.into()))?;
            if let Some(asset_id) = parse_asset_id_from_path(&path) {
                asset_map.insert(asset_id, bytes);
            }
        }
    }

    let mut bundle =
        bundle.ok_or_else(|| Error::Validation("Bundle missing data/bundle.json".to_string()))?;

    if let Some(manifest) = manifest {
        if manifest != bundle.manifest {
            return Err(Error::Validation(
                "manifest.json does not match bundle manifest".to_string(),
            ));
        }
    }

    hydrate_assets_from_map(&mut bundle, &asset_map)?;
    Ok(bundle)
}

fn write_bundle_to_zip(bundle: &HarborBundle, path: &Path) -> Result<()> {
    let file = File::create(path).map_err(|e| Error::Unexpected(e.into()))?;
    let mut writer = ZipWriter::new(file);
    let options = FileOptions::<()>::default();

    let manifest_json =
        serde_json::to_vec_pretty(&bundle.manifest).map_err(|e| Error::Unexpected(e.into()))?;
    writer
        .start_file(MANIFEST_PATH, options)
        .map_err(|e| Error::Unexpected(e.into()))?;
    writer
        .write_all(&manifest_json)
        .map_err(|e| Error::Unexpected(e.into()))?;

    let bundle_json = serde_json::to_vec_pretty(bundle).map_err(|e| Error::Unexpected(e.into()))?;
    writer
        .start_file(BUNDLE_PATH, options)
        .map_err(|e| Error::Unexpected(e.into()))?;
    writer
        .write_all(&bundle_json)
        .map_err(|e| Error::Unexpected(e.into()))?;

    write_assets_to_zip(bundle, &mut writer)?;

    writer.finish().map_err(|e| Error::Unexpected(e.into()))?;
    Ok(())
}

fn write_bundle_to_tar(bundle: &HarborBundle, path: &Path) -> Result<()> {
    let file = File::create(path).map_err(|e| Error::Unexpected(e.into()))?;
    let writer: Box<dyn Write> = if is_tar_gz(path) {
        Box::new(GzEncoder::new(file, Compression::default()))
    } else {
        Box::new(file)
    };
    let mut builder = Builder::new(writer);

    let manifest_json =
        serde_json::to_vec_pretty(&bundle.manifest).map_err(|e| Error::Unexpected(e.into()))?;
    append_bytes(&mut builder, MANIFEST_PATH, &manifest_json)?;

    let bundle_json = serde_json::to_vec_pretty(bundle).map_err(|e| Error::Unexpected(e.into()))?;
    append_bytes(&mut builder, BUNDLE_PATH, &bundle_json)?;

    write_assets_to_tar(bundle, &mut builder)?;

    builder.finish().map_err(|e| Error::Unexpected(e.into()))?;
    Ok(())
}

fn write_assets_to_zip(bundle: &HarborBundle, writer: &mut ZipWriter<File>) -> Result<()> {
    let options = FileOptions::<()>::default();
    let mut asset_index = 0usize;

    for resource in &bundle.resources {
        for asset in &resource.assets {
            if asset.data_base64.trim().is_empty() {
                continue;
            }
            let data = STANDARD
                .decode(asset.data_base64.as_bytes())
                .map_err(|_| Error::Validation("Invalid asset data".to_string()))?;

            let filename = build_asset_filename(
                resource.key.as_str(),
                asset.id.as_deref(),
                &asset.filename,
                asset_index,
            );
            asset_index += 1;
            writer
                .start_file(filename, options)
                .map_err(|e| Error::Unexpected(e.into()))?;
            writer
                .write_all(&data)
                .map_err(|e| Error::Unexpected(e.into()))?;
        }
    }

    Ok(())
}

fn write_assets_to_tar(bundle: &HarborBundle, builder: &mut Builder<Box<dyn Write>>) -> Result<()> {
    let mut asset_index = 0usize;

    for resource in &bundle.resources {
        for asset in &resource.assets {
            if asset.data_base64.trim().is_empty() {
                continue;
            }
            let data = STANDARD
                .decode(asset.data_base64.as_bytes())
                .map_err(|_| Error::Validation("Invalid asset data".to_string()))?;

            let filename = build_asset_filename(
                resource.key.as_str(),
                asset.id.as_deref(),
                &asset.filename,
                asset_index,
            );
            asset_index += 1;
            append_bytes(builder, &filename, &data)?;
        }
    }

    Ok(())
}

fn append_bytes(builder: &mut Builder<Box<dyn Write>>, path: &str, data: &[u8]) -> Result<()> {
    let mut header = Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, path, Cursor::new(data))
        .map_err(|e| Error::Unexpected(e.into()))?;
    Ok(())
}

fn build_asset_filename(
    resource_key: &str,
    asset_id: Option<&str>,
    filename: &str,
    index: usize,
) -> String {
    let id = asset_id
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.to_string())
        .unwrap_or_else(|| format!("asset-{}", index));
    format!(
        "{}/{}/{}__{}",
        ASSETS_PREFIX.trim_end_matches('/'),
        resource_key,
        id,
        filename
    )
}

fn parse_asset_id_from_path(path: &str) -> Option<String> {
    let filename = Path::new(path).file_name().and_then(|name| name.to_str())?;
    let (id_part, _) = filename.split_once("__")?;
    if Uuid::parse_str(id_part).is_ok() {
        return Some(id_part.to_string());
    }
    if !id_part.trim().is_empty() {
        return Some(id_part.to_string());
    }
    None
}

fn hydrate_assets_from_map(
    bundle: &mut HarborBundle,
    map: &HashMap<String, Vec<u8>>,
) -> Result<()> {
    for resource in &mut bundle.resources {
        if resource.assets.is_empty() {
            continue;
        }
        for asset in &mut resource.assets {
            if !asset.data_base64.trim().is_empty() {
                continue;
            }
            let Some(id) = asset.id.as_ref() else {
                continue;
            };
            let bytes = map
                .get(id)
                .ok_or_else(|| Error::Validation(format!("Asset data missing for id {}", id)))?;
            asset.data_base64 = STANDARD.encode(bytes);
        }
    }
    Ok(())
}

fn is_tar_archive(path: &Path) -> bool {
    let lower = path.to_string_lossy().to_lowercase();
    lower.ends_with(".tar") || lower.ends_with(".tar.gz") || lower.ends_with(".tgz")
}

fn is_tar_gz(path: &Path) -> bool {
    let lower = path.to_string_lossy().to_lowercase();
    lower.ends_with(".tar.gz") || lower.ends_with(".tgz")
}
