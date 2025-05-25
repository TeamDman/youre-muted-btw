use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::EguiContext;
use bevy_egui::EguiMultipassSchedule;
use bevy_egui::egui;
use ymb_ui_automation::AncestryTree;
use ymb_ui_automation::ElementInfo;
use ymb_ui_automation_plugin::LatestTree; // Added import

pub struct TreeWindowPlugin;

impl Plugin for TreeWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_window);
        app.add_systems(UITreeWindowEguiContextPass, ui_tree_window);
    }
}
const DEFAULT_SIZE: (f32, f32) = (320., 160.);

#[derive(Debug, Component, Reflect)]
pub struct UITreeWindow;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UITreeWindowEguiContextPass;

fn spawn_window(mut commands: Commands) {
    commands.spawn((
        Window {
            title: "UI Tree".to_string(),
            resolution: WindowResolution::new(DEFAULT_SIZE.0, DEFAULT_SIZE.1),
            ..default()
        },
        UITreeWindow,
        Name::new("UI Tree Window"),
        EguiMultipassSchedule::new(UITreeWindowEguiContextPass),
    ));
}

fn ui_tree_window(
    mut window: Query<&mut EguiContext, With<UITreeWindow>>,
    latest: Query<&AncestryTree, With<LatestTree>>,
) -> Result {
    let mut ctx = window.single_mut()?;
    let Ok(AncestryTree { tree, start: _ }) = latest.single() else {
        return Ok(());
    };
    egui::CentralPanel::default().show(ctx.get_mut(), |ui| {
        egui::ScrollArea::both().show(ui, |ui| {
            // ui.label("ui tree"); // Removed old label
            render_element_info_node(ui, tree);
        });
    });
    Ok(())
}

fn render_element_info_node(ui: &mut egui::Ui, element_info: &ElementInfo) {
    let id = ui.make_persistent_id(&element_info.drill_id);
    let header_text = format!(
        "{} ({}) - {}",
        element_info.name, element_info.localized_control_type, element_info.drill_id
    );

    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
        .show_header(ui, |ui| {
            ui.label(header_text);
        })
        .body(|ui| {
            if let Some(children) = &element_info.children {
                for child in children {
                    render_element_info_node(ui, child);
                }
            }
        });
}
