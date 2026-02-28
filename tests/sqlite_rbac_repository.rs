mod support;

use anyhow::Result;
use reauth::adapters::persistence::connection::Database;
use reauth::adapters::persistence::sqlite_rbac_repository::SqliteRbacRepository;
use reauth::domain::group::Group;
use reauth::domain::pagination::{PageRequest, SortDirection};
use reauth::domain::rbac::{
    CustomPermission, GroupMemberFilter, GroupRoleFilter, RoleCompositeFilter, RoleMemberFilter,
    UserRoleFilter,
};
use reauth::domain::role::{Permission, Role};
use reauth::error::Error;
use reauth::ports::rbac_repository::RbacRepository;
use std::collections::{HashMap, HashSet};
use support::TestDb;
use uuid::Uuid;

fn page_request(
    page: i64,
    per_page: i64,
    sort_by: Option<&str>,
    sort_dir: Option<SortDirection>,
    q: Option<&str>,
) -> PageRequest {
    PageRequest {
        page,
        per_page,
        sort_by: sort_by.map(|value| value.to_string()),
        sort_dir,
        q: q.map(|value| value.to_string()),
    }
}

fn role(id: Uuid, realm_id: Uuid, client_id: Option<Uuid>, name: &str) -> Role {
    Role {
        id,
        realm_id,
        client_id,
        name: name.to_string(),
        description: Some(format!("{} role", name)),
    }
}

fn group(id: Uuid, realm_id: Uuid, parent_id: Option<Uuid>, name: &str, sort_order: i64) -> Group {
    Group {
        id,
        realm_id,
        parent_id,
        name: name.to_string(),
        description: Some(format!("{} group", name)),
        sort_order,
    }
}

fn custom_permission(
    id: Uuid,
    realm_id: Uuid,
    client_id: Option<Uuid>,
    permission: &str,
    name: &str,
    created_by: Option<Uuid>,
) -> CustomPermission {
    CustomPermission {
        id,
        realm_id,
        client_id,
        permission: permission.to_string(),
        name: name.to_string(),
        description: Some(format!("{} permission", name)),
        created_by,
    }
}

async fn insert_realm(pool: &Database, realm_id: Uuid, name: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO realms (id, name, access_token_ttl_secs, refresh_token_ttl_secs) VALUES (?, ?, ?, ?)",
    )
    .bind(realm_id.to_string())
    .bind(name)
    .bind(900_i64)
    .bind(604800_i64)
    .execute(&**pool)
    .await?;
    Ok(())
}

async fn insert_user(pool: &Database, user_id: Uuid, realm_id: Uuid, username: &str) -> Result<()> {
    sqlx::query("INSERT INTO users (id, realm_id, username, hashed_password) VALUES (?, ?, ?, ?)")
        .bind(user_id.to_string())
        .bind(realm_id.to_string())
        .bind(username)
        .bind("hashed")
        .execute(&**pool)
        .await?;
    Ok(())
}

async fn insert_client(
    pool: &Database,
    client_id: Uuid,
    realm_id: Uuid,
    client_key: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO oidc_clients (id, realm_id, client_id, client_secret, redirect_uris, web_origins, scopes) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(client_id.to_string())
    .bind(realm_id.to_string())
    .bind(client_key)
    .bind("secret")
    .bind("[]")
    .bind("[]")
    .bind("openid")
    .execute(&**pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn role_crud_and_listing() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteRbacRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();
    insert_realm(&db.pool, realm_id, "realm-one").await?;

    let role_admin = role(Uuid::new_v4(), realm_id, None, "admin");
    let role_viewer = role(Uuid::new_v4(), realm_id, None, "viewer");

    repo.create_role(&role_admin, None).await?;
    repo.create_role(&role_viewer, None).await?;

    let found = repo.find_role_by_id(&role_admin.id).await?;
    assert_eq!(found.unwrap().name, "admin");

    let found_by_name = repo.find_role_by_name(&realm_id, "viewer").await?;
    assert_eq!(found_by_name.unwrap().id, role_viewer.id);

    let req = page_request(1, 10, Some("name"), Some(SortDirection::Asc), Some("view"));
    let page = repo.list_roles(&realm_id, &req).await?;
    assert_eq!(page.meta.total, 1);
    assert_eq!(page.data[0].id, role_viewer.id);

    let mut updated = role_viewer.clone();
    updated.name = "viewer-updated".to_string();
    updated.description = Some("updated".to_string());
    repo.update_role(&updated, None).await?;

    let fetched = repo.find_role_by_id(&role_viewer.id).await?.unwrap();
    assert_eq!(fetched.name, "viewer-updated");

    repo.delete_role(&role_viewer.id, None).await?;
    let missing = repo.find_role_by_id(&role_viewer.id).await?;
    assert!(missing.is_none());

    let err = repo.delete_role(&role_viewer.id, None).await.unwrap_err();
    assert!(matches!(err, Error::NotFound(_)));
    Ok(())
}

#[tokio::test]
async fn client_role_listing() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteRbacRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();
    insert_realm(&db.pool, realm_id, "realm-client").await?;

    let client_id = Uuid::new_v4();
    insert_client(&db.pool, client_id, realm_id, "web-client").await?;

    let client_role = role(Uuid::new_v4(), realm_id, Some(client_id), "client-admin");
    repo.create_role(&client_role, None).await?;

    let req = page_request(1, 10, Some("name"), Some(SortDirection::Asc), None);
    let page = repo.list_client_roles(&realm_id, &client_id, &req).await?;
    assert_eq!(page.meta.total, 1);
    assert_eq!(page.data[0].id, client_role.id);

    let global_roles = repo.list_roles(&realm_id, &req).await?;
    assert_eq!(global_roles.meta.total, 0);
    Ok(())
}

#[tokio::test]
async fn group_hierarchy_and_ordering() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteRbacRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();
    insert_realm(&db.pool, realm_id, "realm-groups").await?;

    let root_a = group(Uuid::new_v4(), realm_id, None, "root-a", 0);
    let root_b = group(Uuid::new_v4(), realm_id, None, "root-b", 1);
    let child_a = group(Uuid::new_v4(), realm_id, Some(root_a.id), "child-a", 0);

    repo.create_group(&root_a, None).await?;
    repo.create_group(&root_b, None).await?;
    repo.create_group(&child_a, None).await?;

    let found_by_name = repo.find_group_by_name(&realm_id, "root-a").await?;
    assert_eq!(found_by_name.unwrap().id, root_a.id);
    let found_by_id = repo.find_group_by_id(&root_b.id).await?;
    assert_eq!(found_by_id.unwrap().name, "root-b");

    let search_req = page_request(1, 10, Some("name"), Some(SortDirection::Asc), Some("root"));
    let filtered_groups = repo.list_groups(&realm_id, &search_req).await?;
    assert_eq!(filtered_groups.meta.total, 2);

    let req = page_request(1, 10, Some("name"), Some(SortDirection::Asc), None);
    let groups = repo.list_groups(&realm_id, &req).await?;
    assert_eq!(groups.meta.total, 3);

    let roots_filtered = repo.list_group_roots(&realm_id, &search_req).await?;
    assert_eq!(roots_filtered.meta.total, 2);

    let roots = repo.list_group_roots(&realm_id, &req).await?;
    let root_ids: HashSet<Uuid> = roots.data.iter().map(|row| row.id).collect();
    assert!(root_ids.contains(&root_a.id));
    assert!(root_ids.contains(&root_b.id));
    let root_a_row = roots
        .data
        .iter()
        .find(|row| row.id == root_a.id)
        .expect("root-a row");
    assert!(root_a_row.has_children);

    let child_search_req =
        page_request(1, 10, Some("name"), Some(SortDirection::Asc), Some("child"));
    let children_filtered = repo
        .list_group_children(&realm_id, &root_a.id, &child_search_req)
        .await?;
    assert_eq!(children_filtered.meta.total, 1);

    let children = repo
        .list_group_children(&realm_id, &root_a.id, &req)
        .await?;
    assert_eq!(children.meta.total, 1);
    assert_eq!(children.data[0].id, child_a.id);

    let root_ids_ordered = repo.list_group_ids_by_parent(&realm_id, None).await?;
    assert_eq!(root_ids_ordered, vec![root_a.id, root_b.id]);

    let next_sort = repo
        .get_next_group_sort_order(&realm_id, Some(&root_a.id))
        .await?;
    assert_eq!(next_sort, 1);

    let is_descendant = repo
        .is_group_descendant(&realm_id, &root_a.id, &child_a.id)
        .await?;
    assert!(is_descendant);

    let subtree_ids = repo.list_group_subtree_ids(&realm_id, &root_a.id).await?;
    let subtree_set: HashSet<Uuid> = subtree_ids.into_iter().collect();
    assert!(subtree_set.contains(&root_a.id));
    assert!(subtree_set.contains(&child_a.id));

    repo.set_group_orders(&realm_id, None, &[root_b.id, root_a.id], None)
        .await?;
    let reordered = repo.list_group_ids_by_parent(&realm_id, None).await?;
    assert_eq!(reordered, vec![root_b.id, root_a.id]);

    let mut updated_root = root_a.clone();
    updated_root.name = "root-a-updated".to_string();
    updated_root.description = None;
    repo.update_group(&updated_root, None).await?;
    let updated = repo.find_group_by_id(&root_a.id).await?.unwrap();
    assert_eq!(updated.name, "root-a-updated");
    assert!(updated.description.is_none());

    repo.delete_groups(&[], None).await?;
    repo.delete_groups(&[root_b.id], None).await?;
    let deleted = repo.find_group_by_id(&root_b.id).await?;
    assert!(deleted.is_none());
    Ok(())
}

#[tokio::test]
async fn custom_permissions_and_role_permissions() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteRbacRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();
    insert_realm(&db.pool, realm_id, "realm-perms").await?;

    let user_id = Uuid::new_v4();
    insert_user(&db.pool, user_id, realm_id, "creator").await?;

    let role_one = role(Uuid::new_v4(), realm_id, None, "alpha");
    let role_two = role(Uuid::new_v4(), realm_id, None, "beta");
    repo.create_role(&role_one, None).await?;
    repo.create_role(&role_two, None).await?;

    repo.assign_permission_to_role(&"perm.read".to_string(), &role_one.id, None)
        .await?;
    let perms = repo.get_permissions_for_role(&role_one.id).await?;
    assert_eq!(perms, vec!["perm.read".to_string()]);

    repo.remove_permission(&role_one.id, "perm.read", None)
        .await?;
    let perms = repo.get_permissions_for_role(&role_one.id).await?;
    assert!(perms.is_empty());

    repo.bulk_update_permissions(
        &role_one.id,
        vec!["perm.add".to_string(), "perm.remove".to_string()],
        "add",
        None,
    )
    .await?;
    let perms = repo.get_permissions_for_role(&role_one.id).await?;
    let perm_set: HashSet<Permission> = perms.into_iter().collect();
    assert!(perm_set.contains("perm.add"));
    assert!(perm_set.contains("perm.remove"));

    repo.bulk_update_permissions(
        &role_one.id,
        vec!["perm.remove".to_string()],
        "remove",
        None,
    )
    .await?;
    let perms = repo.get_permissions_for_role(&role_one.id).await?;
    assert_eq!(perms, vec!["perm.add".to_string()]);

    repo.assign_permission_to_role(&"perm.shared".to_string(), &role_one.id, None)
        .await?;
    repo.assign_permission_to_role(&"perm.shared".to_string(), &role_two.id, None)
        .await?;
    repo.remove_role_permissions_by_key("perm.shared", None)
        .await?;
    let perms_one = repo.get_permissions_for_role(&role_one.id).await?;
    let perms_two = repo.get_permissions_for_role(&role_two.id).await?;
    assert!(!perms_one.contains(&"perm.shared".to_string()));
    assert!(!perms_two.contains(&"perm.shared".to_string()));

    let custom_one = custom_permission(
        Uuid::new_v4(),
        realm_id,
        None,
        "custom.read",
        "Custom Read",
        Some(user_id),
    );
    let custom_two = custom_permission(
        Uuid::new_v4(),
        realm_id,
        None,
        "custom.write",
        "Custom Write",
        Some(user_id),
    );
    repo.create_custom_permission(&custom_one, None).await?;
    repo.create_custom_permission(&custom_two, None).await?;

    let by_key = repo
        .find_custom_permission_by_key(&realm_id, None, "custom.read")
        .await?;
    assert_eq!(by_key.unwrap().id, custom_one.id);

    let by_id = repo
        .find_custom_permission_by_id(&realm_id, &custom_two.id)
        .await?;
    assert_eq!(by_id.unwrap().permission, "custom.write");

    let list = repo.list_custom_permissions(&realm_id, None).await?;
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].permission, "custom.read");

    let mut updated = custom_two.clone();
    updated.name = "Custom Write Updated".to_string();
    updated.description = None;
    repo.update_custom_permission(&updated, None).await?;

    let refreshed = repo
        .find_custom_permission_by_id(&realm_id, &custom_two.id)
        .await?
        .expect("updated custom permission");
    assert_eq!(refreshed.name, "Custom Write Updated");
    assert!(refreshed.description.is_none());

    repo.delete_custom_permission(&custom_one.id, None).await?;
    let deleted = repo
        .find_custom_permission_by_id(&realm_id, &custom_one.id)
        .await?;
    assert!(deleted.is_none());
    Ok(())
}

#[tokio::test]
async fn membership_and_composites_queries() -> Result<()> {
    let db = TestDb::new().await;
    let repo = SqliteRbacRepository::new(db.pool.clone());
    let realm_id = Uuid::new_v4();
    insert_realm(&db.pool, realm_id, "realm-members").await?;

    let u1 = Uuid::new_v4();
    let u2 = Uuid::new_v4();
    let u3 = Uuid::new_v4();
    insert_user(&db.pool, u1, realm_id, "alice").await?;
    insert_user(&db.pool, u2, realm_id, "bob").await?;
    insert_user(&db.pool, u3, realm_id, "carol").await?;

    let g1 = group(Uuid::new_v4(), realm_id, None, "group-a", 0);
    repo.create_group(&g1, None).await?;

    let r1 = role(Uuid::new_v4(), realm_id, None, "role-1");
    let r2 = role(Uuid::new_v4(), realm_id, None, "role-2");
    let r3 = role(Uuid::new_v4(), realm_id, None, "role-3");
    let r4 = role(Uuid::new_v4(), realm_id, None, "role-4");
    let r5 = role(Uuid::new_v4(), realm_id, None, "role-5");
    for role in [&r1, &r2, &r3, &r4, &r5] {
        repo.create_role(role, None).await?;
    }

    repo.assign_composite_role(&r1.id, &r2.id, None).await?;
    repo.assign_composite_role(&r2.id, &r3.id, None).await?;
    repo.assign_composite_role(&r3.id, &r4.id, None).await?;

    repo.assign_role_to_user(&u1, &r1.id, None).await?;
    repo.assign_user_to_group(&u2, &g1.id, None).await?;
    repo.assign_role_to_group(&r2.id, &g1.id, None).await?;

    repo.assign_permission_to_role(&"perm.r2".to_string(), &r2.id, None)
        .await?;
    repo.assign_permission_to_role(&"perm.r3".to_string(), &r3.id, None)
        .await?;
    repo.assign_permission_to_role(&"perm.r4".to_string(), &r4.id, None)
        .await?;

    let req = page_request(1, 50, Some("name"), Some(SortDirection::Asc), None);

    let role_members = repo
        .list_role_members(&realm_id, &r1.id, RoleMemberFilter::All, &req)
        .await?;
    let role_member_map: HashMap<Uuid, (bool, bool)> = role_members
        .data
        .iter()
        .map(|row| (row.id, (row.is_direct, row.is_effective)))
        .collect();
    assert_eq!(role_member_map.len(), 3);
    assert_eq!(role_member_map.get(&u1), Some(&(true, true)));
    assert_eq!(role_member_map.get(&u2), Some(&(false, true)));
    assert_eq!(role_member_map.get(&u3), Some(&(false, false)));

    let direct_members = repo
        .list_role_members(&realm_id, &r1.id, RoleMemberFilter::Direct, &req)
        .await?;
    assert_eq!(direct_members.meta.total, 1);
    assert_eq!(direct_members.data[0].id, u1);

    let effective_members = repo
        .list_role_members(&realm_id, &r1.id, RoleMemberFilter::Effective, &req)
        .await?;
    assert_eq!(effective_members.meta.total, 1);
    assert_eq!(effective_members.data[0].id, u2);

    let unassigned_members = repo
        .list_role_members(&realm_id, &r1.id, RoleMemberFilter::Unassigned, &req)
        .await?;
    assert_eq!(unassigned_members.meta.total, 1);
    assert_eq!(unassigned_members.data[0].id, u3);

    let group_members = repo
        .list_group_members(&realm_id, &g1.id, GroupMemberFilter::Members, &req)
        .await?;
    assert_eq!(group_members.meta.total, 1);
    assert_eq!(group_members.data[0].id, u2);

    let group_non_members = repo
        .list_group_members(&realm_id, &g1.id, GroupMemberFilter::NonMembers, &req)
        .await?;
    let non_member_ids: HashSet<Uuid> = group_non_members.data.iter().map(|row| row.id).collect();
    assert_eq!(non_member_ids.len(), 2);
    assert!(non_member_ids.contains(&u1));
    assert!(non_member_ids.contains(&u3));

    let group_roles_direct = repo
        .list_group_roles(&realm_id, &g1.id, GroupRoleFilter::Direct, &req)
        .await?;
    assert_eq!(group_roles_direct.meta.total, 1);
    assert_eq!(group_roles_direct.data[0].id, r2.id);

    let group_roles_effective = repo
        .list_group_roles(&realm_id, &g1.id, GroupRoleFilter::Effective, &req)
        .await?;
    let effective_ids: HashSet<Uuid> = group_roles_effective
        .data
        .iter()
        .map(|row| row.id)
        .collect();
    assert_eq!(effective_ids.len(), 2);
    assert!(effective_ids.contains(&r3.id));
    assert!(effective_ids.contains(&r4.id));

    let group_roles_unassigned = repo
        .list_group_roles(&realm_id, &g1.id, GroupRoleFilter::Unassigned, &req)
        .await?;
    let unassigned_ids: HashSet<Uuid> = group_roles_unassigned
        .data
        .iter()
        .map(|row| row.id)
        .collect();
    assert!(unassigned_ids.contains(&r1.id));
    assert!(unassigned_ids.contains(&r5.id));

    let user_roles_effective = repo
        .list_user_roles(&realm_id, &u2, UserRoleFilter::Effective, &req)
        .await?;
    let effective_ids: HashSet<Uuid> = user_roles_effective.data.iter().map(|row| row.id).collect();
    assert_eq!(effective_ids.len(), 3);
    assert!(effective_ids.contains(&r2.id));
    assert!(effective_ids.contains(&r3.id));
    assert!(effective_ids.contains(&r4.id));

    let user_roles_unassigned = repo
        .list_user_roles(&realm_id, &u2, UserRoleFilter::Unassigned, &req)
        .await?;
    let unassigned_ids: HashSet<Uuid> = user_roles_unassigned
        .data
        .iter()
        .map(|row| row.id)
        .collect();
    assert!(unassigned_ids.contains(&r1.id));
    assert!(unassigned_ids.contains(&r5.id));

    let composites_direct = repo
        .list_role_composites(&realm_id, &r2.id, &None, RoleCompositeFilter::Direct, &req)
        .await?;
    assert_eq!(composites_direct.meta.total, 1);
    assert_eq!(composites_direct.data[0].id, r3.id);

    let composites_effective = repo
        .list_role_composites(
            &realm_id,
            &r2.id,
            &None,
            RoleCompositeFilter::Effective,
            &req,
        )
        .await?;
    assert_eq!(composites_effective.meta.total, 1);
    assert_eq!(composites_effective.data[0].id, r4.id);

    let composite_ids = repo.list_role_composite_ids(&r2.id).await?;
    assert_eq!(composite_ids, vec![r3.id]);
    let composite_effective = repo.list_effective_role_composite_ids(&r2.id).await?;
    let composite_effective_set: HashSet<Uuid> = composite_effective.into_iter().collect();
    assert!(composite_effective_set.contains(&r3.id));
    assert!(composite_effective_set.contains(&r4.id));

    let role_ids_for_group = repo.find_role_ids_for_group(&g1.id).await?;
    assert_eq!(role_ids_for_group, vec![r2.id]);
    let effective_roles_for_group = repo.find_effective_role_ids_for_group(&g1.id).await?;
    let effective_set: HashSet<Uuid> = effective_roles_for_group.into_iter().collect();
    assert!(effective_set.contains(&r2.id));
    assert!(effective_set.contains(&r3.id));
    assert!(effective_set.contains(&r4.id));

    let user_ids_for_group = repo.find_user_ids_in_group(&g1.id).await?;
    assert_eq!(user_ids_for_group, vec![u2]);

    let user_ids = repo.find_user_ids_in_groups(&[g1.id]).await?;
    assert_eq!(user_ids, vec![u2]);

    let user_count = repo.count_user_ids_in_groups(&[g1.id]).await?;
    assert_eq!(user_count, 1);

    let role_count = repo.count_role_ids_in_groups(&[g1.id]).await?;
    assert_eq!(role_count, 1);

    let direct_role_ids = repo.find_direct_role_ids_for_user(&u1).await?;
    assert_eq!(direct_role_ids, vec![r1.id]);

    let effective_role_ids = repo.find_effective_role_ids_for_user(&u2).await?;
    let effective_role_ids_set: HashSet<Uuid> = effective_role_ids.into_iter().collect();
    assert!(effective_role_ids_set.contains(&r2.id));
    assert!(effective_role_ids_set.contains(&r3.id));
    assert!(effective_role_ids_set.contains(&r4.id));

    let group_role_ids = repo.find_role_ids_for_user(&u2).await?;
    assert_eq!(group_role_ids, vec![r2.id]);

    let permissions = repo.find_permissions_for_roles(&[r2.id]).await?;
    let perm_set: HashSet<String> = permissions.into_iter().collect();
    assert!(perm_set.contains("perm.r2"));
    assert!(perm_set.contains("perm.r3"));
    assert!(perm_set.contains("perm.r4"));

    let effective_permissions = repo.get_effective_permissions_for_user(&u2).await?;
    assert!(effective_permissions.contains("perm.r2"));
    assert!(effective_permissions.contains("perm.r3"));
    assert!(effective_permissions.contains("perm.r4"));

    let role_names = repo.find_role_names_for_user(&u2).await?;
    let role_name_set: HashSet<String> = role_names.into_iter().collect();
    assert!(role_name_set.contains("role-2"));
    assert!(role_name_set.contains("role-3"));
    assert!(role_name_set.contains("role-4"));

    let group_names = repo.find_group_names_for_user(&u2).await?;
    assert_eq!(group_names, vec!["group-a".to_string()]);

    let user_ids_for_role = repo.find_user_ids_for_role(&r1.id).await?;
    let users_for_role: HashSet<Uuid> = user_ids_for_role.into_iter().collect();
    assert!(users_for_role.contains(&u1));
    assert!(users_for_role.contains(&u2));

    let direct_user_ids = repo.find_direct_user_ids_for_role(&r1.id).await?;
    assert_eq!(direct_user_ids, vec![u1]);

    let descendant = repo.is_role_descendant(&r1.id, &r4.id).await?;
    assert!(descendant);
    repo.remove_composite_role(&r2.id, &r3.id, None).await?;
    let descendant_after = repo.is_role_descendant(&r1.id, &r4.id).await?;
    assert!(!descendant_after);

    repo.remove_role_from_group(&r2.id, &g1.id, None).await?;
    let group_roles_after = repo
        .list_group_roles(&realm_id, &g1.id, GroupRoleFilter::Direct, &req)
        .await?;
    assert_eq!(group_roles_after.meta.total, 0);

    repo.remove_user_from_group(&u2, &g1.id, None).await?;
    let group_members_after = repo
        .list_group_members(&realm_id, &g1.id, GroupMemberFilter::Members, &req)
        .await?;
    assert_eq!(group_members_after.meta.total, 0);

    repo.remove_role_from_user(&u1, &r1.id, None).await?;
    let direct_members_after = repo
        .list_role_members(&realm_id, &r1.id, RoleMemberFilter::Direct, &req)
        .await?;
    assert_eq!(direct_members_after.meta.total, 0);

    Ok(())
}
