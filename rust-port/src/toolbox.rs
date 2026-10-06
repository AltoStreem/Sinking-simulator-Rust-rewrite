//! Toolbox.java graphics colour-picker adapter. Layout uses the Bevy UI panel;
//! other toolbox pages and source editor/workshop behavior remain unconverted.
use bevy::prelude::*;
pub(crate) fn alpha_at(y: f32) -> f32 {
    ((y + 310.0) / 360.0).clamp(0.0, 1.0)
}
pub(crate) fn alpha_marker_y(alpha: f32) -> f32 {
    -310.0 + alpha * 360.0
}
pub(crate) fn rgba_readout_hit(point: Vec2) -> Option<usize> {
    if (point.y + 333.0).abs() > 15.0 {
        return None;
    }
    (0..4).find(|index| (point.x - (-602.0 + *index as f32 * 62.0)).abs() <= 29.0)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alpha_picker_and_marker_share_normalized_range() {
        assert_eq!(alpha_at(-310.0), 0.0);
        assert_eq!(alpha_at(50.0), 1.0);
        assert_eq!(alpha_at(500.0), 1.0);
        for alpha in [0.0, 191.0 / 255.0, 1.0] {
            assert!((alpha_at(alpha_marker_y(alpha)) - alpha).abs() < 0.00001);
        }
    }
    #[test]
    fn rgba_fields_are_individually_selectable() {
        for index in 0..4 {
            assert_eq!(
                rgba_readout_hit(Vec2::new(-602.0 + index as f32 * 62.0, -333.0)),
                Some(index)
            );
        }
        assert_eq!(rgba_readout_hit(Vec2::new(-602.0, 0.0)), None);
        assert_eq!(rgba_readout_hit(Vec2::new(-571.0, -333.0)), None);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::{SeaColorPreview, SeaColorReadout, Simulation};
    #[test]
    fn source_alpha_value_reaches_render_color_and_ui() {
        let mut simulation = Simulation::default();
        assert_eq!(
            simulation.water_color(),
            crate::game_parameters::GameParameterProvider::default().water_color
        );
        simulation.adjust(24, -10.0);
        assert_eq!(simulation.water_color().w, 0.0);
        simulation.adjust(24, 10.0);
        assert_eq!(simulation.water_color().w, 1.0);
        simulation.sea_alpha = 0.25;
        crate::apply_sea_color_point(Vec2::new(-500.0, -100.0), &mut simulation);
        crate::apply_sea_hue_point(Vec2::new(-432.0, 0.0), &mut simulation);
        assert_eq!(simulation.sea_alpha, 0.25);
        let mut app = App::new();
        app.insert_resource(simulation)
            .init_resource::<Assets<Mesh>>();
        let opaque = app
            .world_mut()
            .spawn((SeaColorPreview(false), Sprite::default()))
            .id();
        let alpha = app
            .world_mut()
            .spawn((SeaColorPreview(true), Sprite::default()))
            .id();
        let readout = app
            .world_mut()
            .spawn((SeaColorReadout(3), Text2d::default()))
            .id();
        app.add_systems(Update, crate::sync_settings_ui);
        app.update();
        assert_eq!(
            app.world()
                .get::<Sprite>(opaque)
                .unwrap()
                .color
                .to_srgba()
                .alpha,
            1.0
        );
        assert_eq!(
            app.world()
                .get::<Sprite>(alpha)
                .unwrap()
                .color
                .to_srgba()
                .alpha,
            0.25
        );
        assert_eq!(app.world().get::<Text2d>(readout).unwrap().0, "A: 64");
    }
}

/// Source Toolbox visibility/filter/catalog state. Constructor scheduling, upload and complete rendering remain pending.
pub(crate) struct SourceToolbox {
    tools_visible: bool,
    catalog: crate::toolbox_reload::ReloadState<
        crate::toolbox_reload_filesystem::FilesystemReloadBackend,
    >,
    ship_filter: std::rc::Rc<std::cell::RefCell<Vec<u16>>>,
}
impl Default for SourceToolbox {
    fn default() -> Self {
        Self {
            tools_visible: true,
            catalog: crate::toolbox_reload::ReloadState::default(),
            ship_filter: std::rc::Rc::new(std::cell::RefCell::new(vec![0; 256])),
        }
    }
}
impl SourceToolbox {
    pub fn reload_files(
        &mut self,
        backend: &mut crate::toolbox_reload_filesystem::FilesystemReloadBackend,
    ) -> Result<(), String> {
        self.catalog.reload_files(backend)
    }
    pub fn catalog(
        &self,
    ) -> &crate::toolbox_reload::ReloadState<
        crate::toolbox_reload_filesystem::FilesystemReloadBackend,
    > {
        &self.catalog
    }
    pub fn ship_filter(&self) -> std::rc::Rc<std::cell::RefCell<Vec<u16>>> {
        self.ship_filter.clone()
    }
    pub fn access_set_ship_filter(&mut self, value: std::rc::Rc<std::cell::RefCell<Vec<u16>>>) {
        self.ship_filter = value;
    }
    pub fn tools_visible(&self) -> bool {
        self.tools_visible
    }
    pub fn set_tools_visible(&mut self, value: bool) {
        self.tools_visible = value;
    }
}
