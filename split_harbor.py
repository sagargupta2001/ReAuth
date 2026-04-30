import re
import os

with open('src/application/harbor/service.rs', 'r') as f:
    content = f.read()

impl_start = content.find('impl HarborService {')
pre_impl = content[:impl_start]
body = content[impl_start:]

depth = 0
impl_chunks = []
current_chunk = ""
for i, char in enumerate(body):
    current_chunk += char
    if char == '{':
        depth += 1
    elif char == '}':
        depth -= 1
        if depth == 1 and current_chunk.strip():
            # if we are not at the very end of the impl block
            if current_chunk != '}':
                impl_chunks.append(current_chunk)
            current_chunk = ""
        elif depth == 0:
            # End of impl block
            post_impl = body[i+1:]
            break

# remove the first '{' from the first chunk
if impl_chunks and impl_chunks[0].startswith('impl HarborService {'):
    impl_chunks[0] = impl_chunks[0][len('impl HarborService {'):]

# Now for post_impl, split it into standalone functions
post_chunks = []
current_chunk = ""
depth = 0
for char in post_impl:
    current_chunk += char
    if char == '{':
        depth += 1
    elif char == '}':
        depth -= 1
        if depth == 0 and current_chunk.strip():
            post_chunks.append(current_chunk)
            current_chunk = ""

if current_chunk.strip():
    post_chunks.append(current_chunk)

# Classify methods
export_methods = ['export_bundle', 'export_bundle_with_job', 'export_bundle_internal', 'export_full_bundle', 'list_all_clients', 'list_all_flow_drafts', 'list_all_flow_ids_for_export', 'list_all_roles', 'list_all_users', 'estimate_export_size']
import_methods = ['import_bundle', 'import_bundle_with_job', 'import_bundle_internal', 'import_bundle_with_tx', 'import_full_bundle', 'validate_bundle', 'validate_bundle_for_scope', 'record_import_progress']
job_methods = ['start_job', 'create_job', 'try_update_job_total', 'try_update_job_progress', 'set_job_artifact', 'try_mark_completed', 'try_mark_failed', 'list_jobs', 'get_job', 'spawn_job', 'mark_job_completed', 'mark_job_failed', 'list_job_conflicts', 'try_record_conflict']
utils_methods = ['upgrade_bundle', 'upgrade_v0_to_v1', 'rewrite_reference_ids', 'rewrite_realm_flow_bindings', 'rewrite_user_role_client_ids', 'rewrite_user_role_refs', 'encode_role_ref', 'decode_role_ref', 'parse_theme_meta', 'resolve_available_theme_name', 'normalize_export_selection', 'normalize_bundle_for_export', 'canonicalize_value', 'resource_sort_key', 'asset_sort_key', 'get_string_field', 'normalize_theme_meta', 'summarize_import_counts', 'scope_label', 'conflict_policy_label']
core_methods = ['new']

export_impl = []
import_impl = []
job_impl = []
core_impl = []

for chunk in impl_chunks:
    m = re.search(r'fn\s+([a-zA-Z0-9_]+)', chunk)
    if m:
        name = m.group(1)
        if name in export_methods:
            export_impl.append(chunk)
        elif name in import_methods:
            import_impl.append(chunk)
        elif name in job_methods:
            job_impl.append(chunk)
        elif name in core_methods:
            core_impl.append(chunk)
        else:
            core_impl.append(chunk)
    else:
        core_impl.append(chunk)

export_post = []
import_post = []
job_post = []
utils_post = []

for chunk in post_chunks:
    m = re.search(r'fn\s+([a-zA-Z0-9_]+)', chunk)
    if m:
        name = m.group(1)
        if name in export_methods:
            export_post.append(chunk)
        elif name in import_methods:
            import_post.append(chunk)
        elif name in job_methods:
            job_post.append(chunk)
        elif name in utils_methods:
            utils_post.append(chunk)
        else:
            utils_post.append(chunk)
    else:
        if chunk.strip():
            utils_post.append(chunk)

with open('src/application/harbor/export.rs', 'w') as f:
    f.write('use super::service::HarborService;' + chr(10))
    f.write('use crate::domain::harbor::*;' + chr(10))
    f.write('use crate::error::{Error, Result};' + chr(10))
    f.write('use uuid::Uuid;' + chr(10))
    f.write('use std::collections::{HashMap, HashSet};' + chr(10))
    f.write('use serde_json::Value;' + chr(10))
    f.write('use tracing::{info, warn};' + chr(10) + chr(10))
    f.write('use super::utils::*;' + chr(10))
    f.write('impl HarborService {' + chr(10))
    f.write(''.join(export_impl))
    f.write(chr(10) + '}' + chr(10))
    f.write(chr(10).join(export_post))

with open('src/application/harbor/import.rs', 'w') as f:
    f.write('use super::service::HarborService;' + chr(10))
    f.write('use crate::domain::harbor::*;' + chr(10))
    f.write('use crate::error::{Error, Result};' + chr(10))
    f.write('use uuid::Uuid;' + chr(10))
    f.write('use std::collections::{HashMap, HashSet};' + chr(10))
    f.write('use serde_json::Value;' + chr(10))
    f.write('use tracing::{info, warn};' + chr(10))
    f.write('use crate::ports::transaction_manager::Transaction;' + chr(10) + chr(10))
    f.write('use super::utils::*;' + chr(10))
    f.write('impl HarborService {' + chr(10))
    f.write(''.join(import_impl))
    f.write(chr(10) + '}' + chr(10))
    f.write(chr(10).join(import_post))

with open('src/application/harbor/jobs.rs', 'w') as f:
    f.write('use super::service::HarborService;' + chr(10))
    f.write('use crate::domain::harbor::*;' + chr(10))
    f.write('use crate::domain::harbor_job::*;' + chr(10))
    f.write('use crate::domain::harbor_job_conflict::*;' + chr(10))
    f.write('use crate::error::{Error, Result};' + chr(10))
    f.write('use uuid::Uuid;' + chr(10))
    f.write('use tracing::{info, warn, error};' + chr(10) + chr(10))
    f.write('use super::utils::*;' + chr(10))
    f.write('impl HarborService {' + chr(10))
    f.write(''.join(job_impl))
    f.write(chr(10) + '}' + chr(10))
    f.write(chr(10).join(job_post))

with open('src/application/harbor/utils.rs', 'w') as f:
    f.write('use crate::domain::harbor::*;' + chr(10))
    f.write('use crate::error::{Error, Result};' + chr(10))
    f.write('use uuid::Uuid;' + chr(10))
    f.write('use std::collections::{HashMap, HashSet};' + chr(10))
    f.write('use serde_json::Value;' + chr(10) + chr(10))
    f.write(chr(10).join(utils_post))

with open('src/application/harbor/service.rs', 'w') as f:
    f.write(pre_impl)
    f.write('impl HarborService {')
    f.write(''.join(core_impl))
    f.write(chr(10) + '}' + chr(10))

print("Harbor service split successfully")
