use uiautomation::UIAutomation;
use ymb_ui_automation::DiscordMuteButton;
use ymb_ui_automation::DiscordWindowsApp;
use ymb_ui_automation::Drillable;

#[test]
fn it_works() -> eyre::Result<()> {
    let automation = UIAutomation::new()?;
    let matcher = DiscordWindowsApp::get_matcher(&automation);
    let elem = matcher.find_first()?;
    println!("Located Discord window: {elem:#?}");
    let walker = automation.create_tree_walker()?;

    let drill_ids = [
        [0, 1, 1, 0, 1, 0, 0, 0, 1, 3, 1, 1],
        [0, 0, 1, 0, 1, 0, 0, 0, 2, 3, 1, 1],
        [0, 0, 1, 0, 1, 0, 0, 0, 1, 3, 1, 1],
    ];
    let mut found = Vec::new();
    for drill_id in drill_ids {
        print!("{drill_id:?} = ");
        let elem = elem.clone();
        let Ok(mut drilled) = elem.drill(&walker, drill_id) else {
            println!("not found");
            found.push(None);
            continue;
        };
        let (mute_button_element, mute_button_element_info) = drilled.pop_back().unwrap();
        if DiscordMuteButton::try_eq(&mute_button_element_info).is_ok() {
            found.push(Some((mute_button_element, mute_button_element_info)));
            println!("found");
        } else {
            found.push(None);
            println!("not found");
        }
    }
    assert!(found.iter().any(|x| x.is_some()));
    Ok(())
}
