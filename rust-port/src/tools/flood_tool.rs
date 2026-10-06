use crate::{GpuShipPhysicsAssets, GpuShipPhysicsSnapshot, ShaderBuffer};
use bevy::{math::Vec2, prelude::Assets};

pub(super) fn apply(
    cursor: Vec2,
    radius: f32,
    snapshot: &GpuShipPhysicsSnapshot,
    physics: &mut GpuShipPhysicsAssets,
    buffers: &mut Assets<ShaderBuffer>,
) {
    super::water_brush::apply(true, cursor, radius, snapshot, physics, buffers);
}

/// Original class input/uniform callbacks; pass construction/update remains separate.
pub(crate) struct SourceFloodCallbacks<B: super::brush_callbacks::BrushUniforms>(
    pub super::brush_callbacks::BrushCallbacks<B>,
);
impl<B: super::brush_callbacks::BrushUniforms>
    crate::input_handler::InputHandler<crate::window::SourceWindow> for SourceFloodCallbacks<B>
{
    fn on_size(
        &mut self,
        blocked: bool,
        _win: &crate::window::SourceWindow,
        height: i32,
        width: i32,
    ) -> bool {
        self.0.on_size(blocked, height, width, false)
    }
    fn on_cursor_pos(
        &mut self,
        blocked: bool,
        _win: &crate::window::SourceWindow,
        x: f64,
        y: f64,
    ) -> bool {
        self.0.on_cursor(blocked, x, y)
    }
    fn on_mouse_button(
        &mut self,
        blocked: bool,
        _win: &crate::window::SourceWindow,
        button: i32,
        action: i32,
        mods: i32,
    ) -> bool {
        self.0
            .activation
            .on_mouse_button(blocked, button, action, mods)
    }
}

/// Original class constructor/update path through converted source objects.
/// Concrete graphics backends and active Bevy integration remain pending.
pub(crate) struct SourceFloodTool {
    texture: crate::texture_2d::SourceTexture2D,
    active_texture: crate::texture_2d::SourceTexture2D,
    pub callbacks: SourceFloodCallbacks<super::brush_runtime::BrushPassUniforms>,
    pub runtime: Option<super::brush_runtime::BrushRuntime>,
    pub resource: Option<crate::resource::ResourceHandle>,
    camera_control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
    game_parameter_provider:
        std::rc::Rc<std::cell::RefCell<crate::game_parameters::SourceGameParameterProvider>>,
}
impl SourceFloodTool {
    pub fn game_parameter_provider(
        &self,
    ) -> std::rc::Rc<std::cell::RefCell<crate::game_parameters::SourceGameParameterProvider>> {
        self.game_parameter_provider.clone()
    }
    pub fn camera_control(
        &self,
    ) -> std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>> {
        self.camera_control.clone()
    }
    pub fn new(
        mut env: super::brush_construction::BrushEnvironment,
    ) -> Result<std::rc::Rc<std::cell::RefCell<Self>>, String> {
        let removal_key = std::rc::Rc::new(std::cell::RefCell::new(None));
        let cleanup_key = removal_key.clone();
        let cleanup_control = env.camera_control.clone();
        let resource = env.resources.allocate_local(&[], move || {
            if let Some(key) = cleanup_key.borrow().as_ref() {
                cleanup_control
                    .borrow()
                    .camera()
                    .remove_camera_callback(key);
            }
        });
        let texture = env.icon(
            "icons/Flood.png",
            std::sync::Arc::new(super::flood_tool_texture::configure),
        )?;
        let active_texture = env.icon(
            "icons/Flood2.png",
            std::sync::Arc::new(super::flood_tool_active_texture::configure),
        )?;
        let passes = super::brush_construction::passes(
            env.factory.as_mut(),
            PREVIEW_GLSL,
            ACTION_GLSL,
            false,
        );
        let window = env.camera_control.borrow().window();
        super::brush_construction::initial_uniforms(&passes, &window, false);
        let callbacks = SourceFloodCallbacks(super::brush_callbacks::BrushCallbacks::new(
            super::brush_runtime::BrushPassUniforms(passes.clone()),
            window,
        ));
        let tool = std::rc::Rc::new(std::cell::RefCell::new(Self {
            texture,
            active_texture,
            callbacks,
            runtime: None,
            resource: Some(resource),
            camera_control: env.camera_control.clone(),
            game_parameter_provider: env.game_parameter_provider.clone(),
        }));
        *removal_key.borrow_mut() = Some(super::flood_tool_free_camera_reference::key(
            std::rc::Rc::as_ptr(&tool) as usize as u64,
        ));
        let camera = env.camera_control.borrow().camera();
        camera.add_camera_callback(super::flood_tool_camera_reference::callback(tool.clone()));
        let parameters = env.game_parameter_provider.clone();
        let runtime = super::brush_runtime::BrushRuntime::from_shared(
            passes,
            env.current_ship,
            move || parameters.borrow().tool(),
            env.blend,
            env.framebuffer,
            env.context,
            env.resources,
        )?;
        tool.borrow_mut().runtime = Some(runtime);
        Ok(tool)
    }
    /// Registers immediately, as Camera2D.addCameraCallback does; cleanup only removes the method reference.
    pub fn register_camera(
        tool: std::rc::Rc<std::cell::RefCell<Self>>,
        camera: std::rc::Rc<crate::camera_2d::SourceCamera2D>,
        resources: &crate::resource::ResourceRuntime,
    ) -> crate::resource::ResourceHandle {
        assert!(
            tool.borrow().resource.is_none(),
            "Tool camera already registered"
        );
        let key = super::flood_tool_free_camera_reference::key(
            std::rc::Rc::as_ptr(&tool) as usize as u64
        );
        let cleanup_camera = camera.clone();
        let resource =
            resources.allocate_local(&[], move || cleanup_camera.remove_camera_callback(&key));
        tool.borrow_mut().resource = Some(resource.clone());
        camera.add_camera_callback(super::flood_tool_camera_reference::callback(tool));
        resource
    }
    pub fn from_parts(
        texture: crate::texture_2d::SourceTexture2D,
        active_texture: crate::texture_2d::SourceTexture2D,
        camera_control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
        game_parameter_provider: std::rc::Rc<
            std::cell::RefCell<crate::game_parameters::SourceGameParameterProvider>,
        >,
        mut runtime: super::brush_runtime::BrushRuntime,
    ) -> Self {
        runtime.use_source_parameters(game_parameter_provider.clone());
        let callbacks = SourceFloodCallbacks(super::brush_callbacks::BrushCallbacks::new(
            runtime.uniforms(),
            camera_control.borrow().window(),
        ));
        Self {
            texture,
            active_texture,
            callbacks,
            runtime: Some(runtime),
            resource: None,
            camera_control,
            game_parameter_provider,
        }
    }
    pub fn on_camera_change(&mut self, matrix: bevy::math::Mat4) {
        self.callbacks.0.on_camera_change(matrix);
    }
    pub fn update(&mut self) {
        self.runtime
            .as_mut()
            .expect("Tool framebuffer not initialized")
            .update(self.callbacks.0.activation.active)
            .unwrap_or_else(|error| panic!("{error}"));
    }
}
impl crate::input_handler::InputHandler<crate::window::SourceWindow> for SourceFloodTool {
    fn on_size(
        &mut self,
        blocked: bool,
        win: &crate::window::SourceWindow,
        height: i32,
        width: i32,
    ) -> bool {
        crate::input_handler::InputHandler::on_size(
            &mut self.callbacks,
            blocked,
            win,
            height,
            width,
        )
    }
    fn on_cursor_pos(
        &mut self,
        blocked: bool,
        win: &crate::window::SourceWindow,
        x: f64,
        y: f64,
    ) -> bool {
        crate::input_handler::InputHandler::on_cursor_pos(&mut self.callbacks, blocked, win, x, y)
    }
    fn on_mouse_button(
        &mut self,
        blocked: bool,
        win: &crate::window::SourceWindow,
        button: i32,
        action: i32,
        mods: i32,
    ) -> bool {
        crate::input_handler::InputHandler::on_mouse_button(
            &mut self.callbacks,
            blocked,
            win,
            button,
            action,
            mods,
        )
    }
}
impl super::tool::SourceTool<crate::window::SourceWindow> for SourceFloodTool {
    type Texture = crate::texture_2d::SourceTexture2D;
    fn texture(&self) -> &Self::Texture {
        &self.texture
    }
    fn active_texture(&self) -> &Self::Texture {
        &self.active_texture
    }
    fn name(&self) -> &str {
        "Flood"
    }
    fn update(&mut self) {
        SourceFloodTool::update(self);
    }
}

const PREVIEW_GLSL: &str = r###"
            #version 150 core
            out vec4 col;

            uniform mat4 u_inv;
            uniform vec2 u_mouse;
            uniform float u_size;
            uniform vec2 u_window;
            uniform vec2 u_scale;

            const vec3 color = vec3(0, 0, 1);

            void main() {
                vec4 ref = u_inv * vec4((u_mouse * u_window * 2 - vec2(1)) * vec2(1, -1), 0, 1);
                vec4 base = u_inv * vec4(gl_FragCoord.xy * u_scale * u_window * 2 - vec2(1), 0, 1);
                float dist = distance(ref.xy, base.xy);
                col = vec4(color, 0.5 - smoothstep(u_size, u_size + dist / 10, dist) * 0.5);
            }
            "###;
const ACTION_GLSL: &str = r###"
            #version 150 core
            out vec4 out_water;

            uniform sampler2D in_pos_vel;
            uniform sampler2D in_water;
            uniform mat4 u_inv;
            uniform vec2 u_mouse;
            uniform float u_size;
            uniform vec2 u_window;

            const vec3 color = vec3(1, 0, 0);

            void main() {
                ivec2 idx = ivec2(gl_FragCoord.xy);
                vec4 ref = u_inv * vec4((u_mouse * u_window * 2 - vec2(1)) * vec2(1, -1), 0, 1);
                vec2 pos = texelFetch(in_pos_vel, idx, 0).xy;
                float dist = distance(ref.xy, pos);
                out_water = texelFetch(in_water, idx, 0);
                if (dist < u_size)
                    out_water.x += u_size - dist;
            }
    "###;
