use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::text::Text2d;
use bevy::text::TextBounds;
use bevy::text::TextLayout;
use bevy::window::CursorOptions;
use bevy::window::WindowLevel;
use bevy_egui::EguiContext;
use bevy_egui::EguiMultipassSchedule;
use bevy_egui::egui;
use ymb_assets;

pub struct TargettingWindowPlugin;

impl Plugin for TargettingWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_targetting_elements);
        app.add_systems(Startup, spawn_targetting_window);
        app.add_systems(
            TargettingWindowEguiContextPass,
            ui_targetting_window.run_if(|| false),
        );
    }
}

#[derive(Debug, Component, Reflect)]
pub struct TargettingWindow;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TargettingWindowEguiContextPass;

fn spawn_targetting_window(mut commands: Commands) {
    let window = commands
        .spawn((
            Window {
                title: "UI Targetting".to_string(),
                transparent: true,
                decorations: false,
                focused: true,
                window_level: WindowLevel::AlwaysOnTop,
                cursor_options: CursorOptions {
                    visible: false,
                    ..default()
                },
                ..default()
            },
            Name::new("Targetting Window"),
            TargettingWindow,
            EguiMultipassSchedule::new(TargettingWindowEguiContextPass),
        ))
        .id();
    commands.spawn((
        Camera2d,
        Camera {
            target: RenderTarget::Window(bevy::window::WindowRef::Entity(window)),
            ..default()
        },
    ));
}

fn setup_targetting_elements(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Sprite::from_image(
        asset_server.load(ymb_assets::Texture::TargettingCircle),
    ));

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
        ))
        .with_children(|builder| {
            builder.spawn((
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

fn ui_targetting_window(mut window: Query<&mut EguiContext, With<TargettingWindow>>) -> Result {
    let mut ctx = window.single_mut()?;
    egui::Window::new("bruh").show(ctx.get_mut(), |ui| {
        ui.label("bruh");
    });
    Ok(())
}
