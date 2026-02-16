use serde::Serialize;
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
    pub is_assigned: bool,
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
    Assigned,
    Unassigned,
}
