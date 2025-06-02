use bevy::prelude::*;
use bevy_winit::WinitWindows;

pub struct WindowIconPlugin;

impl Plugin for WindowIconPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<WindowIcon>();
        app.add_event::<AddWindowIconWithRetry>();
        app.add_observer(on_window_icon_added);
        app.add_systems(Update, set_window_icon);
    }
}

#[derive(Debug, Component, Reflect)]
pub struct WindowIcon {
    pub image: Handle<Image>,
}

#[derive(Debug, Event, Clone, Reflect)]
pub struct AddWindowIconWithRetry {
    pub image: Handle<Image>,
    pub window: Entity,
}

impl WindowIcon {
    pub fn new(image: Handle<Image>) -> Self {
        Self { image }
    }
}

fn on_window_icon_added(
    trigger: Trigger<OnAdd, WindowIcon>,
    icon: Query<&WindowIcon>,
    mut events: EventWriter<AddWindowIconWithRetry>,
) -> Result {
    debug!("Triggered on_add_window_icon for {:?}", trigger.target());
    let icon = icon.get(trigger.target())?;
    events.write(AddWindowIconWithRetry {
        image: icon.image.clone(),
        window: trigger.target(),
    });
    Ok(())
}

fn set_window_icon(
    mut events: ParamSet<(
        EventReader<AddWindowIconWithRetry>,
        EventWriter<AddWindowIconWithRetry>,
    )>,
    windows: NonSend<WinitWindows>,
    assets: Res<Assets<Image>>,
) -> Result {
    let mut outgoing = Vec::new();
    for event in events.p0().read() {
        debug!("Triggered on_retry_add_window_icon for {:?}", event.window);
        debug!("Windows: {:#?}", windows);
        let target_window = windows.get_window(event.window);
        let Some(window) = target_window else {
            warn!(
                "Window {:?} does not exist, retrying later...",
                event.window
            );
            outgoing.push(event.clone());
            continue;
        };
        info!("Setting window icon for {:?}", event.window);
        let Some(image) = assets.get(&event.image) else {
            error!(
                "Image handle {:?} not found in assets, the window will not have our custom icon",
                event.image
            );
            continue;
        };
        let Some(image_data) = image.data.clone() else {
            error!(
                "Image handle {:?} has no data, the window will not have our custom icon",
                event.image
            );
            continue;
        };
        info!("Setting window icon to image {}", image.size());
        let icon = winit::window::Icon::from_rgba(
            image_data,
            image.texture_descriptor.size.width,
            image.texture_descriptor.size.height,
        );
        match icon {
            Ok(icon) => {
                window.set_window_icon(Some(icon));
            }
            Err(e) => {
                error!("Failed to construct window icon: {:?}", e);
            }
        }
    }
    for event in outgoing {
        events.p1().write(event);
    }
    Ok(())
}
