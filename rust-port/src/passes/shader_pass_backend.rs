//! Explicit GPU adapter for StandardPass/StencilPass. No OpenGL calls are
//! silently simulated: concrete shader, uniform and state operations are required.
use std::any::Any;
pub(crate) trait ShaderPassBackend {
    fn create_shader(&mut self, fragment: &str, output_bindings: &[String]);
    fn uniform_location(&mut self, name: &str) -> i32;
    fn start_shader(&mut self);
    fn stop_shader(&mut self);
    fn write_float_uniform(&mut self, location: i32, values: &[f32]);
    fn write_int_uniform(&mut self, location: i32, values: &[i32]);
    fn write_matrix_uniform(&mut self, location: i32, transposed: bool, matrix: &[f32; 16]);
    fn stencil_test(&mut self, enabled: bool);
    fn color_mask(&mut self, mask: [bool; 4]);
    fn current_state(&mut self) -> Box<dyn Any>;
    fn apply_state(&mut self);
    fn restore_state(&mut self, state: Box<dyn Any>);
    fn draw_fullscreen(&mut self);
}
/// ShaderProgram.setArg starts/stops the program, including constructor samplers.
pub(crate) fn initialize(
    backend: &mut dyn ShaderPassBackend,
    src: &[String],
    dst: &[String],
    fragment: &str,
) {
    backend.create_shader(fragment, dst);
    for (unit, name) in src.iter().enumerate() {
        let location = backend.uniform_location(name);
        backend.start_shader();
        backend.write_int_uniform(location, &[unit as i32]);
        backend.stop_shader();
    }
}
pub(crate) fn set_floats(backend: &mut dyn ShaderPassBackend, name: &str, values: &[f32]) {
    let location = backend.uniform_location(name);
    backend.start_shader();
    assert!(
        (1..=4).contains(&values.len()),
        "Maximum 4 vector components, {} given.",
        values.len()
    );
    backend.write_float_uniform(location, values);
    backend.stop_shader();
}
pub(crate) fn set_ints(backend: &mut dyn ShaderPassBackend, name: &str, values: &[i32]) {
    let location = backend.uniform_location(name);
    backend.start_shader();
    assert!(
        (1..=4).contains(&values.len()),
        "Maximum 4 vector components, {} given.",
        values.len()
    );
    backend.write_int_uniform(location, values);
    backend.stop_shader();
}
pub(crate) fn set_matrix(
    backend: &mut dyn ShaderPassBackend,
    name: &str,
    transposed: bool,
    matrix: &[f32; 16],
) {
    let location = backend.uniform_location(name);
    backend.start_shader();
    backend.write_matrix_uniform(location, transposed, matrix);
    backend.stop_shader();
}
