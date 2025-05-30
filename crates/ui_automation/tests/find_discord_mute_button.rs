use uiautomation::UIAutomation;
use ymb_ui_automation::DiscordMuteButton;
use ymb_ui_automation::MuteButtonState;

#[test]
fn it_works() -> eyre::Result<()> {
    let automation = UIAutomation::new()?;
    let elem = DiscordMuteButton::try_find(&automation)?;
    let mute_state = MuteButtonState::try_from(&elem)?;
    println!("Located Discord mute button: {elem:#?} {mute_state:?}");
    Ok(())
}
