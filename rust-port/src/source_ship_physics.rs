//! ShipPhysics.java native constructor, pass graph and uniform cadence.
//! The Bevy compute adapter remains in ship_physics.rs.
#![allow(dead_code)]
use crate::{
    force_data::SourceForceDataHolder,
    gl_data_holder::SourceGlDataHolder,
    mask_struts_data::SourceMaskStrutsDataHolder,
    mass_strength_data::SourceMassStrengthDataHolder,
    materials::SourceMaterials,
    passes::{
        pass::Pass,
        pass_builder::{NamedTexture, PassBuilder, PassFactory},
        provider_pass::{ProviderPass, TextureBinding},
        stateful_pass::StatefulPass,
    },
    pos_vel_data::SourcePosVelDataHolder,
    render_buffer::{RenderBuffer, RenderBufferBackend},
    resource::{ResourceHandle, ResourceRuntime},
    ship_data::SourceShipData,
    ship_physics_shaders as shaders,
    texture::TextureBackend,
    texture_2d::SourceTexture2D,
    water_data::SourceWaterDataHolder,
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

struct NativeBinding(Arc<SourceTexture2D>);
impl TextureBinding for NativeBinding {
    fn bind(&mut self, unit: usize) {
        self.0.texture.bind_unit(unit as i32);
    }
    fn unbind(&mut self, unit: usize) {
        self.0.texture.unbind_unit(unit as i32);
    }
    fn native_texture(&self) -> Option<Arc<SourceTexture2D>> {
        Some(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        gl_state::GlState,
        passes::{
            shader_pass_backend::ShaderPassBackend,
            target_pass::{StencilTarget, TargetBinding},
        },
    };
    type Log = Arc<Mutex<Vec<String>>>;
    struct Shader {
        id: usize,
        log: Log,
        names: Vec<String>,
    }
    impl ShaderPassBackend for Shader {
        fn create_shader(&mut self, source: &str, dst: &[String]) {
            let sources = [
                shaders::FILTER_DYNAMIC,
                shaders::FORCES,
                shaders::INTEGRATE,
                shaders::FILTER_PERMEABLE,
                shaders::FILL_WATER,
                shaders::FLOW_WATER,
                shaders::TRANSPORT_WATER,
                shaders::UPDATE_MASS,
                shaders::FILTER_OCCUPIED,
                shaders::REPAIR_MASK,
                shaders::MOVE,
            ];
            assert_eq!(source, sources[self.id]);
            self.log
                .lock()
                .unwrap()
                .push(format!("shader:{}:{dst:?}", self.id));
        }
        fn uniform_location(&mut self, name: &str) -> i32 {
            self.names.push(name.into());
            (self.names.len() - 1) as i32
        }
        fn start_shader(&mut self) {}
        fn stop_shader(&mut self) {}
        fn write_float_uniform(&mut self, location: i32, values: &[f32]) {
            self.log.lock().unwrap().push(format!(
                "float:{}:{}:{values:?}",
                self.id, self.names[location as usize]
            ));
        }
        fn write_int_uniform(&mut self, location: i32, values: &[i32]) {
            self.log.lock().unwrap().push(format!(
                "int:{}:{}:{values:?}",
                self.id, self.names[location as usize]
            ));
        }
        fn write_matrix_uniform(&mut self, _: i32, _: bool, _: &[f32; 16]) {}
        fn stencil_test(&mut self, _: bool) {}
        fn color_mask(&mut self, _: [bool; 4]) {}
        fn current_state(&mut self) -> Box<dyn std::any::Any> {
            Box::new(())
        }
        fn apply_state(&mut self) {}
        fn restore_state(&mut self, _: Box<dyn std::any::Any>) {}
        fn draw_fullscreen(&mut self) {
            self.log.lock().unwrap().push(format!("draw:{}", self.id));
        }
    }
    struct Target;
    impl TargetBinding for Target {
        fn viewport(&mut self) -> [i32; 4] {
            [0, 0, 800, 400]
        }
        fn bind(&mut self) {}
        fn draw_buffers(&mut self, _: &[u32]) {}
        fn set_viewport(&mut self, _: [i32; 4]) {}
        fn unbind(&mut self) {}
    }
    struct Factory {
        count: usize,
        log: Log,
        states: Vec<Rc<dyn GlState>>,
        renderbuffers: Arc<Mutex<dyn RenderBufferBackend>>,
        context: ResourceHandle,
        runtime: ResourceRuntime,
        targets: Vec<(Vec<String>, Vec<i32>, Rc<dyn StencilTarget>)>,
    }
    impl PassFactory for Factory {
        fn shader_backend(&mut self, state: Rc<dyn GlState>) -> Box<dyn ShaderPassBackend> {
            let id = self.count;
            self.count += 1;
            self.states.push(state);
            Box::new(Shader {
                id,
                log: self.log.clone(),
                names: vec![],
            })
        }
        fn stencil(&mut self, size: [i32; 2], format: i32) -> Rc<dyn StencilTarget> {
            self.log
                .lock()
                .unwrap()
                .push(format!("new-stencil:{size:?}:{format}"));
            Rc::new(RenderBuffer::new(
                size[0],
                size[1],
                format,
                self.renderbuffers.clone(),
                self.context.clone(),
                &self.runtime,
            ))
        }
        fn target(
            &mut self,
            dst: &[NamedTexture],
            stencil: Rc<dyn StencilTarget>,
        ) -> Box<dyn TargetBinding> {
            let names = dst.iter().map(|texture| texture.name.clone()).collect();
            let ids = dst
                .iter()
                .map(|texture| {
                    texture
                        .texture
                        .borrow()
                        .native_texture()
                        .unwrap()
                        .texture
                        .id()
                })
                .collect();
            self.targets.push((names, ids, stencil));
            Box::new(Target)
        }
    }

    #[test]
    fn original_constructor_allocates_and_connects_all_holders_passes_and_stencils_in_order() {
        let runtime = ResourceRuntime::default();
        let context = runtime.allocate(&[], || {});
        let log = Arc::new(Mutex::new(Vec::new()));
        let backend = crate::render_fbo::source_tests::backend(log.clone());
        let mut factory = Factory {
            count: 0,
            log: log.clone(),
            states: vec![],
            renderbuffers: backend.clone(),
            context: context.clone(),
            runtime: runtime.clone(),
            targets: vec![],
        };
        let mut material = crate::materials::Material::default();
        material.mass = 100.;
        let dat = Rc::new(SourceShipData::new(
            Arc::new(Mutex::new(crate::image_data::ImageData::new(
                vec![],
                0,
                0,
                6408,
            ))),
            Arc::new(SourceMaterials::default()),
            Rc::new(RefCell::new(vec![Some(Arc::new(material))])),
            1,
            1,
        ));
        let clock = Rc::new(RefCell::new(0i64));
        let ticks = clock.clone();
        let mut physics = SourceShipPhysics::new(
            dat.clone(),
            backend.clone(),
            backend,
            &mut factory,
            context.clone(),
            &runtime,
            Box::new(move || {
                *ticks.borrow_mut() += 1;
                *ticks.borrow()
            }),
        )
        .unwrap();
        assert!(Rc::ptr_eq(&physics.dat, &dat));
        assert_eq!(physics.last, 1);
        assert_eq!(factory.count, 11);
        assert_eq!(factory.targets.len(), 8);
        let ids: Vec<_> = factory
            .targets
            .iter()
            .map(|target| target.1.clone())
            .collect();
        assert_eq!(
            ids,
            vec![
                vec![2, 8],
                vec![1],
                vec![3],
                vec![4, 5, 6, 7],
                vec![3],
                vec![9],
                vec![8],
                vec![1]
            ]
        );
        for index in [1, 2, 3, 4, 5, 7] {
            assert!(Rc::ptr_eq(&factory.targets[0].2, &factory.targets[index].2));
        }
        assert!(!Rc::ptr_eq(&factory.targets[0].2, &factory.targets[6].2));
        assert!(Rc::ptr_eq(&factory.states[0], &factory.states[8]));
        assert!(Rc::ptr_eq(&factory.states[1], &factory.states[2]));
        assert!(Rc::ptr_eq(&factory.states[9], &factory.states[10]));
        let events = log.lock().unwrap().clone();
        let draws: Vec<_> = events
            .iter()
            .filter(|event| event.starts_with("draw:"))
            .cloned()
            .collect();
        assert_eq!(draws, ["draw:0", "draw:8"]);
        let uploads: Vec<_> = events
            .iter()
            .filter(|event| event.starts_with("image2:"))
            .collect();
        assert_eq!(uploads.len(), 9);
        assert!(
            events
                .iter()
                .any(|event| event == "new-stencil:[1, 1]:35056")
        );
        assert!(
            uploads
                .last()
                .unwrap()
                .contains(":34836:[1, 1]:6408:5126:false")
        );
        let saved = crate::game_parameter_provider_kt::get_game_parameter_provider();
        let provider = Rc::new(RefCell::new(
            crate::game_parameters::SourceGameParameterProvider::default(),
        ));
        provider.borrow_mut().set_physics_steps(7);
        provider.borrow_mut().set_water_steps(3);
        crate::game_parameter_provider_kt::set_game_parameter_provider(provider.clone());
        log.lock().unwrap().clear();
        physics.update().unwrap();
        assert_eq!(physics.last, 2);
        let events = log.lock().unwrap().clone();
        assert_eq!(
            events
                .iter()
                .filter(|event| event.as_str() == "draw:1")
                .count(),
            7
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| event.as_str() == "draw:4")
                .count(),
            3
        );
        assert!(events.contains(&"int:1:iter:[7]".into()));
        assert!(events.contains(&format!("float:1:deltaT:[{:?}]", 0.016666668f32 / 7.)));
        provider.borrow_mut().set_water_steps(8);
        log.lock().unwrap().clear();
        assert!(physics.update().unwrap_err().contains("remainder"));
        assert_eq!(
            log.lock()
                .unwrap()
                .iter()
                .filter(|event| event.as_str() == "draw:1")
                .count(),
            1
        );
        provider.borrow_mut().set_water_steps(0);
        log.lock().unwrap().clear();
        assert!(physics.update().unwrap_err().contains("division"));
        assert!(log.lock().unwrap().contains(&"float:4:deltaT:[inf]".into()));
        assert!(
            !log.lock()
                .unwrap()
                .iter()
                .any(|event| event.starts_with("draw:"))
        );
        log.lock().unwrap().clear();
        physics.move_by(bevy::prelude::Vec2::new(3., -4.));
        physics.set_time(-0.0);
        let events = log.lock().unwrap().clone();
        assert!(events.contains(&"float:10:u_mouse:[3.0, -4.0]".into()));
        assert!(events.contains(&"draw:10".into()));
        assert!(events.contains(&"float:1:time:[-0.0]".into()));
        assert!(events.contains(&"float:4:time:[-0.0]".into()));
        physics.close();
        assert!(physics.freed());
        runtime.run_main();
        assert!(
            !log.lock()
                .unwrap()
                .iter()
                .any(|event| event.starts_with("delete_tex:"))
        );
        context.close();
        runtime.run_main();
        assert_eq!(
            log.lock()
                .unwrap()
                .iter()
                .filter(|event| event.starts_with("delete_tex:"))
                .count(),
            9
        );
        crate::game_parameter_provider_kt::set_game_parameter_provider(saved);
    }
}
fn named(name: &str, holder: &impl SourceGlDataHolder) -> NamedTexture {
    let texture = holder.source_texture();
    NamedTexture {
        name: name.into(),
        size: [texture.width, texture.height],
        texture: Rc::new(RefCell::new(NativeBinding(texture))),
    }
}

pub(crate) struct SourceShipPhysics {
    resource: ResourceHandle,
    pub(crate) dat: Rc<SourceShipData<SourceMaterials>>,
    pub(crate) pos_vel: SourcePosVelDataHolder,
    pub(crate) forces: SourceForceDataHolder,
    pub(crate) water: SourceWaterDataHolder,
    pub(crate) water_outflow_1: SourceWaterDataHolder,
    pub(crate) water_outflow_2: SourceWaterDataHolder,
    pub(crate) water_scalar_vel_1: SourceWaterDataHolder,
    pub(crate) water_scalar_vel_2: SourceWaterDataHolder,
    pub(crate) mask_struts: SourceMaskStrutsDataHolder,
    pub(crate) mass_strength: SourceMassStrengthDataHolder,
    pub(crate) physics_filter: Arc<RenderBuffer>,
    pub(crate) physics_pass: ProviderPass,
    pub(crate) final_pass: ProviderPass,
    pub(crate) pos_change_pass: ProviderPass,
    pub(crate) last: i64,
    clock: Box<dyn FnMut() -> i64>,
}
impl SourceShipPhysics {
    pub(crate) fn new(
        dat: Rc<SourceShipData<SourceMaterials>>,
        textures: Arc<Mutex<dyn TextureBackend>>,
        renderbuffers: Arc<Mutex<dyn RenderBufferBackend>>,
        factory: &mut dyn PassFactory,
        context: ResourceHandle,
        runtime: &ResourceRuntime,
        mut clock: Box<dyn FnMut() -> i64>,
    ) -> Result<Self, String> {
        // Resource(empty dependencies) precedes all data-holder allocations.
        let resource = runtime.allocate(&[], || {});
        let pos_vel =
            SourcePosVelDataHolder::new(&dat, textures.clone(), context.clone(), runtime)?;
        let forces = SourceForceDataHolder::new(&dat, textures.clone(), context.clone(), runtime)?;
        let water = SourceWaterDataHolder::new(&dat, textures.clone(), context.clone(), runtime)?;
        let water_outflow_1 =
            SourceWaterDataHolder::new(&dat, textures.clone(), context.clone(), runtime)?;
        let water_outflow_2 =
            SourceWaterDataHolder::new(&dat, textures.clone(), context.clone(), runtime)?;
        let water_scalar_vel_1 =
            SourceWaterDataHolder::new(&dat, textures.clone(), context.clone(), runtime)?;
        let water_scalar_vel_2 =
            SourceWaterDataHolder::new(&dat, textures.clone(), context.clone(), runtime)?;
        let mask_struts =
            SourceMaskStrutsDataHolder::new(&dat, textures.clone(), context.clone(), runtime)?;
        let mass_strength =
            SourceMassStrengthDataHolder::new(&dat, textures, context.clone(), runtime)?;
        let physics_filter = Arc::new(RenderBuffer::new(
            dat.width,
            dat.height,
            35056,
            renderbuffers,
            context,
            runtime,
        ));
        let stencil = Rc::new(crate::render_buffer::SharedStencilTarget(
            physics_filter.clone(),
        ));
        let states = crate::ship_physics_stencil::states();
        let physics_pass = PassBuilder::with(
            vec![
                named("in_pos_vel", &pos_vel),
                named("in_force", &forces),
                named("in_mask_struts", &mask_struts),
                named("in_mass_strength", &mass_strength),
            ],
            factory,
            |step| {
                step.set_stencil_target(stencil.clone());
                step.target_pass(
                    vec![named("out_force", &forces), named("out_mask", &mask_struts)],
                    |target| {
                        target.setup_stencil_pass(states.set7.clone(), shaders::FILTER_DYNAMIC);
                        target.pass(states.execIf7.clone(), shaders::FORCES);
                    },
                );
                step.target_pass(vec![named("out_pos_vel", &pos_vel)], |target| {
                    target.pass(states.execIf7.clone(), shaders::INTEGRATE);
                });
            },
        );
        let final_pass = PassBuilder::with(
            vec![
                named("in_pos_vel", &pos_vel),
                named("in_water", &water),
                named("in_mask_struts", &mask_struts),
                named("in_mass_strength", &mass_strength),
                named("in_water_out_1", &water_outflow_1),
                named("in_water_out_2", &water_outflow_2),
                named("in_water_vel_1", &water_scalar_vel_1),
                named("in_water_vel_2", &water_scalar_vel_2),
            ],
            factory,
            |step| {
                step.set_stencil_target(stencil.clone());
                step.target_pass(vec![named("out_water", &water)], |target| {
                    target.stencil_pass(states.set1If7.clone(), shaders::FILTER_PERMEABLE);
                    target.pass(states.execIf7.clone(), shaders::FILL_WATER);
                });
                step.target_pass(
                    vec![
                        named("out_water_out_1", &water_outflow_1),
                        named("out_water_out_2", &water_outflow_2),
                        named("out_water_vel_1", &water_scalar_vel_1),
                        named("out_water_vel_2", &water_scalar_vel_2),
                    ],
                    |target| {
                        target.pass(states.execIf1.clone(), shaders::FLOW_WATER);
                    },
                );
                step.target_pass(vec![named("out_water", &water)], |target| {
                    target.pass(states.execIf1.clone(), shaders::TRANSPORT_WATER);
                });
                step.target_pass(vec![named("out_mass_strength", &mass_strength)], |target| {
                    target.pass(states.execIf7.clone(), shaders::UPDATE_MASS);
                });
                step.reset_stencil();
                step.target_pass(vec![named("out_mask", &mask_struts)], |target| {
                    target.setup_stencil_pass(states.set7.clone(), shaders::FILTER_OCCUPIED);
                    target.pass(states.exec.clone(), shaders::REPAIR_MASK);
                });
            },
        );
        let pos_change_pass =
            PassBuilder::with(vec![named("in_pos_vel", &pos_vel)], factory, |step| {
                step.set_stencil_target(stencil.clone());
                step.target_pass(vec![named("out_pos_vel", &pos_vel)], |target| {
                    target.pass(states.exec.clone(), shaders::MOVE);
                });
            });
        let last = clock();
        Ok(Self {
            resource,
            dat,
            pos_vel,
            forces,
            water,
            water_outflow_1,
            water_outflow_2,
            water_scalar_vel_1,
            water_scalar_vel_2,
            mask_struts,
            mass_strength,
            physics_filter,
            physics_pass,
            final_pass,
            pos_change_pass,
            last,
            clock,
        })
    }
    pub(crate) fn close(&self) {
        self.resource.close();
    }
    pub(crate) fn freed(&self) -> bool {
        self.resource.freed()
    }
    pub(crate) fn move_by(&mut self, delta: bevy::prelude::Vec2) {
        self.pos_change_pass
            .set_float_arg("u_mouse", &delta.to_array());
        self.pos_change_pass.render();
    }
    pub(crate) fn set_time(&mut self, time: f32) {
        self.physics_pass.set_float_arg("time", &[time]);
        self.final_pass.set_float_arg("time", &[time]);
    }
    pub(crate) fn update(&mut self) -> Result<(), String> {
        use crate::game_parameter_provider_kt::get_game_parameter_provider as provider;
        // Each source getter is evaluated independently: setters may replace
        // the global provider while a pass receives an earlier uniform.
        let x = provider().borrow().waves().borrow().x;
        let y = provider().borrow().waves().borrow().y;
        self.physics_pass.set_float_arg("waveSize", &[x, y]);
        let x = provider().borrow().waves().borrow().x;
        let y = provider().borrow().waves().borrow().y;
        self.final_pass.set_float_arg("waveSize", &[x, y]);
        {
            let values = [-provider().borrow().sea_floor()];
            self.physics_pass.set_float_arg("floorHeight", &values);
        }
        {
            let values = [provider().borrow().buoyancy()];
            self.physics_pass.set_float_arg("u_buoyancy", &values);
        }
        {
            let values = [provider().borrow().buoyancy()];
            self.final_pass.set_float_arg("u_buoyancy", &values);
        }
        {
            let values = [provider().borrow().drag()];
            self.physics_pass.set_float_arg("u_drag", &values);
        }
        {
            let values = [provider().borrow().flow()];
            self.final_pass.set_float_arg("u_flow", &values);
        }
        {
            let values = [provider().borrow().inflow()];
            self.final_pass.set_float_arg("u_inflow", &values);
        }
        {
            let values = [provider().borrow().funk()];
            self.final_pass.set_float_arg("u_funk", &values);
        }
        {
            let values = [0.0, -provider().borrow().gravity()];
            self.physics_pass.set_float_arg("gravity", &values);
        }
        {
            let values = [0.0, -provider().borrow().gravity()];
            self.final_pass.set_float_arg("gravity", &values);
        }
        {
            let values = [provider().borrow().strength()];
            self.physics_pass.set_float_arg("u_strength", &values);
        }
        {
            let values = [provider().borrow().rigidity()];
            self.physics_pass.set_float_arg("u_rigidity", &values);
        }
        {
            let values = [provider().borrow().dampening()];
            self.physics_pass.set_float_arg("u_dampening", &values);
        }
        {
            let values = [provider().borrow().water_weight()];
            self.final_pass.set_float_arg("u_waterweight", &values);
        }
        {
            let values = [1.0 - provider().borrow().thickness()];
            self.final_pass.set_float_arg("u_thickness", &values);
        }
        self.last = (self.clock)();
        let delta = 0.016666668f32;
        let iterations = provider().borrow().physics_steps();
        let water_iterations = provider().borrow().water_steps();
        self.physics_pass
            .set_float_arg("deltaT", &[delta / iterations as f32]);
        self.physics_pass
            .set_float_arg("fps", &[iterations as f32 / delta]);
        self.physics_pass.set_int_arg("iter", &[iterations]);
        self.final_pass
            .set_float_arg("deltaT", &[delta / water_iterations as f32]);
        if water_iterations == 0 {
            return Err("source integer division by zero".into());
        }
        let interval = iterations.wrapping_div(water_iterations);
        for iteration in 0..iterations {
            self.physics_pass.render();
            if interval == 0 {
                return Err("source integer remainder by zero".into());
            }
            if iteration.wrapping_rem(interval) == interval.wrapping_sub(1) {
                self.final_pass.render();
            }
        }
        Ok(())
    }
}
