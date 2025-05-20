use uiautomation::UIAutomation;
use ymb_ui_automation::DiscordWindowsApp;

#[test]
fn it_works() -> eyre::Result<()> {
    let automation = UIAutomation::new()?;
    let matcher = DiscordWindowsApp::get_matcher(&automation);
    dbg!(&matcher);
    let elem = matcher.find_first()?;
    println!("Located Discord window: {elem:#?}");
    Ok(())
}