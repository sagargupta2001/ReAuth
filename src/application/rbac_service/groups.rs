use super::CreateGroupPayload;
use super::RbacService;
use crate::domain::events::DomainEvent;
use crate::domain::group::Group;
use crate::domain::pagination::{PageRequest, PageResponse};
use crate::domain::rbac::{GroupDeleteSummary, GroupTreeRow};
use crate::error::{Error, Result};
use uuid::Uuid;

impl RbacService {
    // --- Group Operations ---
    pub async fn create_group(&self, realm_id: Uuid, payload: CreateGroupPayload) -> Result<Group> {
        if self
            .rbac_repo
            .find_group_by_name(&realm_id, &payload.name)
            .await?
            .is_some()
        {
            return Err(Error::GroupAlreadyExists);
        }

        if let Some(parent_id) = payload.parent_id {
            let _ = self.get_group(realm_id, parent_id).await?;
        }

        let sort_order = self
            .rbac_repo
            .get_next_group_sort_order(&realm_id, payload.parent_id.as_ref())
            .await?;

        let group = Group {
            id: Uuid::new_v4(),
            realm_id,
            parent_id: payload.parent_id,
            name: payload.name,
            description: payload.description,
            sort_order,
        };
        self.rbac_repo.create_group(&group, None).await?;
        Ok(group)
    }

    pub async fn list_groups(
        &self,
        realm_id: Uuid,
        req: PageRequest,
    ) -> Result<PageResponse<Group>> {
        self.rbac_repo.list_groups(&realm_id, &req).await
    }

    pub async fn list_group_roots(
        &self,
        realm_id: Uuid,
        req: PageRequest,
    ) -> Result<PageResponse<GroupTreeRow>> {
        self.rbac_repo.list_group_roots(&realm_id, &req).await
    }

    pub async fn list_group_children(
        &self,
        realm_id: Uuid,
        parent_id: Uuid,
        req: PageRequest,
    ) -> Result<PageResponse<GroupTreeRow>> {
        let _ = self.get_group(realm_id, parent_id).await?;
        self.rbac_repo
            .list_group_children(&realm_id, &parent_id, &req)
            .await
    }

    pub async fn move_group(
        &self,
        realm_id: Uuid,
        group_id: Uuid,
        parent_id: Option<Uuid>,
        before_id: Option<Uuid>,
        after_id: Option<Uuid>,
    ) -> Result<()> {
        if before_id.is_some() && after_id.is_some() {
            return Err(Error::Validation(
                "Provide only one of before_id or after_id.".into(),
            ));
        }

        let group = self.get_group(realm_id, group_id).await?;

        if let Some(parent_id) = parent_id {
            if parent_id == group_id {
                return Err(Error::Validation("Group cannot be its own parent.".into()));
            }

            let _ = self.get_group(realm_id, parent_id).await?;

            if self
                .rbac_repo
                .is_group_descendant(&realm_id, &group_id, &parent_id)
                .await?
            {
                return Err(Error::Validation(
                    "Cannot move a group inside its own subtree.".into(),
                ));
            }
        }

        if let Some(before_id) = before_id {
            let before_group = self.get_group(realm_id, before_id).await?;
            if before_group.parent_id != parent_id {
                return Err(Error::Validation(
                    "before_id must be a sibling under the target parent.".into(),
                ));
            }
        }

        if let Some(after_id) = after_id {
            let after_group = self.get_group(realm_id, after_id).await?;
            if after_group.parent_id != parent_id {
                return Err(Error::Validation(
                    "after_id must be a sibling under the target parent.".into(),
                ));
            }
        }

        let mut siblings = self
            .rbac_repo
            .list_group_ids_by_parent(&realm_id, parent_id.as_ref())
            .await?;

        siblings.retain(|id| id != &group_id);

        let insert_index = if let Some(before_id) = before_id {
            siblings
                .iter()
                .position(|id| id == &before_id)
                .ok_or_else(|| Error::Validation("before_id not found.".into()))?
        } else if let Some(after_id) = after_id {
            let pos = siblings
                .iter()
                .position(|id| id == &after_id)
                .ok_or_else(|| Error::Validation("after_id not found.".into()))?;
            pos + 1
        } else {
            siblings.len()
        };

        siblings.insert(insert_index, group_id);

        self.rbac_repo
            .set_group_orders(&realm_id, parent_id.as_ref(), &siblings, None)
            .await?;

        if group.parent_id != parent_id {
            let mut old_siblings = self
                .rbac_repo
                .list_group_ids_by_parent(&realm_id, group.parent_id.as_ref())
                .await?;
            old_siblings.retain(|id| id != &group_id);
            self.rbac_repo
                .set_group_orders(&realm_id, group.parent_id.as_ref(), &old_siblings, None)
                .await?;
        }

        Ok(())
    }

    pub async fn get_group(&self, realm_id: Uuid, group_id: Uuid) -> Result<Group> {
        let group = self
            .rbac_repo
            .find_group_by_id(&group_id)
            .await?
            .ok_or(Error::NotFound("Group not found".into()))?;

        if group.realm_id != realm_id {
            return Err(Error::SecurityViolation(
                "Group belongs to different realm".into(),
            ));
        }

        Ok(group)
    }

    pub async fn update_group(
        &self,
        realm_id: Uuid,
        group_id: Uuid,
        payload: CreateGroupPayload,
    ) -> Result<Group> {
        let mut group = self.get_group(realm_id, group_id).await?;

        group.name = payload.name;
        group.description = payload.description;

        self.rbac_repo.update_group(&group, None).await?;

        Ok(group)
    }

    pub async fn get_group_delete_summary(
        &self,
        realm_id: Uuid,
        group_id: Uuid,
    ) -> Result<GroupDeleteSummary> {
        let group = self.get_group(realm_id, group_id).await?;
        let subtree_ids = self
            .rbac_repo
            .list_group_subtree_ids(&realm_id, &group_id)
            .await?;

        let direct_children_count = self
            .rbac_repo
            .list_group_ids_by_parent(&realm_id, Some(&group_id))
            .await?
            .len() as i64;

        let descendant_count = subtree_ids.len().saturating_sub(1) as i64;
        let member_count = self
            .rbac_repo
            .count_user_ids_in_groups(&subtree_ids)
            .await?;
        let role_count = self
            .rbac_repo
            .count_role_ids_in_groups(&subtree_ids)
            .await?;

        Ok(GroupDeleteSummary {
            group_id,
            name: group.name,
            direct_children_count,
            descendant_count,
            member_count,
            role_count,
        })
    }

    pub async fn delete_group(&self, realm_id: Uuid, group_id: Uuid, cascade: bool) -> Result<()> {
        let _ = self.get_group(realm_id, group_id).await?;

        let direct_children = self
            .rbac_repo
            .list_group_ids_by_parent(&realm_id, Some(&group_id))
            .await?;
        if !cascade && !direct_children.is_empty() {
            return Err(Error::Validation(
                "Group has child groups. Use cascade delete to remove the subtree.".into(),
            ));
        }

        let group_ids = if cascade {
            self.rbac_repo
                .list_group_subtree_ids(&realm_id, &group_id)
                .await?
        } else {
            vec![group_id]
        };

        let affected_users = self.rbac_repo.find_user_ids_in_groups(&group_ids).await?;

        let event = DomainEvent::GroupDeleted(crate::domain::events::GroupDeleted {
            group_ids: group_ids.clone(),
            affected_user_ids: affected_users,
        });

        let mut tx = self.tx_manager.begin().await?;
        let result = async {
            self.rbac_repo
                .delete_groups(&group_ids, Some(&mut *tx))
                .await?;
            self.write_outbox(&event, Some(realm_id), &mut *tx).await?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => {
                self.tx_manager.commit(tx).await?;
                self.event_bus.publish(event).await;
            }
            Err(err) => {
                self.tx_manager.rollback(tx).await?;
                return Err(err);
            }
        }

        Ok(())
    }
}
