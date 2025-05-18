use crate::DrillId;
use crate::ElementInfo;

pub fn update_drill_ids(children: Option<&mut Vec<ElementInfo>>, ancestor_path: &DrillId) {
    let Some(children) = children else { return };
    for child_info in children.iter_mut() {
        // Check if the child has a base drill_id set
        if let DrillId::Child(base_drill_id) = &child_info.drill_id {
            let mut new_path = ancestor_path.clone();
            if let Some(&child_position) = base_drill_id.back() {
                new_path = match new_path {
                    DrillId::Root | DrillId::Unknown => DrillId::Child(vec![child_position].into()),
                    DrillId::Child(ref mut path) => {
                        let mut new_path = path.clone();
                        new_path.push_back(child_position);
                        DrillId::Child(new_path)
                    }
                };

                // Update the child's drill_id by concatenating the ancestor_path with its own position
                child_info.drill_id = new_path.clone();
            }

            // Recursively update this child's children
            update_drill_ids(child_info.children.as_mut(), &new_path);
        }
    }
}
