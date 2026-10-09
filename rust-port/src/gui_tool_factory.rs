//! Concrete GUI tool factory using the separately translated original tool classes.
//! The source graphics/ship environment and native backends still have to be supplied by the runtime adapter.
use crate::{
    camera_control::SourceCameraControl,
    file_reader::FileReader,
    game_parameters::SourceGameParameterProvider,
    gui::{GuiTool, GuiToolFactory},
    resource::{ResourceHandle, ResourceRuntime},
    texture::TextureBackend,
    tools::{
        break_tool::SourceBreakTool,
        brush_construction::BrushEnvironment,
        dry_tool::SourceDryTool,
        flood_tool::SourceFloodTool,
        move_tool::SourceMoveTool,
        source_brush_ship::{BrushTarget, SourceBrushBlend, current_ship_provider},
    },
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};
type Control = Rc<RefCell<SourceCameraControl>>;
type Provider = Rc<RefCell<SourceGameParameterProvider>>;
pub(crate) struct SourceGuiToolFactory {
    pub environment:
        Box<dyn FnMut(Control, Provider, BrushTarget) -> Result<BrushEnvironment, String>>,
    pub reader: FileReader,
    pub textures: Arc<Mutex<dyn TextureBackend>>,
    pub context: ResourceHandle,
    pub resources: ResourceRuntime,
    pub start_drag: Rc<RefCell<Box<dyn FnMut()>>>,
}
impl SourceGuiToolFactory {
    /// Connect GUI's original constructor order to retained native ship/pass objects.
    pub fn native(
        reader: FileReader,
        passes: crate::passes::native_pass_factory::NativePassEnvironment,
        textures: Arc<Mutex<dyn TextureBackend>>,
        state: Rc<RefCell<dyn crate::source_ship::ShipSceneStateBackend>>,
        current_ship: Rc<dyn Fn() -> Rc<RefCell<crate::source_ship::SourceShip>>>,
    ) -> Self {
        let brush_reader = reader.clone();
        let brush_textures = textures.clone();
        let context = passes.context.clone();
        let resources = passes.runtime.clone();
        let move_ship = current_ship.clone();
        Self {
            reader,
            textures,
            context,
            resources,
            start_drag: Rc::new(RefCell::new(Box::new(move || {
                move_ship().borrow().start_drag()
            }))),
            environment: Box::new(move |control, provider, target| {
                let ship = current_ship.clone();
                Ok(BrushEnvironment {
                    reader: brush_reader.clone(),
                    camera_control: control,
                    game_parameter_provider: provider,
                    textures: brush_textures.clone(),
                    context: passes.context.clone(),
                    resources: passes.runtime.clone(),
                    factory: Box::new(crate::passes::native_pass_factory::NativePassFactory(
                        passes.clone(),
                    )),
                    current_ship: Box::new(current_ship_provider(target, move || ship())),
                    blend: Box::new(SourceBrushBlend(state.clone())),
                    framebuffer: passes.framebuffers.clone(),
                })
            }),
        }
    }
    fn brush_environment(
        &mut self,
        control: Control,
        provider: Provider,
        target: BrushTarget,
    ) -> Result<BrushEnvironment, String> {
        let env = (self.environment)(control.clone(), provider.clone(), target)?;
        assert!(
            Rc::ptr_eq(&env.camera_control, &control),
            "Source GUI environment must retain supplied CameraControl"
        );
        assert!(
            Rc::ptr_eq(&env.game_parameter_provider, &provider),
            "Source GUI environment must retain supplied GameParameterProvider"
        );
        Ok(env)
    }
}
impl GuiToolFactory for SourceGuiToolFactory {
    fn break_tool(&mut self, control: Control, provider: Provider) -> Result<Rc<GuiTool>, String> {
        Ok(SourceBreakTool::new(self.brush_environment(
            control,
            provider,
            BrushTarget::MaskStruts,
        )?)?)
    }
    fn flood_tool(&mut self, control: Control, provider: Provider) -> Result<Rc<GuiTool>, String> {
        Ok(SourceFloodTool::new(self.brush_environment(
            control,
            provider,
            BrushTarget::Water,
        )?)?)
    }
    fn dry_tool(&mut self, control: Control, provider: Provider) -> Result<Rc<GuiTool>, String> {
        Ok(SourceDryTool::new(self.brush_environment(
            control,
            provider,
            BrushTarget::Water,
        )?)?)
    }
    fn move_tool(&mut self) -> Result<Rc<GuiTool>, String> {
        let start_drag = self.start_drag.clone();
        let tool = SourceMoveTool::new(
            &self.reader,
            self.textures.clone(),
            self.context.clone(),
            &self.resources,
            move || (start_drag.borrow_mut())(),
        )?;
        Ok(Rc::new(RefCell::new(tool)))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn factory_constructs_all_actual_source_tools_with_original_names_and_icons() {
        let (env, _) = crate::tools::brush_construction::tests::fixture();
        let control = env.camera_control.clone();
        let provider = env.game_parameter_provider.clone();
        let mut factory = SourceGuiToolFactory {
            reader: env.reader,
            textures: env.textures,
            context: env.context,
            resources: env.resources,
            start_drag: Rc::new(RefCell::new(Box::new(|| {}))),
            environment: Box::new(|control, provider, _target| {
                let (mut env, _) = crate::tools::brush_construction::tests::fixture();
                env.camera_control = control;
                env.game_parameter_provider = provider;
                Ok(env)
            }),
        };
        let tools = [
            factory
                .break_tool(control.clone(), provider.clone())
                .unwrap(),
            factory
                .flood_tool(control.clone(), provider.clone())
                .unwrap(),
            factory.dry_tool(control.clone(), provider.clone()).unwrap(),
            factory.move_tool().unwrap(),
        ];
        for (tool, name) in tools.iter().zip(["Break", "Flood", "Dry", "Move"]) {
            assert_eq!(tool.borrow().name(), name);
            // Both source Texture2D getters refer to independently created icon textures.
            assert!(!std::ptr::eq(
                tool.borrow().texture(),
                tool.borrow().active_texture()
            ));
        }
        control.borrow().camera().translate(1.0, 0.0);
    }
}
