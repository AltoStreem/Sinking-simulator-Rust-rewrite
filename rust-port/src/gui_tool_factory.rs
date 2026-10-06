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
        break_tool::SourceBreakTool, brush_construction::BrushEnvironment, dry_tool::SourceDryTool,
        flood_tool::SourceFloodTool, move_tool::SourceMoveTool,
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
    pub environment: Box<dyn FnMut(Control, Provider) -> Result<BrushEnvironment, String>>,
    pub reader: FileReader,
    pub textures: Arc<Mutex<dyn TextureBackend>>,
    pub context: ResourceHandle,
    pub resources: ResourceRuntime,
    pub start_drag: Rc<RefCell<Box<dyn FnMut()>>>,
}
impl SourceGuiToolFactory {
    fn brush_environment(
        &mut self,
        control: Control,
        provider: Provider,
    ) -> Result<BrushEnvironment, String> {
        let env = (self.environment)(control.clone(), provider.clone())?;
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
        Ok(SourceBreakTool::new(
            self.brush_environment(control, provider)?,
        )?)
    }
    fn flood_tool(&mut self, control: Control, provider: Provider) -> Result<Rc<GuiTool>, String> {
        Ok(SourceFloodTool::new(
            self.brush_environment(control, provider)?,
        )?)
    }
    fn dry_tool(&mut self, control: Control, provider: Provider) -> Result<Rc<GuiTool>, String> {
        Ok(SourceDryTool::new(
            self.brush_environment(control, provider)?,
        )?)
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
            environment: Box::new(|control, provider| {
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
