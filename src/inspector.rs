use crate::input::{action_toggle_active, Action};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultInspectorConfigPlugin, EguiPlugin))
            .add_systems(
                Update,
                inspector_ui.run_if(action_toggle_active(false, Action::ToggleInspector)),
            );
    }
}

fn inspector_ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new("inspector")
        .default_pos((1200.0, 80.0))
        .show(egui_context.get_mut(), |parent| {
            egui::ScrollArea::vertical().show(parent, |parent| {
                bevy_inspector_egui::bevy_inspector::ui_for_world(world, parent);
            });
        });
}
