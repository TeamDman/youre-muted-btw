use uiautomation::UIElement;

#[allow(dead_code)]
#[derive(Debug)]
pub enum StopBehaviour {
    EndOfSiblings,
    LastChildEncountered,
    TaskbarEndEncountered,
    RootEndEncountered, // Calling get_next_sibling on the last child of root will hang, so use this to mitigate
}
impl StopBehaviour {
    pub fn include_last_child(&self) -> bool {
        !matches!(self, StopBehaviour::TaskbarEndEncountered)
    }
}
pub trait GatherChildrenStopBehaviourFn {
    fn should_stop(&self, next: &UIElement) -> bool;
}

#[derive(Debug)]
pub struct EndOfSiblings;
impl GatherChildrenStopBehaviourFn for EndOfSiblings {
    fn should_stop(&self, _element: &UIElement) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct LastChildEncountered {
    pub runtime_id_of_last: Vec<i32>,
}
impl GatherChildrenStopBehaviourFn for LastChildEncountered {
    fn should_stop(&self, element: &UIElement) -> bool {
        element.get_runtime_id() == Ok(self.runtime_id_of_last.clone())
    }
}

#[derive(Debug)]
pub struct TaskbarEndEncountered;
impl GatherChildrenStopBehaviourFn for TaskbarEndEncountered {
    fn should_stop(&self, element: &UIElement) -> bool {
        element.get_automation_id() == Ok("TaskbarEndAccessibilityElement".to_string())
    }
}

#[derive(Debug)]
pub struct RootEndEncountered;
impl GatherChildrenStopBehaviourFn for RootEndEncountered {
    fn should_stop(&self, element: &UIElement) -> bool {
        element.get_name() == Ok("Program Manager".to_string())
            && element.get_classname() == Ok("Progman".to_string())
        // This could be more specific, but until a false positive is encountered, this is fine
    }
}
