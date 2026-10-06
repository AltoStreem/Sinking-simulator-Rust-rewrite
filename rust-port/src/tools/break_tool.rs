//! Source BreakTool.destroyPass: cut Y/Z connections, retaining material flags.
use crate::{GpuShipPhysicsAssets, Vec2};

pub(crate) fn apply(physics: &mut GpuShipPhysicsAssets, cursor: Vec2, radius: f32) {
    physics.break_brush = Some([cursor.x, cursor.y, radius, 1.0]);
}

#[cfg(test)]
fn cut_links(
    width: usize,
    height: usize,
    positions: &[Vec2],
    input: &[[u32; 4]],
    cursor: Vec2,
    radius: f32,
) -> Vec<[u32; 4]> {
    let mut output = input.to_vec();
    for index in 0..positions.len() {
        if positions[index].distance(cursor) < radius {
            output[index][1] = 0;
            output[index][2] = 0;
        }
        let x = (index % width) as isize;
        let y = (index / width) as isize;
        for (direction, (dx, dy)) in crate::EIGHT_NEIGHBORS.iter().copied().enumerate() {
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0
                && ny >= 0
                && nx < width as isize
                && ny < height as isize
                && positions[ny as usize * width + nx as usize].distance(cursor) < radius
            {
                output[index][1] &= !(1 << direction);
                output[index][2] &= !(1 << direction);
            }
        }
    }
    output
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_destroy_cuts_both_ends_without_deleting_material() {
        let positions = [Vec2::ZERO, Vec2::X, Vec2::new(2.0, 0.0)];
        let input = [[10, 1, 1, 7], [8, 17, 17, 9], [9, 16, 0, 3]];
        let cut = cut_links(3, 1, &positions, &input, Vec2::X, 0.5);
        assert_eq!(cut, vec![[10, 0, 0, 7], [8, 0, 0, 9], [9, 0, 0, 3]]);
    }
    #[test]
    fn source_destroy_uses_live_positions_and_strict_radius() {
        let positions = [Vec2::ZERO, Vec2::new(20.0, 0.0)];
        let input = [[8, 1, 1, 0], [8, 16, 16, 0]];
        assert_eq!(
            cut_links(2, 1, &positions, &input, Vec2::new(10.0, 0.0), 10.0),
            input
        );
        let cut = cut_links(2, 1, &positions, &input, Vec2::new(20.0, 0.0), 0.1);
        assert_eq!(cut, vec![[8, 0, 0, 0], [8, 0, 0, 0]]);
    }
}

/// Original class input/uniform callbacks; pass construction/update remains separate.
pub(crate) struct SourceBreakCallbacks<B: super::brush_callbacks::BrushUniforms>(
    pub super::brush_callbacks::BrushCallbacks<B>,
);
impl<B: super::brush_callbacks::BrushUniforms>
    crate::input_handler::InputHandler<crate::window::SourceWindow> for SourceBreakCallbacks<B>
{
    fn on_size(
        &mut self,
        blocked: bool,
        _win: &crate::window::SourceWindow,
        height: i32,
        width: i32,
    ) -> bool {
        self.0.on_size(blocked, height, width, true)
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
pub(crate) struct SourceBreakTool {
    texture: crate::texture_2d::SourceTexture2D,
    active_texture: crate::texture_2d::SourceTexture2D,
    pub callbacks: SourceBreakCallbacks<super::brush_runtime::BrushPassUniforms>,
    pub runtime: Option<super::brush_runtime::BrushRuntime>,
    pub resource: Option<crate::resource::ResourceHandle>,
    camera_control: std::rc::Rc<std::cell::RefCell<crate::camera_control::SourceCameraControl>>,
    game_parameter_provider:
        std::rc::Rc<std::cell::RefCell<crate::game_parameters::SourceGameParameterProvider>>,
}
impl SourceBreakTool {
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
            "icons/Break.png",
            std::sync::Arc::new(super::break_tool_texture::configure),
        )?;
        let active_texture = env.icon(
            "icons/Break2.png",
            std::sync::Arc::new(super::break_tool_active_texture::configure),
        )?;
        let passes = super::brush_construction::passes(
            env.factory.as_mut(),
            PREVIEW_GLSL,
            ACTION_GLSL,
            true,
        );
        let window = env.camera_control.borrow().window();
        super::brush_construction::initial_uniforms(&passes, &window, true);
        let callbacks = SourceBreakCallbacks(super::brush_callbacks::BrushCallbacks::new(
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
        *removal_key.borrow_mut() = Some(super::break_tool_free_camera_reference::key(
            std::rc::Rc::as_ptr(&tool) as usize as u64,
        ));
        let camera = env.camera_control.borrow().camera();
        camera.add_camera_callback(super::break_tool_camera_reference::callback(tool.clone()));
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
        let key = super::break_tool_free_camera_reference::key(
            std::rc::Rc::as_ptr(&tool) as usize as u64
        );
        let cleanup_camera = camera.clone();
        let resource =
            resources.allocate_local(&[], move || cleanup_camera.remove_camera_callback(&key));
        tool.borrow_mut().resource = Some(resource.clone());
        camera.add_camera_callback(super::break_tool_camera_reference::callback(tool));
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
        let callbacks = SourceBreakCallbacks(super::brush_callbacks::BrushCallbacks::new(
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
impl crate::input_handler::InputHandler<crate::window::SourceWindow> for SourceBreakTool {
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
impl super::tool::SourceTool<crate::window::SourceWindow> for SourceBreakTool {
    type Texture = crate::texture_2d::SourceTexture2D;
    fn texture(&self) -> &Self::Texture {
        &self.texture
    }
    fn active_texture(&self) -> &Self::Texture {
        &self.active_texture
    }
    fn name(&self) -> &str {
        "Break"
    }
    fn update(&mut self) {
        SourceBreakTool::update(self);
    }
}

const PREVIEW_GLSL: &str = r###"
            #version 150 core
            out vec4 col;

            uniform mat4 u_inv;
            uniform vec2 u_mouse;
            uniform float u_size;
            uniform vec2 u_scale;
            uniform vec2 u_window;

            const vec3 color = vec3(1, 0, 0);

            void main() {
                vec4 ref = u_inv * vec4((u_mouse * u_window * 2 - vec2(1)) * vec2(1, -1), 0, 1);
                vec4 base = u_inv * vec4(gl_FragCoord.xy * u_scale * u_window * 2 - vec2(1), 0, 1);
                float dist = distance(ref.xy, base.xy);
                col = vec4(color, 0.5 - smoothstep(u_size, u_size + dist / 10, dist) * 0.5);
            }
            "###;
const ACTION_GLSL: &str = r###"
            #version 150 core
            out uvec4 out_mask_struts;

            uniform sampler2D in_pos_vel;
            uniform usampler2D in_mask_struts;
            uniform mat4 u_inv;
            uniform vec2 u_mouse;
            uniform float u_size;
            uniform vec2 u_window;

            const vec3 color = vec3(1, 0, 0);

            const ivec2 s[8] = ivec2[8](
                ivec2(1, 0),
                ivec2(1, 1),
                ivec2(0, 1),
                ivec2(-1, 1),
                ivec2(-1, 0),
                ivec2(-1, -1),
                ivec2(0, -1),
                ivec2(1, -1)
            );

            void main() {
                ivec2 idx = ivec2(gl_FragCoord.xy);
                vec4 ref = u_inv * vec4((u_mouse * u_window * 2 - vec2(1)) * vec2(1, -1), 0, 1);
                vec2 pos = texelFetch(in_pos_vel, idx, 0).xy;
                float dist = distance(ref.xy, pos);
                out_mask_struts = texelFetch(in_mask_struts, idx, 0);
                out_mask_struts.yz &= 255u * uint(dist >= u_size);
                for (uint i = 0u; i < 8u; i++) {
                    vec2 other = texelFetch(in_pos_vel, idx + s[i], 0).xy;
                    float diff = distance(ref.xy, other);
                    out_mask_struts.yz &= ~(uint(diff < u_size) << i);
                }
            }
    "###;
