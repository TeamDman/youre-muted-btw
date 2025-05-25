use crate::TargetingCircleSprite;
use crate::TargetingCircleText;
use crate::TargetingCircleWindow;
use crate::TargetingState;
use bevy::color::palettes::css::RED;
use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::text::TextBounds;
use bevy::window::CursorOptions;
use bevy::window::WindowPosition;
use ymb_assets::TARGETING_CIRCLE_WINDOW_LAYERS;
use ymb_host_cursor_position_plugin::HostCursorPosition;

pub(super) struct WindowManagementPlugin;

impl Plugin for WindowManagementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TargetingState>();
        app.add_systems(Startup, setup_window);
        app.add_systems(Update, set_circle_color);
        app.add_systems(Update, set_window_position);
        app.add_systems(Update, set_window_focus);
    }
}

fn setup_window(mut commands: Commands, asset_server: Res<AssetServer>) {
    debug!("Spawning targeting circle window");
    let window = commands
        .spawn((
            Window {
                title: "UI Targetting".to_string(),
                transparent: true,
                decorations: false,
                focused: true,
                // window_level: WindowLevel::AlwaysOnTop,
                cursor_options: CursorOptions {
                    visible: false,
                    ..default()
                },
                ..default()
            },
            Name::new("Targetting Window"),
            TargetingCircleWindow,
        ))
        .id();

    debug!("Spawning targeting circle camera");
    commands.spawn((
        Camera2d,
        Camera {
            target: RenderTarget::Window(bevy::window::WindowRef::Entity(window)),
            ..default()
        },
        TARGETING_CIRCLE_WINDOW_LAYERS,
    ));

    debug!("Spawning targeting circle sprite");
    commands.spawn((
        Sprite::from_image(asset_server.load(ymb_assets::Texture::TargettingCircle)),
        TargetingCircleSprite,
        TARGETING_CIRCLE_WINDOW_LAYERS,
    ));

    let do_text = false;
    if !do_text {
        return;
    }
    debug!("Spawning targeting circle text");
    let font = asset_server.load(ymb_assets::Font::FixederSys2x);
    let slightly_smaller_text_font = TextFont {
        font,
        font_size: 35.0,
        ..default()
    };
    let box_size = Vec2::new(300.0, 200.0);
    let box_position = Vec2::new(-300.0, 250.0);
    commands
        .spawn((
            Sprite::from_color(Color::srgb(0.25, 0.25, 0.55), box_size),
            Transform::from_translation(box_position.extend(0.0)),
            TargetingCircleText,
            TARGETING_CIRCLE_WINDOW_LAYERS,
        ))
        .with_children(|builder| {
            builder.spawn((
                TARGETING_CIRCLE_WINDOW_LAYERS,
                TargetingCircleText,
                Text2d::new("Where is the Discord mute button? Esc to cancel."),
                slightly_smaller_text_font.clone(),
                TextLayout::new(JustifyText::Left, LineBreak::WordBoundary),
                // Wrap text in the rectangle
                TextBounds::from(box_size),
                // Ensure the text is drawn on top of the box
                Transform::from_translation(Vec3::Z),
            ));
        });
}

fn set_window_focus(
    mut windows: Query<&mut Window, With<TargetingCircleWindow>>,
    targeting_state: Res<TargetingState>,
) {
    if !targeting_state.is_changed() {
        return;
    }
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    match *targeting_state {
        TargetingState::FollowingMouse => {
            window.focused = true;
            window.cursor_options.visible = false;
        }
        TargetingState::Paused => {
            window.focused = false;
            window.cursor_options.visible = true;
        }
    }
}

fn set_window_position(
    mut windows: Query<&mut Window, With<TargetingCircleWindow>>,
    host_cursor_position: Res<HostCursorPosition>,
    targeting_state: Res<TargetingState>,
) {
    let TargetingState::FollowingMouse = *targeting_state else {
        return;
    };
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let window_size = window.size();
    let new_position = host_cursor_position.as_vec2() - window_size / 2.0;
    window.position = WindowPosition::At(new_position.as_ivec2());
}

fn set_circle_color(
    mut circle_sprites: Query<&mut Sprite, With<TargetingCircleSprite>>,
    targeting_state: Res<TargetingState>,
) {
    if !targeting_state.is_changed() {
        return;
    }
    if let Ok(mut sprite) = circle_sprites.single_mut() {
        match *targeting_state {
            TargetingState::FollowingMouse => {
                sprite.color = WHITE.into();
            }
            TargetingState::Paused => {
                sprite.color = RED.into();
            }
        }
    }
}
