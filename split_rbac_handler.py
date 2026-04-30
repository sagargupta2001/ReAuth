import re
import os

with open('src/adapters/web/rbac_handler.rs', 'r') as f:
    content = f.read()

# First find the first function or macro. Everything before it is imports.
match = re.search(r'// POST /api/realms/', content)
if not match:
    print("Could not find start of functions")
    exit(1)

preamble = content[:match.start()].replace('async fn record_audit', 'pub(super) async fn record_audit')
body = content[match.start():]

depth = 0
chunks = []
current_chunk = ""
for char in body:
    current_chunk += char
    if char == '{':
        depth += 1
    elif char == '}':
        depth -= 1
        if depth == 0:
            chunks.append(current_chunk)
            current_chunk = ""

roles_methods = [
    'create_role_handler', 'list_roles_handler', 'list_client_roles_handler',
    'get_role_handler', 'update_role_handler', 'delete_role_handler',
    'list_permissions_handler', 'create_custom_permission_handler',
    'update_custom_permission_handler', 'delete_custom_permission_handler',
    'list_role_permissions_handler', 'list_role_members_handler',
    'list_role_members_page_handler', 'revoke_permission_handler',
    'bulk_permissions_handler', 'list_role_composites_handler',
    'list_role_composites_page_handler', 'assign_composite_role_handler',
    'remove_composite_role_handler', 'list_user_roles_handler',
    'list_user_roles_page_handler', 'assign_permission_handler',
    'assign_user_role_handler', 'remove_user_role_handler'
]

groups_methods = [
    'create_group_handler', 'list_groups_handler', 'list_group_roots_handler',
    'list_group_children_handler', 'get_group_handler',
    'get_group_delete_summary_handler', 'update_group_handler',
    'delete_group_handler', 'move_group_handler', 'assign_user_to_group_handler',
    'remove_user_from_group_handler', 'list_group_members_handler',
    'list_group_members_page_handler', 'assign_role_to_group_handler',
    'remove_role_from_group_handler', 'list_group_roles_handler',
    'list_group_roles_page_handler'
]

roles_content = []
groups_content = []
other_content = []

for chunk in chunks:
    m = re.search(r'pub async fn\s+([a-zA-Z0-9_]+)', chunk)
    if m:
        name = m.group(1)
        if name in roles_methods:
            roles_content.append(chunk)
        elif name in groups_methods:
            groups_content.append(chunk)
        else:
            other_content.append(chunk)
    else:
        if chunk.strip():
            other_content.append(chunk)

os.makedirs('src/adapters/web/rbac_handler', exist_ok=True)

imports = '''use crate::adapters::web::auth_middleware::AuthUser;
use crate::application::rbac_service::{
    CreateCustomPermissionPayload, CreateGroupPayload, CreateRolePayload,
    UpdateCustomPermissionPayload,
};
use crate::domain::audit::NewAuditEvent;
use crate::domain::pagination::PageRequest;
use crate::domain::permissions::{self, PermissionDef, ResourceGroup};
use crate::domain::rbac::{
    GroupMemberFilter, GroupRoleFilter, RoleCompositeFilter, RoleMemberFilter, UserRoleFilter,
};
use crate::error::{Error, Result};
use crate::AppState;
use axum::extract::Query;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;
use serde_json::json;
use tracing::error;
use uuid::Uuid;
use super::record_audit;
'''

with open('src/adapters/web/rbac_handler/role_handlers.rs', 'w') as f:
    f.write(imports)
    f.write(chr(10).join(roles_content))

with open('src/adapters/web/rbac_handler/group_handlers.rs', 'w') as f:
    f.write(imports)
    f.write(chr(10).join(groups_content))

with open('src/adapters/web/rbac_handler/mod.rs', 'w') as f:
    f.write(preamble)
    f.write(chr(10) + 'pub mod role_handlers;' + chr(10))
    f.write('pub mod group_handlers;' + chr(10) + chr(10))
    f.write('pub use role_handlers::*;' + chr(10))
    f.write('pub use group_handlers::*;' + chr(10))
    if other_content:
        f.write(chr(10).join(other_content))

print("Handlers split successfully")
