use proptest::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

// Abstract representation of an RBAC system to test invariants
#[derive(Debug, Clone, Default)]
struct RbacGraph {
    user_roles: HashMap<Uuid, HashSet<Uuid>>,
    user_groups: HashMap<Uuid, HashSet<Uuid>>,
    group_roles: HashMap<Uuid, HashSet<Uuid>>,
    role_composites: HashMap<Uuid, HashSet<Uuid>>, // parent -> children
    group_parents: HashMap<Uuid, Option<Uuid>>,
}

impl RbacGraph {
    fn get_effective_roles_for_user(&self, user_id: &Uuid) -> HashSet<Uuid> {
        let mut roles = HashSet::new();

        // 1. Direct user roles
        if let Some(direct) = self.user_roles.get(user_id) {
            for r in direct {
                roles.insert(*r);
                self.collect_role_descendants(*r, &mut roles);
            }
        }

        // 2. Roles from groups
        if let Some(groups) = self.user_groups.get(user_id) {
            for g in groups {
                self.collect_group_roles(*g, &mut roles);
            }
        }

        roles
    }

    fn collect_role_descendants(&self, role_id: Uuid, roles: &mut HashSet<Uuid>) {
        if let Some(children) = self.role_composites.get(&role_id) {
            for child in children {
                if roles.insert(*child) {
                    self.collect_role_descendants(*child, roles);
                }
            }
        }
    }

    fn collect_group_roles(&self, group_id: Uuid, roles: &mut HashSet<Uuid>) {
        if let Some(g_roles) = self.group_roles.get(&group_id) {
            for r in g_roles {
                roles.insert(*r);
                self.collect_role_descendants(*r, roles);
            }
        }
        if let Some(Some(parent_id)) = self.group_parents.get(&group_id) {
            self.collect_group_roles(*parent_id, roles);
        }
    }
}

proptest! {
    #[test]
    fn direct_roles_are_always_subset_of_effective_roles(
        role_count in 0..15usize,
    ) {
        let mut graph = RbacGraph::default();
        let user_id = Uuid::new_v4();
        let mut direct_roles = HashSet::new();
        for _ in 0..role_count {
            direct_roles.insert(Uuid::new_v4());
        }
        graph.user_roles.insert(user_id, direct_roles.clone());

        let effective = graph.get_effective_roles_for_user(&user_id);

        for r in &direct_roles {
            prop_assert!(effective.contains(r), "Direct role must be in effective roles");
        }
    }

    #[test]
    fn composite_roles_expand_effective_permissions(
        parent_has_child in proptest::bool::ANY,
    ) {
        let mut graph = RbacGraph::default();
        let user_id = Uuid::new_v4();

        let parent_role = Uuid::new_v4();
        let child_role = Uuid::new_v4();

        let mut user_roles = HashSet::new();
        user_roles.insert(parent_role);
        graph.user_roles.insert(user_id, user_roles);

        if parent_has_child {
            let mut children = HashSet::new();
            children.insert(child_role);
            graph.role_composites.insert(parent_role, children);
        }

        let effective = graph.get_effective_roles_for_user(&user_id);

        prop_assert!(effective.contains(&parent_role));
        if parent_has_child {
            prop_assert!(effective.contains(&child_role));
        } else {
            prop_assert!(!effective.contains(&child_role));
        }
    }
}
