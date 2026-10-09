//! Actual original GLSL/native object checks; synthetic window queries only.
use crate::{native_gl_backend::NativeGlBackend, texture::TextureBackend,
    fbo::FramebufferBackend, source_ship::ShipSceneStateBackend,
    shader_program::ProgramBackend, gl_data_holder::SourceGlDataHolder,
    i_drawable::IDrawable};
use std::{cell::RefCell,rc::Rc,sync::{Arc,Mutex}};

#[test]
#[ignore="Requires the native Windows OpenGL driver"]
fn actual_gpu_native_source_shader_and_framebuffer_errors_propagate() {
    crate::native_gl_mips::with_current(|| {
        let mut driver=NativeGlBackend::new()?;
        let runtime=crate::resource::ResourceRuntime::default();
        let context=runtime.allocate(&[],||{});
        let backend=Arc::new(Mutex::new(driver.clone()));
        let failed=crate::shader::Shader::new("#version 150 core\nthis is not GLSL",35632,vec![],
            backend.clone(),context.clone(),&runtime);
        assert!(matches!(failed,Err(ref message) if message=="Failed to create Shader"));
        runtime.run_main();assert_eq!(driver.error(),0);
        let vertex=Arc::new(crate::shader::Shader::new("#version 150 core\nin vec3 Position;out vec2 mismatch;void main(){mismatch=Position.xy;gl_Position=vec4(Position,1);}",
            35633,vec!["Position".into()],backend.clone(),context.clone(),&runtime)?);
        let fragment=Arc::new(crate::shader::Shader::new("#version 150 core\nin vec3 mismatch;out vec4 Color;void main(){Color=vec4(mismatch,1);}",
            35632,vec!["Color".into()],backend.clone(),context.clone(),&runtime)?);
        let unlinked=crate::shader_program::ShaderProgram::new(vec![vertex,fragment],backend.clone(),context.clone(),&runtime);
        assert!(driver.is_program(unlinked.id()),"Source program construction retains the native object despite link failure");
        assert!(!driver.program_info_log(unlinked.id()).is_empty());
        driver.use_program(unlinked.id());assert_eq!(driver.error(),1282,"Failed native link cannot be used");
        let incomplete=driver.create_framebuffer();driver.bind_framebuffer(36160,incomplete);
        assert_ne!(driver.framebuffer_status(),36053);
        driver.clear(16384);
        let caught=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||FramebufferBackend::check_error(&mut driver,"Native failure probe")));
        let payload=caught.expect_err("Source default GL check must propagate the native failure");
        let message=payload.downcast_ref::<String>().map(String::as_str).or_else(||payload.downcast_ref::<&str>().copied()).unwrap_or("");
        assert_eq!(message,"OpenGL Error (GL_INVALID_FRAMEBUFFER_OPERATION) at Native failure probe");
        assert_eq!(driver.error(),0,"checkError consumes exactly the reported native error");
        driver.bind_framebuffer(36160,0);driver.delete_framebuffer(incomplete);
        context.close();runtime.run_main();assert_eq!(driver.error(),0);
        Ok(())
    }).unwrap();
    crate::native_gl_mips::release_current_thread();
}

#[test]
#[ignore="Requires the native Windows OpenGL driver"]
fn actual_gpu_native_source_ship_constructs_physics_renders_and_defers_cleanup() {
    let _palette=crate::main_globals::fixture_materials();
    let folder=std::env::temp_dir().join(format!("ss2_native_ship_{}_{}",std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&folder).unwrap();
    let base=folder.join("Native_base.png");
    image::RgbaImage::from_pixel(4,4,image::Rgba([0x5d,0x5d,0x60,255])).save(&base).unwrap();
    crate::native_gl_mips::with_current(|| {
        let mut driver=NativeGlBackend::new()?;
        println!("Native SourceShip GL vendor={}, renderer={}, version={}",driver.driver_string(7936),driver.driver_string(7937),driver.driver_string(7938));
        assert_eq!(driver.error(),0);
        // SourceWindow supplies synthetic 400x200 screen queries only. Rendering,
        // resource IDs, shader compilation, stencils and texture storage are real.
        let (window,runtime,_)=crate::window::tests::fixture();
        let context=runtime.allocate(&[],||{});
        let backend=Arc::new(Mutex::new(driver.clone()));
        let scene=Rc::new(RefCell::new(driver.clone()));
        let vertices=Arc::new(crate::vertex_shaders::SourceVertexShaders::new(backend.clone(),context.clone(),&runtime));
        let environment=crate::passes::native_pass_factory::NativePassEnvironment {
            shaders:backend.clone(),programs:backend.clone(),vertices,
            buffers:backend.clone(),arrays:backend.clone(),draws:backend.clone(),
            renderbuffers:backend.clone(),framebuffers:backend.clone(),state:scene.clone(),
            fullscreen:Rc::new(RefCell::new(None)),context:context.clone(),runtime:runtime.clone(),
        };
        let statics=Arc::new(crate::ship_shaders::SourceShipShaders::new(backend.clone(),backend.clone(),context.clone(),&runtime,
            |_|assert_eq!(driver.error(),0))?);
        let struts=Arc::new(crate::ship_struts::SourceShipStrutShader::new(backend.clone(),context.clone(),&runtime)?);
        let resolver=crate::ship_resources::source_texture_resolver_globals(crate::ship_resources::SourceTextureEnvironment {
            backend:backend.clone(),context:context.clone(),runtime:runtime.clone(),
        });
        let thumbnail=Rc::new(crate::ship_thumbnail::ShipThumbnail::from_base_file(&base)?);
        let camera=Rc::new(crate::camera_2d::SourceCamera2D::new(64,64));
        let control=Rc::new(RefCell::new(crate::camera_control::SourceCameraControl::with_camera(window,camera.clone())));
        let ship=crate::source_ship::SourceShip::from_thumbnail_current(thumbnail,crate::main_globals::get_global_materials,
            control,environment,backend.clone(),statics,struts,resolver,scene,Box::new(||0))?;
        assert_eq!((ship.dat.width,ship.dat.height),(4,4));
        assert!(driver.is_program(ship.shader.id()) && driver.is_program(ship.struts.shader.id()));
        assert_eq!(driver.error(),0,"Constructor produced a native GL error");
        let initial=ship.physics.borrow().pos_vel.source_texture().texture.download_floats(4,4,4)?;
        assert_eq!(initial.len(),64);
        assert!(initial.iter().all(|value|value.is_finite()));
        let provider=crate::game_parameter_provider_kt::get_game_parameter_provider();
        provider.borrow_mut().set_physics_steps(2);provider.borrow_mut().set_water_steps(1);
        ship.set_time(0.0);
        ship.events.borrow_mut().moving=true;
        ship.update()?;
        driver.finish();
        assert_eq!(driver.error(),0,"Original native physics passes produced a GL error");
        let moved=ship.physics.borrow().pos_vel.source_texture().texture.download_floats(4,4,4)?;
        assert!(moved.iter().all(|value|value.is_finite()));
        assert_ne!(initial,moved,"The original pass graph must advance native positions/velocities");
        let color=crate::texture_2d::SourceTexture2D::new(None,[64,64],6408,32856,5121,false,
            Arc::new(|texture| {texture.set_parameter(10240,9728);texture.set_parameter(10241,9728);}),backend.clone(),context.clone(),&runtime);
        let stencil=crate::render_buffer::RenderBuffer::new(64,64,35056,backend.clone(),context.clone(),&runtime);
        let fbo=crate::fbo::Fbo::new(backend.clone(),context.clone(),&runtime);
        fbo.bind_texture(&color,36064);fbo.bind_texture(&stencil,33306);
        fbo.bind();driver.draw_buffers(&[36064]);driver.set_viewport([0,0,64,64]);
        assert_eq!(driver.framebuffer_status(),36053);
        driver.clear_color([0.,0.,0.,0.]);driver.clear(16384|1024|256);
        ship.render();driver.finish();
        assert_eq!(driver.error(),0,"Original hull/strut render produced a GL error");
        let pixels=driver.read_rgba8([64,64]);
        let visible=pixels.chunks_exact(4).filter(|pixel|pixel[3]!=0).count();
        assert!(visible>0,"Native SourceShip must emit visible pixels to the real offscreen target");
        println!("Native original SourceShip rendered {visible} nontransparent pixels after physics");
        fbo.unbind();
        let location=ship.shader.uniform_location("transform");
        assert!(location>=0);
        let before=unsafe {driver.matrix_uniform_value(ship.shader.id(),location)};
        ship.close();assert!(ship.freed());
        camera.translate(2.,1.);
        let closed_pending=unsafe {driver.matrix_uniform_value(ship.shader.id(),location)};
        assert_ne!(before,closed_pending,"Source camera callback remains registered until queued free runs");
        runtime.run_main();camera.translate(2.,1.);
        assert_eq!(unsafe {driver.matrix_uniform_value(ship.shader.id(),location)},closed_pending,
            "Source Ship.free removes hull camera subscription after queued cleanup");
        assert!(driver.is_program(ship.shader.id()),"Ship.close must not eagerly delete its program");
        let program=ship.shader.id();context.close();
        assert!(driver.is_program(program),"Native deletion is deferred until runMain");
        runtime.run_main();assert!(!driver.is_program(program));
        assert_eq!(driver.error(),0,"Native deferred cleanup produced a GL error");
        Ok(())
    }).unwrap();
    crate::native_gl_mips::release_current_thread();
    std::fs::remove_file(base).unwrap();std::fs::remove_dir(folder).unwrap();
}
