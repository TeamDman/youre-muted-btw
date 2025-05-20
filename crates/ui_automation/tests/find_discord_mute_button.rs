use uiautomation::UIAutomation;
use ymb_ui_automation::DiscordMuteButton;

#[test]
fn it_works() -> Result<(), uiautomation::Error> {
    let automation = UIAutomation::new()?;
    let matcher = DiscordMuteButton::get_matcher(&automation);
    dbg!(&matcher);
    let elem = matcher.find_first()?;
    println!("Located Discord mute button: {elem:#?}");
    Ok(())
}
