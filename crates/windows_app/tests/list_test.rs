use ymb_windows_app::list_apps;
use ymb_windy::WindyResult;

#[test]
fn list_test() -> WindyResult<()> {
    let apps = list_apps()?;
    for app in apps {
        println!("{app:#?}");
    }
    Ok(())
}
