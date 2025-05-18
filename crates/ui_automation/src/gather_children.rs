use crate::EndOfSiblings;
use crate::GatherChildrenStopBehaviourFn;
use crate::LastChildEncountered;
use crate::RootEndEncountered;
use crate::StopBehaviour;
use crate::TaskbarEndEncountered;
use std::collections::VecDeque;
use uiautomation::UIElement;
use uiautomation::UITreeWalker;

pub trait GatherChildrenable {
    fn gather_children(
        &self,
        walker: &UITreeWalker,
        stop_behaviour: &StopBehaviour,
    ) -> VecDeque<UIElement>;
}
impl GatherChildrenable for UIElement {
    fn gather_children(
        &self,
        walker: &UITreeWalker,
        stop_behaviour: &StopBehaviour,
    ) -> VecDeque<UIElement> {
        gather_children(walker, self, stop_behaviour)
    }
}

pub fn gather_children(
    walker: &UITreeWalker,
    parent: &UIElement,
    stop_behaviour: &StopBehaviour,
) -> VecDeque<UIElement> {
    // println!("Gathering children of {:?}", parent);
    let mut children = VecDeque::new();

    // println!("Constructing stop behaviour fn for {:?}", stop_behaviour);

    let stop: Box<dyn GatherChildrenStopBehaviourFn> = match stop_behaviour {
        StopBehaviour::EndOfSiblings => Box::new(EndOfSiblings),
        StopBehaviour::LastChildEncountered => {
            // println!("Getting last child of {:?}", parent);
            let last = walker.get_last_child(parent);
            let last = match last {
                Ok(last) => last,
                Err(_) => {
                    eprintln!("Failed to get last child of {:?}", parent);
                    return children;
                }
            };
            let runtime_id_of_last = last.get_runtime_id();
            let runtime_id_of_last = match runtime_id_of_last {
                Ok(runtime_id_of_last) => runtime_id_of_last,
                Err(_) => {
                    eprintln!(
                        "Failed to get runtime id of last child {:?} of {:?}",
                        last, parent
                    );
                    return children;
                }
            };
            Box::new(LastChildEncountered { runtime_id_of_last })
        }
        StopBehaviour::TaskbarEndEncountered => Box::new(TaskbarEndEncountered),
        StopBehaviour::RootEndEncountered => Box::new(RootEndEncountered),
    };

    // println!("Constructed stop behaviour {:?}", stop_behaviour);

    // println!("Finding first child");
    let first = walker.get_first_child(parent);
    // println!("Found first child");

    let Ok(first) = first else {
        return children;
    };
    children.push_back(first.clone());
    let mut next = first;
    loop {
        // println!("About to grab next sibling of {:?}", next);
        let sibling = walker.get_next_sibling(&next);

        if let Ok(sibling) = sibling {
            // println!("Got sibling {:?}", sibling);
            // println!("Checking if should stop");
            if stop.should_stop(&sibling) {
                // println!("Should stop");
                if stop_behaviour.include_last_child() {
                    // println!("Including last child");
                    children.push_back(sibling.clone());
                }
                break;
            } else {
                // println!("Should not stop");
                children.push_back(sibling.clone());
                next = sibling;
            }
        } else {
            break;
        }
    }
    // println!("Gathered {} children", children.len());
    // println!("| {}", metrics.report().split(" | ").join("\n| "));
    children
}
