use crate::DrillId;
use crate::ElementInfo;
use eyre::bail;

pub trait HasChildren {
    fn children(&self) -> impl IntoIterator<Item = &Self>;
    fn children_mut(&mut self) -> impl IntoIterator<Item = &mut Self>;
}
pub trait HasDrillId {
    fn drill_id(&self) -> &DrillId;
    fn drill_id_mut(&mut self) -> &mut DrillId;
}
impl HasChildren for ElementInfo {
    fn children(&self) -> impl IntoIterator<Item = &Self> {
        self.children.iter().flat_map(|children| children.iter())
    }
    fn children_mut(&mut self) -> impl IntoIterator<Item = &mut Self> {
        self.children
            .iter_mut()
            .flat_map(|children| children.iter_mut())
    }
}
impl HasDrillId for ElementInfo {
    fn drill_id(&self) -> &DrillId {
        &self.drill_id
    }
    fn drill_id_mut(&mut self) -> &mut DrillId {
        &mut self.drill_id
    }
}

/// This fn assumes that no children have been omitted
pub fn update_drill_ids<T: HasChildren + HasDrillId>(root: &mut T) -> eyre::Result<()> {
    let mut to_process = Vec::new();
    match root.drill_id() {
        DrillId::Unknown => bail!("Cannot update drill IDs on an unknown drill ID"),
        DrillId::Root => {
            for (root_child_index, root_child) in root.children_mut().into_iter().enumerate() {
                *root_child.drill_id_mut() = [root_child_index].into();
                to_process.push(root_child);
            }
        }
        DrillId::Path(_) => {
            to_process.push(root);
        }
    }
    while let Some(parent) = to_process.pop() {
        let parent_drill_id = parent.drill_id().to_owned();
        for (child_index, child) in parent.children_mut().into_iter().enumerate() {
            *child.drill_id_mut() = parent_drill_id.try_join([child_index])?;
            to_process.push(child);
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::DrillId;
    use crate::ElementInfo;
    use crate::update_drill_ids;

    #[test]
    fn simple_parent_child() -> eyre::Result<()> {
        let mut root = ElementInfo::default();
        root.drill_id = DrillId::Root;
        let child1 = ElementInfo::default();
        let child2 = ElementInfo::default();
        let child3 = ElementInfo::default();
        root.children = Some([child1, child2, child3].into());
        update_drill_ids(&mut root)?;
        assert_eq!(root.drill_id, DrillId::Root);
        assert_eq!(
            root.children.as_ref().unwrap()[0].drill_id,
            DrillId::Path([0].into())
        );
        assert_eq!(
            root.children.as_ref().unwrap()[1].drill_id,
            DrillId::Path([1].into())
        );
        assert_eq!(
            root.children.as_ref().unwrap()[2].drill_id,
            DrillId::Path([2].into())
        );

        Ok(())
    }
    #[test]
    fn halfway_down() -> eyre::Result<()> {
        let mut root = ElementInfo::default();
        root.drill_id = [2, 3].into();
        let child1 = ElementInfo::default();
        let child2 = ElementInfo::default();
        let child3 = ElementInfo::default();
        root.children = Some([child1, child2, child3].into());
        update_drill_ids(&mut root)?;
        assert_eq!(root.drill_id, DrillId::Root);
        assert_eq!(
            root.children.as_ref().unwrap()[0].drill_id,
            [2, 3, 0].into()
        );
        assert_eq!(
            root.children.as_ref().unwrap()[1].drill_id,
            [2, 3, 1].into()
        );
        assert_eq!(
            root.children.as_ref().unwrap()[2].drill_id,
            [2, 3, 2].into()
        );

        Ok(())
    }
}
