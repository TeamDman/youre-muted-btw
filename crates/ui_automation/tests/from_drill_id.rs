use uiautomation::UIAutomation;
use ymb_ui_automation::DrillId;
use ymb_ui_automation::GatherChildrenable;
use ymb_ui_automation::StopBehaviour;
use ymb_ui_automation::gather_info_tree;
use ymb_ui_automation::update_drill_ids;

#[test]
fn from_drill_id() -> eyre::Result<()> {
    let mut automation = UIAutomation::new()?;
    let matcher = automation.create_matcher().contains_name("Discord");
    let elem = matcher.find_first()?;

    let root = automation.get_root_element()?;
    let walker = automation.create_tree_walker()?;
    let tree = root.gather_children(&walker, &StopBehaviour::RootEndEncountered);
    update_drill_ids(Some(&mut tree), &DrillId::Root);

    dbg!(&tree);
    // // let drill_id = DrillId::Child([8, 0, 1, 1, 0, 1, 0, 0, 0, 1, 3, 1, 1].into());
    // let drill_id = DrillId::Child([8,0,1].into());
    // let elem = drill_id.resolve()?;
    // dbg!(&elem);
    Ok(())
}
