use crate::DrillId;
use crate::ElementInfo;
use crate::gather_single_element_info;
use eyre::Context;
use eyre::bail;
use eyre::eyre;
use std::collections::VecDeque;
use uiautomation::UIElement;
use uiautomation::UITreeWalker;

pub trait Drillable {
    fn drill<T: Into<DrillId>>(
        self,
        walker: &UITreeWalker,
        path: T,
    ) -> eyre::Result<VecDeque<(UIElement, ElementInfo)>>;
}
impl Drillable for UIElement {
    fn drill<T: Into<DrillId>>(
        self,
        walker: &UITreeWalker,
        path: T,
    ) -> eyre::Result<VecDeque<(UIElement, ElementInfo)>> {
        let drill_id = path.into();
        match drill_id {
            DrillId::Root => {
                let self_info = gather_single_element_info(&self)?;
                Ok([(self, self_info)].into())
            }
            DrillId::Unknown => bail!("Cannot drill using {}", drill_id),
            DrillId::Path(x) if x.is_empty() => {
                let self_info = gather_single_element_info(&self)?;
                Ok([(self, self_info)].into())
            }
            DrillId::Path(path) => {
                let mut rtn: VecDeque<(UIElement, ElementInfo)> = Default::default();
                let self_info = gather_single_element_info(&self)?;
                rtn.push_front((self, self_info));
                while rtn.len() <= path.len() {
                    let seeking_index = path[rtn.len() - 1];
                    let (parent, _parent_info) = rtn.back().unwrap();
                    let mut child = walker.get_first_child(parent).wrap_err_with(|| {
                        format!(
                            "Resolving {} failed when getting first child of {:?}\n{rtn:#?}",
                            DrillId::from(path.clone()).display_highlighted_index(rtn.len()),
                            parent,
                        )
                    })?;
                    for i in 0..seeking_index {
                        child = walker.get_next_sibling(&child).wrap_err_with(|| {
                            eyre!(
                                "Resolving {} failed when getting next (i={i}) sibling of {:?}\n{rtn:#?}",
                                DrillId::from(path.clone()).display_highlighted_index(rtn.len()),
                                child,
                            )
                        })?;
                    }
                    let mut child_info = gather_single_element_info(&child)?;
                    child_info.drill_id = path.iter().take(rtn.len()).cloned().collect();
                    rtn.push_back((child, child_info));
                }
                Ok(rtn)
            }
        }
    }
}
