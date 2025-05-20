use bevy::math::IVec2;
use ymb_ui_automation::gather_tree_from_position;

#[test]
fn from_pos() -> eyre::Result<()> {
    let pos = IVec2::new(100, 100);
    let gathered = gather_tree_from_position(pos)?;
    dbg!(&gathered);
    Ok(())
}
