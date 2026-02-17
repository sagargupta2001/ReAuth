use serde::Serialize;
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct RoleMemberRow {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    pub username: String,
    pub is_direct: bool,
    pub is_effective: bool,
}

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct GroupMemberRow {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    pub username: String,
    pub is_member: bool,
}

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct GroupRoleRow {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_direct: bool,
    pub is_effective: bool,
}

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct UserRoleRow {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_direct: bool,
    pub is_effective: bool,
}

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct RoleCompositeRow {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_direct: bool,
    pub is_effective: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct GroupTreeRow {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i64,
    pub has_children: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct GroupDeleteSummary {
    pub group_id: Uuid,
    pub name: String,
    pub direct_children_count: i64,
    pub descendant_count: i64,
    pub member_count: i64,
    pub role_count: i64,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for GroupTreeRow {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let parse_uuid = |val: String, col_name: &str| -> Result<Uuid, sqlx::Error> {
            Uuid::parse_str(&val).map_err(|e| sqlx::Error::ColumnDecode {
                index: col_name.into(),
                source: Box::new(e),
            })
        };

        let id_str: String = row.try_get("id")?;
        let parent_id_str: Option<String> = row.try_get("parent_id")?;
        let parent_id = match parent_id_str {
            Some(s) => Some(parse_uuid(s, "parent_id")?),
            None => None,
        };

        Ok(GroupTreeRow {
            id: parse_uuid(id_str, "id")?,
            parent_id,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            sort_order: row.try_get("sort_order")?,
            has_children: row.try_get("has_children")?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RoleMemberFilter {
    All,
    Direct,
    Effective,
    Unassigned,
}

#[derive(Debug, Clone, Copy)]
pub enum GroupMemberFilter {
    All,
    Members,
    NonMembers,
}

#[derive(Debug, Clone, Copy)]
pub enum GroupRoleFilter {
    All,
    Direct,
    Effective,
    Unassigned,
}

#[derive(Debug, Clone, Copy)]
pub enum UserRoleFilter {
    All,
    Direct,
    Effective,
    Unassigned,
}

#[derive(Debug, Clone, Copy)]
pub enum RoleCompositeFilter {
    All,
    Direct,
    Effective,
    Unassigned,
}
