use uiautomation::UIAutomation;
use uiautomation::controls::ControlType;
use ymb_ui_automation::Drillable;
use ymb_ui_automation::gather_single_element_info;

#[test]
fn from_drill_id() -> eyre::Result<()> {
    let automation = UIAutomation::new()?;
    let matcher = automation
        .create_matcher()
        .contains_name("Discord")
        .control_type(ControlType::Pane)
        .classname("Chrome_WidgetWin_1");
    let elem = matcher.find_first()?;
    println!("Located Discord window: {elem:#?}");
    let walker = automation.create_tree_walker()?;
    let drill_id = [0, 1, 1, 0, 1, 0, 0, 0, 1, 3, 1, 1];
    let drilled = elem.drill(&walker, drill_id)?;
    dbg!(&drilled);
    Ok(())
}
