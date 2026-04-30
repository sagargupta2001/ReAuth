import re
import os

with open('src/application/rbac_service/mod.rs', 'r') as f:
    content = f.read()

impl_start = content.find('impl RbacService {')
pre_impl = content[:impl_start]
impl_body = content[impl_start:]

depth = 1
chunks = []
current_chunk = ""
for char in impl_body[18:]:
    current_chunk += char
    if char == '{':
        depth += 1
    elif char == '}':
        depth -= 1
        if depth == 1:
            chunks.append(current_chunk)
            current_chunk = ""
        elif depth == 0:
            break

roles_methods = ['create_role', 'find_role_by_name', 'list_roles', 'list_client_roles', 'get_role', 'update_role', 'delete_role', 'list_custom_permissions', 'create_custom_permission', 'update_custom_permission', 'delete_custom_permission']
groups_methods = ['create_group', 'list_groups', 'list_group_roots', 'list_group_children', 'move_group', 'get_group', 'update_group', 'get_group_delete_summary', 'delete_group']
assignments_methods = ['list_role_members', 'list_group_members', 'list_group_roles', 'list_user_roles', 'list_role_composites', 'assign_role_to_group', 'assign_composite_role', 'assign_user_to_group', 'remove_role_from_group', 'remove_composite_role', 'remove_user_from_group', 'assign_role_to_user', 'remove_role_from_user', 'get_permissions_for_role', 'assign_permission_to_role', 'revoke_permission', 'bulk_update_permissions', 'get_user_roles_and_groups', 'get_direct_user_ids_for_role', 'get_effective_user_ids_for_role', 'get_group_member_ids', 'get_group_role_ids', 'get_effective_group_role_ids', 'get_direct_role_ids_for_user', 'get_effective_role_ids_for_user', 'get_role_composite_ids', 'get_effective_role_composite_ids', 'user_has_permission', 'get_effective_permissions']
core_methods = ['new', 'write_outbox']

roles_content = []
groups_content = []
assignments_content = []
core_content = []

for chunk in chunks:
    match = re.search(r'fn\s+([a-zA-Z0-9_]+)', chunk)
    if match:
        name = match.group(1)
        if name in roles_methods:
            roles_content.append(chunk)
        elif name in groups_methods:
            groups_content.append(chunk)
        elif name in assignments_methods:
            assignments_content.append(chunk)
        elif name in core_methods:
            core_content.append(chunk)
        else:
            core_content.append(chunk)
    else:
        core_content.append(chunk)

with open('src/application/rbac_service/roles.rs', 'w') as f:
    f.write('use super::RbacService;' + chr(10) + 'use crate::domain::role::Role;' + chr(10) + 'use crate::domain::rbac::CustomPermission;' + chr(10) + 'use crate::domain::pagination::{PageRequest, PageResponse};' + chr(10) + 'use super::{CreateRolePayload, CreateCustomPermissionPayload, UpdateCustomPermissionPayload};' + chr(10) + 'use crate::error::{Error, Result};' + chr(10) + 'use uuid::Uuid;' + chr(10) + 'use crate::domain::events::{DomainEvent, RoleCompositeChanged, UserRoleChanged};' + chr(10) + 'use crate::ports::transaction_manager::Transaction;' + chr(10) + 'use chrono::Utc;' + chr(10) + chr(10) + 'impl RbacService {')
    f.write(''.join(roles_content))
    f.write(chr(10) + '}' + chr(10))

with open('src/application/rbac_service/groups.rs', 'w') as f:
    f.write('use super::RbacService;' + chr(10) + 'use crate::domain::group::Group;' + chr(10) + 'use crate::domain::rbac::{GroupDeleteSummary, GroupTreeRow};' + chr(10) + 'use crate::domain::pagination::{PageRequest, PageResponse};' + chr(10) + 'use super::CreateGroupPayload;' + chr(10) + 'use crate::error::{Error, Result};' + chr(10) + 'use uuid::Uuid;' + chr(10) + chr(10) + 'impl RbacService {')
    f.write(''.join(groups_content))
    f.write(chr(10) + '}' + chr(10))

with open('src/application/rbac_service/assignments.rs', 'w') as f:
    f.write('use super::RbacService;' + chr(10) + 'use crate::domain::rbac::*;' + chr(10) + 'use crate::domain::pagination::{PageRequest, PageResponse};' + chr(10) + 'use crate::error::{Error, Result};' + chr(10) + 'use uuid::Uuid;' + chr(10) + 'use crate::domain::events::{DomainEvent, RoleCompositeChanged, UserRoleChanged, RoleGroupChanged, RolePermissionChanged, UserGroupChanged};' + chr(10) + 'use crate::ports::transaction_manager::Transaction;' + chr(10) + 'use std::collections::HashSet;' + chr(10) + 'use tracing::instrument;' + chr(10) + chr(10) + 'impl RbacService {')
    f.write(''.join(assignments_content))
    f.write(chr(10) + '}' + chr(10))

with open('src/application/rbac_service/mod.rs', 'w') as f:
    f.write(pre_impl)
    f.write('pub mod roles;' + chr(10))
    f.write('pub mod groups;' + chr(10))
    f.write('pub mod assignments;' + chr(10) + chr(10))
    f.write('impl RbacService {')
    f.write(''.join(core_content))
    f.write(chr(10) + '}' + chr(10))

print("Done!")
