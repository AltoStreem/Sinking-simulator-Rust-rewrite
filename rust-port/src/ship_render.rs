//! Ship.java render/setTime sequence using the converted stencil state objects.
#![allow(dead_code)]
use crate::gl_state::{
    GlState, StateBackend, stencil_config::StencilConfig, stencil_func::StencilFunc,
    stencil_op::StencilOp, stencil_write_mask::StencilWriteMask,
};
use std::rc::Rc;

pub(crate) trait ShipRenderTexture {
    fn bind(&self, unit: i32);
    fn unbind(&self, unit: i32);
}
impl ShipRenderTexture for crate::texture::Texture {
    fn bind(&self, unit: i32) {
        self.bind_unit(unit);
    }
    fn unbind(&self, unit: i32) {
        self.unbind_unit(unit);
    }
}
pub(crate) trait ShipRenderBackend: StateBackend {
    fn display_arg(&mut self, name: &str, values: &[f32]);
    fn struts_arg(&mut self, name: &str, values: &[f32]);
    fn clear(&mut self, mask: i32);
    fn enable(&mut self, capability: i32);
    fn disable(&mut self, capability: i32);
    fn blend_equation(&mut self, equation: i32);
    fn blend_func(&mut self, source: i32, destination: i32);
    /// Live getter at each original call site: display, pos, mask, water,
    /// interior lights, exterior lights. Missing lights use the shared black map.
    fn texture(&mut self, slot: i32) -> Rc<dyn ShipRenderTexture>;
    fn render_surface(&mut self);
    fn render_struts(&mut self);
    fn display_time(&mut self, time: f32);
    fn struts_time(&mut self, time: f32);
    fn physics_time(&mut self, time: f32);
}
pub(crate) struct SourceShipRender {
    set1: StencilConfig,
    ifnot1: StencilConfig,
}
impl SourceShipRender {
    pub fn new() -> Self {
        Self {
            set1: StencilConfig::new(
                Some(StencilWriteMask::new(255)),
                Some(StencilFunc::new(519, 255, 255)),
                Some(StencilOp::new(0, 7681, 7681)),
            ),
            ifnot1: StencilConfig::new(
                Some(StencilWriteMask::new(255)),
                Some(StencilFunc::new(514, 0, 255)),
                Some(StencilOp::new(7680, 7680, 7680)),
            ),
        }
    }
    pub fn render(&self, backend: &mut impl ShipRenderBackend) {
        use crate::game_parameter_provider_kt::get_game_parameter_provider;
        let day = get_game_parameter_provider().borrow().day();
        backend.display_arg("day", &[day]);
        let color = get_game_parameter_provider()
            .borrow()
            .water_color()
            .borrow()
            .to_array();
        backend.display_arg("waterCol", &color);
        let day = get_game_parameter_provider().borrow().day();
        backend.struts_arg("day", &[day]);
        backend.clear(1024);
        backend.enable(3042);
        backend.blend_equation(32774);
        backend.blend_func(770, 771);
        backend.enable(2960);
        for unit in 0..6 {
            backend.texture(unit).bind(unit);
        }
        let previous = self.set1.current_state(backend);
        self.set1.apply(backend);
        backend.render_surface();
        previous.apply(backend);
        let previous = self.ifnot1.current_state(backend);
        self.ifnot1.apply(backend);
        backend.render_struts();
        previous.apply(backend);
        // The source repeats each getter here, rather than capturing textures.
        for unit in (0..6).rev() {
            backend.texture(unit).unbind(unit);
        }
        backend.disable(3042);
        backend.disable(2960);
    }
    pub fn set_time(&self, backend: &mut impl ShipRenderBackend, time: f32) {
        backend.display_time(time);
        backend.struts_time(time);
        backend.physics_time(time);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    struct Texture {
        log: Rc<RefCell<Vec<String>>>,
        generation: i32,
    }
    impl ShipRenderTexture for Texture {
        fn bind(&self, unit: i32) {
            self.log
                .borrow_mut()
                .push(format!("bind:{unit}:{}", self.generation));
        }
        fn unbind(&self, unit: i32) {
            self.log
                .borrow_mut()
                .push(format!("unbind:{unit}:{}", self.generation));
        }
    }
    struct Backend {
        log: Rc<RefCell<Vec<String>>>,
        generation: i32,
        fail: bool,
    }
    impl Backend {
        fn record(&self, event: String) {
            self.log.borrow_mut().push(event);
        }
    }
    impl StateBackend for Backend {
        fn get_integer(&mut self, parameter: i32) -> i32 {
            self.record(format!("get:{parameter}"));
            parameter
        }
        fn stencil_mask(&mut self, mask: i32) {
            self.record(format!("mask:{mask}"));
        }
        fn stencil_func(&mut self, func: i32, reference: i32, mask: i32) {
            self.record(format!("func:{func}:{reference}:{mask}"));
        }
        fn stencil_op(&mut self, fail: i32, depth: i32, success: i32) {
            self.record(format!("op:{fail}:{depth}:{success}"));
        }
    }
    impl ShipRenderBackend for Backend {
        fn display_arg(&mut self, name: &str, values: &[f32]) {
            self.record(format!("display:{name}:{values:?}"));
        }
        fn struts_arg(&mut self, name: &str, values: &[f32]) {
            self.record(format!("struts:{name}:{values:?}"));
        }
        fn clear(&mut self, mask: i32) {
            self.record(format!("clear:{mask}"));
        }
        fn enable(&mut self, value: i32) {
            self.record(format!("enable:{value}"));
        }
        fn disable(&mut self, value: i32) {
            self.record(format!("disable:{value}"));
        }
        fn blend_equation(&mut self, value: i32) {
            self.record(format!("equation:{value}"));
        }
        fn blend_func(&mut self, source: i32, dest: i32) {
            self.record(format!("blend:{source}:{dest}"));
        }
        fn texture(&mut self, slot: i32) -> Rc<dyn ShipRenderTexture> {
            self.record(format!("texture:{slot}"));
            Rc::new(Texture {
                log: self.log.clone(),
                generation: self.generation,
            })
        }
        fn render_surface(&mut self) {
            self.record("surface".into());
            assert!(!self.fail, "draw failure");
            self.generation += 1;
        }
        fn render_struts(&mut self) {
            self.record("struts".into());
        }
        fn display_time(&mut self, time: f32) {
            self.record(format!("display-time:{time}"));
        }
        fn struts_time(&mut self, time: f32) {
            self.record(format!("struts-time:{time}"));
        }
        fn physics_time(&mut self, time: f32) {
            self.record(format!("physics-time:{time}"));
        }
    }
    #[test]
    fn source_ship_render_stencil_scopes_texture_getters_and_time_order() {
        let log = Rc::new(RefCell::new(vec![]));
        let mut backend = Backend {
            log: log.clone(),
            generation: 0,
            fail: false,
        };
        let ship = SourceShipRender::new();
        ship.render(&mut backend);
        let provider = crate::game_parameter_provider_kt::get_game_parameter_provider();
        let day = provider.borrow().day();
        let color = provider.borrow().water_color().borrow().to_array();
        let mut expected = vec![
            format!("display:day:{:?}", [day]),
            format!("display:waterCol:{color:?}"),
            format!("struts:day:{:?}", [day]),
        ];
        expected.extend(
            [
                "clear:1024",
                "enable:3042",
                "equation:32774",
                "blend:770:771",
                "enable:2960",
            ]
            .into_iter()
            .map(str::to_owned),
        );
        for unit in 0..6 {
            expected.extend([format!("texture:{unit}"), format!("bind:{unit}:0")]);
        }
        let queries = [2968, 2962, 2967, 2963, 2964, 2965, 2966];
        let restore = ["mask:2968", "func:2962:2967:2963", "op:2964:2965:2966"];
        for (config, draw) in [
            (
                ["mask:255", "func:519:255:255", "op:0:7681:7681"],
                "surface",
            ),
            (
                ["mask:255", "func:514:0:255", "op:7680:7680:7680"],
                "struts",
            ),
        ] {
            expected.extend(queries.map(|p| format!("get:{p}")));
            expected.extend(config.map(str::to_owned));
            expected.push(draw.into());
            expected.extend(restore.map(str::to_owned));
        }
        for unit in (0..6).rev() {
            expected.extend([format!("texture:{unit}"), format!("unbind:{unit}:1")]);
        }
        expected.extend(["disable:3042".into(), "disable:2960".into()]);
        assert_eq!(*log.borrow(), expected);
        log.borrow_mut().clear();
        ship.render(&mut backend);
        assert!(!log.borrow().iter().any(|event| event.starts_with("get:")));
        log.borrow_mut().clear();
        ship.set_time(&mut backend, 1.25);
        assert_eq!(
            *log.borrow(),
            ["display-time:1.25", "struts-time:1.25", "physics-time:1.25"]
        );
    }
    #[test]
    fn source_draw_failure_keeps_bound_textures_and_applied_stencil() {
        let log = Rc::new(RefCell::new(vec![]));
        let mut backend = Backend {
            log: log.clone(),
            generation: 0,
            fail: true,
        };
        let ship = SourceShipRender::new();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ship.render(&mut backend)))
                .is_err()
        );
        assert_eq!(log.borrow().last().unwrap(), "surface");
        assert!(
            !log.borrow()
                .iter()
                .any(|event| event.starts_with("unbind:") || event.starts_with("disable:"))
        );
    }
}
