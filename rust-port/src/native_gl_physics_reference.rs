//! Actual OpenGL execution of unmodified retained source physics shaders.
//! Verification adapter only: no CPU arithmetic is used for reference outputs.
use std::ffi::{CString, c_char, c_void};
#[test]
fn retained_glsl_strings_match_original_source_hash_inventory() {
    use crate::ship_physics_shaders as source;
    let inventory: serde_json::Value = serde_json::from_str(include_str!(
        "../tools/fixtures/source-physics-shader-hashes.json"
    ))
    .unwrap();
    let entries = inventory["shaders"].as_array().unwrap();
    let shaders = [
        ("FILTER_DYNAMIC", source::FILTER_DYNAMIC),
        ("FORCES", source::FORCES),
        ("INTEGRATE", source::INTEGRATE),
        ("FILTER_PERMEABLE", source::FILTER_PERMEABLE),
        ("FILL_WATER", source::FILL_WATER),
        ("FLOW_WATER", source::FLOW_WATER),
        ("TRANSPORT_WATER", source::TRANSPORT_WATER),
        ("UPDATE_MASS", source::UPDATE_MASS),
        ("FILTER_OCCUPIED", source::FILTER_OCCUPIED),
        ("REPAIR_MASK", source::REPAIR_MASK),
        ("MOVE", source::MOVE),
    ];
    assert_eq!(entries.len(), shaders.len());
    for (entry, (name, text)) in entries.iter().zip(shaders) {
        assert_eq!(entry["name"].as_str().unwrap(), name);
        assert_eq!(entry["bytes"].as_u64().unwrap(), text.len() as u64);
        assert_eq!(
            entry["crc32"].as_str().unwrap(),
            format!("{:08x}", crc32fast::hash(text.as_bytes()))
        );
    }
}
type Pointer = *const c_void;
macro_rules! functions {
    ($(fn $name:ident($($argument:ty),*) -> $result:ty;)+) => {
        #[allow(non_snake_case)]
        struct Api { $($name:unsafe extern "system" fn($($argument),*) -> $result,)+
            _library: libloading::Library }
        impl Api {
            fn load() -> Result<Self, String> {
                // System OpenGL provider, retained until all objects are freed.
                let library = unsafe { libloading::Library::new("opengl32.dll") }.map_err(|e| e.to_string())?;
                Ok(Self { $($name: {
                    let name = CString::new(stringify!($name)).unwrap();
                    let address = crate::native_gl_mips::procedure(&name);
                    if !address.is_null() && address as usize > 3 && address as isize != -1 {
                        unsafe { std::mem::transmute::<Pointer, unsafe extern "system" fn($($argument),*) -> $result>(address) }
                    } else {
                        unsafe { *library.get::<unsafe extern "system" fn($($argument),*) -> $result>(name.as_bytes_with_nul())
                            .map_err(|e| format!("{}: {e}", stringify!($name)))? }
                    }
                },)+ _library: library })
            }
        }
    }
}
functions! {
    fn glGetString(u32)->*const c_char;
    fn glGenTextures(i32,*mut u32)->();
    fn glDeleteTextures(i32,*const u32)->();
    fn glBindTexture(u32,u32)->();
    fn glTexImage2D(u32,i32,i32,i32,i32,i32,u32,u32,Pointer)->();
    fn glTexParameteri(u32,u32,i32)->();
    fn glGetTexImage(u32,i32,u32,u32,*mut c_void)->();
    fn glActiveTexture(u32)->();
    fn glCreateShader(u32)->u32;
    fn glShaderSource(u32,i32,*const *const c_char,*const i32)->();
    fn glCompileShader(u32)->();
    fn glGetShaderiv(u32,u32,*mut i32)->();
    fn glGetShaderInfoLog(u32,i32,*mut i32,*mut c_char)->();
    fn glDeleteShader(u32)->();
    fn glCreateProgram()->u32;
    fn glAttachShader(u32,u32)->();
    fn glBindAttribLocation(u32,u32,*const c_char)->();
    fn glBindFragDataLocation(u32,u32,*const c_char)->();
    fn glLinkProgram(u32)->();
    fn glGetProgramiv(u32,u32,*mut i32)->();
    fn glGetProgramInfoLog(u32,i32,*mut i32,*mut c_char)->();
    fn glDeleteProgram(u32)->();
    fn glUseProgram(u32)->();
    fn glGetUniformLocation(u32,*const c_char)->i32;
    fn glUniform1i(i32,i32)->();
    fn glUniform1f(i32,f32)->();
    fn glUniform2f(i32,f32,f32)->();
    fn glGenVertexArrays(i32,*mut u32)->();
    fn glBindVertexArray(u32)->();
    fn glDeleteVertexArrays(i32,*const u32)->();
    fn glGenBuffers(i32,*mut u32)->();
    fn glBindBuffer(u32,u32)->();
    fn glBufferData(u32,isize,Pointer,u32)->();
    fn glDeleteBuffers(i32,*const u32)->();
    fn glEnableVertexAttribArray(u32)->();
    fn glVertexAttribPointer(u32,i32,u32,u8,i32,Pointer)->();
    fn glGenFramebuffers(i32,*mut u32)->();
    fn glBindFramebuffer(u32,u32)->();
    fn glFramebufferTexture2D(u32,u32,u32,u32,i32)->();
    fn glDeleteFramebuffers(i32,*const u32)->();
    fn glCheckFramebufferStatus(u32)->u32;
    fn glDrawBuffers(i32,*const u32)->();
    fn glGenRenderbuffers(i32,*mut u32)->();
    fn glBindRenderbuffer(u32,u32)->();
    fn glRenderbufferStorage(u32,u32,i32,i32)->();
    fn glFramebufferRenderbuffer(u32,u32,u32,u32)->();
    fn glDeleteRenderbuffers(i32,*const u32)->();
    fn glViewport(i32,i32,i32,i32)->();
    fn glDrawElements(u32,i32,u32,Pointer)->();
    fn glEnable(u32)->();
    fn glDisable(u32)->();
    fn glColorMask(u8,u8,u8,u8)->();
    fn glStencilMask(u32)->();
    fn glStencilFunc(u32,i32,u32)->();
    fn glStencilOp(u32,u32,u32)->();
    fn glClearStencil(i32)->();
    fn glClear(u32)->();
    fn glReadPixels(i32,i32,i32,i32,u32,u32,*mut c_void)->();
    fn glGetError()->u32;
}
struct Objects<'a> {
    api: &'a Api,
    textures: Vec<u32>,
    shaders: Vec<u32>,
    programs: Vec<u32>,
    buffers: Vec<u32>,
    vao: u32,
    fbo: u32,
    stencil: u32,
    extra_stencils: Vec<u32>,
}
impl<'a> Objects<'a> {
    fn new(api: &'a Api, width: i32, height: i32) -> Self {
        let mut result = Self {
            api,
            textures: vec![],
            shaders: vec![],
            programs: vec![],
            buffers: vec![0; 2],
            vao: 0,
            fbo: 0,
            stencil: 0,
            extra_stencils: vec![],
        };
        // Exact FullScreen.java vertices/indices and Nothing vertex shader.
        let vertices = [-1.0f32, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0];
        let indices = [0u32, 1, 2, 0, 2, 3];
        unsafe {
            (api.glGenVertexArrays)(1, &mut result.vao);
            (api.glBindVertexArray)(result.vao);
            (api.glGenBuffers)(2, result.buffers.as_mut_ptr());
            (api.glBindBuffer)(34962, result.buffers[0]);
            (api.glBufferData)(
                34962,
                std::mem::size_of_val(&vertices) as isize,
                vertices.as_ptr().cast(),
                35044,
            );
            (api.glEnableVertexAttribArray)(0);
            (api.glVertexAttribPointer)(0, 2, 5126, 0, 0, std::ptr::null());
            (api.glBindBuffer)(34963, result.buffers[1]);
            (api.glBufferData)(
                34963,
                std::mem::size_of_val(&indices) as isize,
                indices.as_ptr().cast(),
                35044,
            );
            (api.glGenFramebuffers)(1, &mut result.fbo);
            (api.glBindFramebuffer)(36160, result.fbo);
            (api.glGenRenderbuffers)(1, &mut result.stencil);
            (api.glBindRenderbuffer)(36161, result.stencil);
            (api.glRenderbufferStorage)(36161, 35056, width, height); // source DEPTH24_STENCIL8
            (api.glFramebufferRenderbuffer)(36160, 33306, 36161, result.stencil);
            (api.glViewport)(0, 0, width, height);
            (api.glDisable)(3042);
            (api.glDisable)(2929);
            (api.glDisable)(3089);
        }
        result
    }
    fn texture(
        &mut self,
        width: i32,
        height: i32,
        internal: i32,
        format: u32,
        kind: u32,
        data: Pointer,
    ) -> u32 {
        let mut id = 0;
        unsafe {
            (self.api.glGenTextures)(1, &mut id);
            (self.api.glBindTexture)(3553, id);
            (self.api.glTexParameteri)(3553, 10241, 9728);
            (self.api.glTexParameteri)(3553, 10240, 9728);
            (self.api.glTexParameteri)(3553, 10242, 33069);
            (self.api.glTexParameteri)(3553, 10243, 33069);
            (self.api.glTexImage2D)(3553, 0, internal, width, height, 0, format, kind, data);
        }
        self.textures.push(id);
        id
    }
    fn shader(&mut self, kind: u32, source: &str) -> Result<u32, String> {
        let source = CString::new(source).unwrap();
        let id = unsafe { (self.api.glCreateShader)(kind) };
        self.shaders.push(id);
        unsafe {
            (self.api.glShaderSource)(id, 1, &source.as_ptr(), std::ptr::null());
            (self.api.glCompileShader)(id);
            let mut ok = 0;
            (self.api.glGetShaderiv)(id, 35713, &mut ok);
            if ok == 0 {
                let mut log = vec![0u8; 16384];
                (self.api.glGetShaderInfoLog)(
                    id,
                    log.len() as i32,
                    std::ptr::null_mut(),
                    log.as_mut_ptr().cast(),
                );
                return Err(format!(
                    "Original GLSL compilation: {}",
                    String::from_utf8_lossy(&log).trim_end_matches('\0')
                ));
            }
        }
        Ok(id)
    }
    fn program(&mut self, fragment: &str, outputs: &[&str]) -> Result<u32, String> {
        let vertex = self.shader(35633, crate::vertex_shaders_nothing::SOURCE)?;
        let fragment = self.shader(35632, fragment)?;
        let id = unsafe { (self.api.glCreateProgram)() };
        self.programs.push(id);
        unsafe {
            (self.api.glAttachShader)(id, vertex);
            (self.api.glAttachShader)(id, fragment);
            (self.api.glBindAttribLocation)(id, 0, c"Position".as_ptr());
            for (index, name) in outputs.iter().enumerate() {
                let name = CString::new(*name).unwrap();
                (self.api.glBindFragDataLocation)(id, index as u32, name.as_ptr());
            }
            (self.api.glLinkProgram)(id);
            let mut ok = 0;
            (self.api.glGetProgramiv)(id, 35714, &mut ok);
            if ok == 0 {
                let mut log = vec![0u8; 16384];
                (self.api.glGetProgramInfoLog)(
                    id,
                    log.len() as i32,
                    std::ptr::null_mut(),
                    log.as_mut_ptr().cast(),
                );
                return Err(format!(
                    "Original GLSL link: {}",
                    String::from_utf8_lossy(&log).trim_end_matches('\0')
                ));
            }
        }
        Ok(id)
    }
    fn location(&self, program: u32, name: &str) -> i32 {
        let name = CString::new(name).unwrap();
        unsafe { (self.api.glGetUniformLocation)(program, name.as_ptr()) }
    }
    fn scalar(&self, program: u32, name: &str, value: f32) {
        unsafe {
            (self.api.glUniform1f)(self.location(program, name), value);
        }
    }
    fn bind(&self, program: u32, name: &str, texture: u32, unit: u32) {
        unsafe {
            (self.api.glActiveTexture)(33984 + unit);
            (self.api.glBindTexture)(3553, texture);
            (self.api.glUniform1i)(self.location(program, name), unit as i32);
        }
    }
    fn target(&self, textures: &[u32]) -> Result<(), String> {
        unsafe {
            for slot in 0..4 {
                (self.api.glFramebufferTexture2D)(
                    36160,
                    36064 + slot,
                    3553,
                    textures.get(slot as usize).copied().unwrap_or(0),
                    0,
                );
            }
            let draws: Vec<_> = (0..textures.len())
                .map(|slot| 36064 + slot as u32)
                .collect();
            (self.api.glDrawBuffers)(draws.len() as i32, draws.as_ptr());
            let status = (self.api.glCheckFramebufferStatus)(36160);
            if status != 36053 {
                return Err(format!("Original physics FBO incomplete: {status:#x}"));
            }
        }
        Ok(())
    }
    fn draw(&self) {
        unsafe {
            (self.api.glDrawElements)(4, 6, 5125, std::ptr::null());
        }
    }
    fn source_state(&self, state: &crate::gl_state::stencil_config::StencilConfig) {
        unsafe {
            (self.api.glEnable)(2960);
            if let Some(value) = &state.write_mask {
                (self.api.glStencilMask)(value.mask as u32);
            }
            if let Some(value) = &state.func {
                (self.api.glStencilFunc)(
                    value.func as u32,
                    value.reference,
                    value.value_mask as u32,
                );
            }
            if let Some(value) = &state.op {
                (self.api.glStencilOp)(
                    value.on_failure as u32,
                    value.on_depth_failure as u32,
                    value.on_success as u32,
                );
            }
        }
    }
    fn source_filter(
        &self,
        program: u32,
        mask: u32,
        state: &crate::gl_state::stencil_config::StencilConfig,
    ) {
        self.source_state(state);
        unsafe {
            (self.api.glUseProgram)(program);
            (self.api.glColorMask)(0, 0, 0, 0);
        }
        self.bind(program, "in_mask_struts", mask, 0);
        self.draw();
        unsafe {
            (self.api.glColorMask)(1, 1, 1, 1);
        }
    }
}
#[test]
#[ignore = "Requires actual native OpenGL stencil evaluation of original source configurations"]
fn original_gl_stencil_dynamic_precondition_controls_permeable_ground_cells() {
    crate::native_gl_mips::with_current(|| {
        let api=Api::load()?;let mut objects=Objects::new(&api,16,1);
        let masks:Vec<_>=(0u8..16).map(|flag|[flag,0,0,0]).collect();
        let mask=objects.texture(16,1,36220,36249,5121,masks.as_ptr().cast());
        let dynamic=objects.program(crate::ship_physics_shaders::FILTER_DYNAMIC,&[])?;
        let permeable=objects.program(crate::ship_physics_shaders::FILTER_PERMEABLE,&[])?;
        objects.target(&[mask])?;
        unsafe {(api.glStencilMask)(255);(api.glClearStencil)(0);(api.glClear)(1024);}
        let states=crate::ship_physics_stencil::states();
        objects.source_filter(dynamic,mask,&states.set7);
        objects.source_filter(permeable,mask,&states.set1If7);
        let mut stencil=[0u8;16];
        unsafe {(api.glReadPixels)(0,0,16,1,6401,5121,stencil.as_mut_ptr().cast());}
        for (flag,&bits) in stencil.iter().enumerate() {
            let dynamic=flag!=0 && flag&4==0;
            let flow=dynamic && flag&3==0;
            assert_eq!(bits,if dynamic {128}else {0} | if flow {2}else {0},"source state {flag} got stencil {bits}");
        }
        assert_eq!(stencil[8],130);
        assert_eq!(stencil[4]&2,0);assert_eq!(stencil[12]&2,0);
        assert_eq!(unsafe {(api.glGetError)()},0);
        println!("Original GL set7 + set1If7: all sixteen flags checked; active flag 8 flows, ground flags 4/12 do not");
        Ok(())
    }).unwrap();
    crate::native_gl_mips::release_current_thread();
}
impl Drop for Objects<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.api.glUseProgram)(0);
            (self.api.glBindFramebuffer)(36160, 0);
            (self.api.glBindVertexArray)(0);
            for id in &self.programs {
                (self.api.glDeleteProgram)(*id);
            }
            for id in &self.shaders {
                (self.api.glDeleteShader)(*id);
            }
            (self.api.glDeleteTextures)(self.textures.len() as i32, self.textures.as_ptr());
            (self.api.glDeleteBuffers)(self.buffers.len() as i32, self.buffers.as_ptr());
            (self.api.glDeleteFramebuffers)(1, &self.fbo);
            (self.api.glDeleteRenderbuffers)(1, &self.stencil);
            (self.api.glDeleteRenderbuffers)(
                self.extra_stencils.len() as i32,
                self.extra_stencils.as_ptr(),
            );
            (self.api.glDeleteVertexArrays)(1, &self.vao);
            (self.api.glDisable)(2960);
            (self.api.glColorMask)(1, 1, 1, 1);
            (self.api.glActiveTexture)(33984);
        }
    }
}
pub(crate) struct Output {
    pub positions: Vec<[f32; 4]>,
    pub masks: Vec<[u32; 4]>,
    pub water: Vec<[f32; 4]>,
    pub materials: Vec<[f32; 4]>,
    pub physics_steps: usize,
    pub water_updates: usize,
    pub forces: Vec<[f32;4]>,
    pub water_scratch: [Vec<[f32;4]>;4],
}

#[test]
#[ignore = "Requires original integration collision interpolation measurement"]
fn original_gl_integration_mix_arithmetic_diagnostic() {
    let _=integration_intermediates();
}

pub(crate) fn integration_intermediates() -> [Vec<[f32;4]>;2] {
    let output=crate::native_gl_mips::with_current(|| {
        let api=Api::load()?;let mut objects=Objects::new(&api,128,1);
        let dt=(1.0f32/60.0)/50.0;
        let positions:Vec<[f32;4]>=(0..128).map(|i|[i as f32,10000.0,(i as f32-64.0)*0.03713,(i as f32-64.0)*0.2137]).collect();
        let forces:Vec<[f32;4]>=(0..128).map(|i|[0.0,-7.5367827+i as f32*0.01711,0.0,0.0]).collect();
        let masks=vec![[8u8,0,0,0];128];
        let pos=objects.texture(128,1,34836,6408,5126,positions.as_ptr().cast());
        let force=objects.texture(128,1,34836,6408,5126,forces.as_ptr().cast());
        let mask=objects.texture(128,1,36220,36249,5121,masks.as_ptr().cast());
        let program=objects.program(crate::ship_physics_shaders::INTEGRATE,&["out_pos_vel"])?;
        objects.target(&[pos])?;objects.source_state(&crate::ship_physics_stencil::states().exec);
        unsafe {(api.glUseProgram)(program);}
        objects.bind(program,"in_mask_struts",mask,0);objects.bind(program,"in_pos_vel",pos,1);objects.bind(program,"in_force",force,2);
        objects.scalar(program,"deltaT",dt);objects.scalar(program,"floorHeight",-400.0);objects.draw();
        let mut output=vec![[0.0f32;4];128];
        unsafe {(api.glBindTexture)(3553,pos);(api.glGetTexImage)(3553,0,6408,5126,output.as_mut_ptr().cast());}
        // A diagnostic copy exposes source intermediates. The retained shader
        // used above remains byte-identical; this copy is not the game oracle.
        let debug_source=crate::ship_physics_shaders::INTEGRATE.replace(
            "out_pos_vel = vec4(deltaP * deltaT + posVel.xy, mix(refl, vel, float(collision == 1)));",
            "out_pos_vel = vec4(vel, refl);");
        assert_ne!(debug_source,crate::ship_physics_shaders::INTEGRATE);
        let debug_program=objects.program(&debug_source,&["out_pos_vel"])?;
        let debug_pos=objects.texture(128,1,34836,6408,5126,positions.as_ptr().cast());
        objects.target(&[debug_pos])?;unsafe {(api.glUseProgram)(debug_program);}
        objects.bind(debug_program,"in_mask_struts",mask,0);objects.bind(debug_program,"in_pos_vel",debug_pos,1);objects.bind(debug_program,"in_force",force,2);
        objects.scalar(debug_program,"deltaT",dt);objects.scalar(debug_program,"floorHeight",-400.0);objects.draw();
        let mut intermediate=vec![[0.0f32;4];128];
        unsafe {(api.glBindTexture)(3553,debug_pos);(api.glGetTexImage)(3553,0,6408,5126,intermediate.as_mut_ptr().cast());}
        let mut measured_matches=[0usize;3];
        for i in 0..128 {for lane in 0..2 {
            let v=intermediate[i][lane];let r=intermediate[i][lane+2];
            let candidates=[v,r+(v-r),(v-r).mul_add(1.0,r)];
            for choice in 0..3 {if candidates[choice].to_bits()==output[i][lane+2].to_bits() {measured_matches[choice]+=1;}}
        }}
        println!("Original collision mix using diagnostic GPU intermediates: weighted/difference/fused-difference {measured_matches:?}");
        assert_eq!(measured_matches[2],256,"difference interpolation must preserve all measured source velocity lanes");
        let mut matches=[0usize;3];
        for i in 0..128 {
            let velocity=[forces[i][0].mul_add(dt,positions[i][2]),forces[i][1].mul_add(dt,positions[i][3])];
            let inverse=1.0/(velocity[0]*velocity[0]+velocity[1]*velocity[1]+1.0);
            let reflected=[velocity[0]*inverse,-velocity[1]*inverse];
            for lane in 0..2 {
                let candidates=[velocity[lane],reflected[lane]+(velocity[lane]-reflected[lane]),(velocity[lane]-reflected[lane]).mul_add(1.0,reflected[lane])];
                for choice in 0..3 {if candidates[choice].to_bits()==output[i][lane+2].to_bits() {matches[choice]+=1;}}
            }
        }
        println!("Original INTEGRATE 256 velocity lanes: weighted/difference/fused-difference collision mix matches {matches:?}");
        Ok::<_,String>([output,intermediate])
    }).unwrap();crate::native_gl_mips::release_current_thread();
    output
}

#[test]
#[ignore = "Requires original force feedback arithmetic measurement"]
fn original_gl_force_feedback_arithmetic_diagnostic() {
    let positions:Vec<_>=(0..128).map(|i|[i as f32,10000.0,0.0,0.0]).collect();
    let masks=vec![[8,0,0,0];128];
    let materials:Vec<_>=(0..128).map(|i|[10.0,1e10,1e10,1.9708751+i as f32*0.03137]).collect();
    let forces:Vec<_>=(0..128).map(|i|[0.0,-7.5367827+i as f32*0.01711,0.0,0.0]).collect();
    let settings=vec![[128.0,1.0,50.0,1.0/60.0],[9.81,0.0,0.0,1.0],
        [0.0,1.0,40.0,1.0],[0.0,-400.0,1.0/300.0,0.0],[0.0,0.0,1.0,0.915],
        [0.0;4],[0.0;4],[0.0;4]];
    let output=step(128,1,&positions,&masks,&materials,&forces,&settings).unwrap();
    let density=1025.0f32+(1.225f32-1025.0);
    let mut matches=[[0usize;2];2];
    for i in 0..128 {
        let inverse=1.0f32/materials[i][3];
        let before=[forces[i][1]+density*9.81,density.mul_add(9.81,forces[i][1])];
        for first in 0..2 {
            let values=[before[first]*inverse-9.81,before[first].mul_add(inverse,-9.81)];
            for last in 0..2 {if values[last].to_bits()==output.forces[i][1].to_bits() {matches[first][last]+=1;}}
        }
    }
    println!("Original FORCES 128 samples: buoyancy split/fused then final split/fused matches {matches:?}");
    assert_eq!(matches[1][1],128,"both force contractions must match original GPU arithmetic");
    crate::native_gl_mips::release_current_thread();
}

#[test]
#[ignore = "Requires original GPU material-mass interpolation measurement"]
fn original_gl_material_mass_interpolation_diagnostic() {
    crate::native_gl_mips::with_current(|| {
        let api=Api::load()?;let mut objects=Objects::new(&api,128,1);
        let amounts:Vec<f32>=(0..128).map(|i|i as f32/127.0).collect();
        let water_values:Vec<[f32;4]>=amounts.iter().map(|&q|[q,0.0,0.0,0.0]).collect();
        let masses=vec![[10.0f32,1e10,1e10,10.0];128];let masks=vec![[8u8,0,0,0];128];
        let water=objects.texture(128,1,34836,6408,5126,water_values.as_ptr().cast());
        let mass=objects.texture(128,1,34836,6408,5126,masses.as_ptr().cast());
        let mask=objects.texture(128,1,36220,36249,5121,masks.as_ptr().cast());
        let program=objects.program(crate::ship_physics_shaders::UPDATE_MASS,&["out_mass_strength"])?;
        objects.target(&[mass])?;objects.source_state(&crate::ship_physics_stencil::states().exec);
        unsafe {(api.glUseProgram)(program);}
        objects.bind(program,"in_mask_struts",mask,0);objects.bind(program,"in_water",water,1);objects.bind(program,"in_mass_strength",mass,2);
        objects.scalar(program,"u_thickness",0.915);objects.scalar(program,"u_waterweight",1.0);objects.draw();
        let mut output=vec![[0.0f32;4];128];
        unsafe {(api.glBindTexture)(3553,mass);(api.glGetTexImage)(3553,0,6408,5126,output.as_mut_ptr().cast());}
        let mix: [fn(f32,f32,f32)->f32;4]=[
            |a,b,t| a*(1.0-t)+b*t,
            |a,b,t| a+(b-a)*t,
            |a,b,t| (b-a).mul_add(t,a),
            |a,b,t| b.mul_add(t,a*(1.0-t)),
        ];
        let mut matches=[[0usize;4];4];
        for (&q,actual) in amounts.iter().zip(&output) {
            for inner in 0..4 {for outer in 0..4 {
                let density=mix[inner](1.225,1025.0,q);
                let value=mix[outer](10.0,density,0.915);
                if actual[3].to_bits()==value.to_bits() {matches[inner][outer]+=1;}
            }}
        }
        println!("Original UPDATE_MASS 128 inputs: inner/outer weighted, difference, fused-difference, fused-weighted matches {matches:?}");
        assert_eq!(matches[2][2],128,"source mass interpolation must match both fused difference operations");
        Ok::<_,String>(())
    }).unwrap();crate::native_gl_mips::release_current_thread();
}

#[test]
#[ignore = "Requires native original water-fill arithmetic measurement"]
fn original_gl_water_fill_arithmetic_diagnostic() {
    crate::native_gl_mips::with_current(|| {
        let api=Api::load()?;
        let mut objects=Objects::new(&api,128,1);
        let amounts:Vec<f32>=(0..128).map(|i|0.05+i as f32*0.03137).collect();
        let water_values:Vec<[f32;4]>=amounts.iter().map(|&q|[q,0.0,0.0,0.0]).collect();
        let position_values:Vec<[f32;4]>=(0..128).map(|i|[i as f32,10000.0,0.0,0.0]).collect();
        let mask_values=vec![[8u8,0,0,0];128];
        let water=objects.texture(128,1,34836,6408,5126,water_values.as_ptr().cast());
        let pos=objects.texture(128,1,34836,6408,5126,position_values.as_ptr().cast());
        let mask=objects.texture(128,1,36220,36249,5121,mask_values.as_ptr().cast());
        let fill=objects.program(crate::ship_physics_shaders::FILL_WATER,&["out_water"])?;
        objects.target(&[water])?;
        objects.source_state(&crate::ship_physics_stencil::states().exec);
        unsafe { (api.glUseProgram)(fill); }
        objects.bind(fill,"in_mask_struts",mask,0);objects.bind(fill,"in_pos_vel",pos,1);objects.bind(fill,"in_water",water,2);
        objects.scalar(fill,"u_flow",60.0);objects.scalar(fill,"u_inflow",1.0);
        unsafe { (api.glUniform2f)(objects.location(fill,"gravity"),0.0,-9.81); }
        objects.draw();
        let mut output=vec![[0.0f32;4];128];
        unsafe { (api.glBindTexture)(3553,water);(api.glGetTexImage)(3553,0,6408,5126,output.as_mut_ptr().cast()); }
        let mut matches=[0usize;4];
        for (&q,actual) in amounts.iter().zip(&output) {
            let root=(2.0f32*9.81*q).sqrt();
            let sixtieth=(1.0f32/60.0)/60.0;
            let candidates=[
                q-((root*60.0)*sixtieth).min(q),
                (-root*60.0).mul_add(sixtieth,q).max(0.0),
                (-root).mul_add(60.0*sixtieth,q).max(0.0),
                (q-root*(60.0*sixtieth)).max(0.0),
            ];
            for (count,value) in matches.iter_mut().zip(candidates) {if actual[0]==value {*count+=1;}}
        }
        println!("Original FILL_WATER 128 inputs: split/last-multiply-fused/prescaled-fused/prescaled-split matches {matches:?}");
        Ok::<_,String>(())
    }).unwrap();
    crate::native_gl_mips::release_current_thread();
}


/// Original water() with its two sine terms exposed for cross-backend diagnostics.
pub(crate) fn wave_intermediates(time:f32) -> Vec<[f32;4]> {
    crate::native_gl_mips::with_current(|| {
        let api=Api::load()?;let mut objects=Objects::new(&api,128,1);
        let position_values:Vec<[f32;4]>=(0..128).map(|i|[i as f32,-2.0,0.0,0.0]).collect();
        let pos=objects.texture(128,1,34836,6408,5126,position_values.as_ptr().cast());
        let output=objects.texture(128,1,34836,6408,5126,std::ptr::null());
        let original=crate::ship_physics_shaders::FILL_WATER;
        let diagnostic=original.replace("float h = -min(posVel.y - water(posVel.x), 0);", "float invWave = 3.141592 / waveSize.x; out_water=vec4(water(posVel.x),sin(posVel.x*invWave+time*.3),sin(invWave*3*posVel.x-time),invWave); return; float h=0;");
        assert_ne!(original,diagnostic);
        let program=objects.program(&diagnostic,&["out_water"])?;objects.target(&[output])?;
        unsafe {(api.glDisable)(2960);(api.glUseProgram)(program);(api.glUniform2f)(objects.location(program,"waveSize"),40.0,1.0);}
        objects.scalar(program,"time",time);objects.bind(program,"in_pos_vel",pos,0);objects.draw();
        let mut result=vec![[0.0;4];128];
        unsafe {(api.glBindTexture)(3553,output);(api.glGetTexImage)(3553,0,6408,5126,result.as_mut_ptr().cast());}
        Ok::<_,String>(result)
    }).unwrap()
}

/// Retained original fill plus a diagnostic copy exposing root/velocity/amount/difference.
pub(crate) fn water_fill_intermediates(inflow:f32,flow:f32,y:f32) -> [Vec<[f32;4]>;2] {
    crate::native_gl_mips::with_current(|| {
        let api=Api::load()?;let mut objects=Objects::new(&api,128,1);
        let water_values:Vec<[f32;4]>=(0..128).map(|i|[0.05+i as f32*0.03137,0.0,0.0,0.0]).collect();
        let position_values:Vec<[f32;4]>=(0..128).map(|i|[i as f32,y,0.0,0.0]).collect();
        let mask_values=vec![[8u8,0,0,0];128];
        let pos=objects.texture(128,1,34836,6408,5126,position_values.as_ptr().cast());
        let mask=objects.texture(128,1,36220,36249,5121,mask_values.as_ptr().cast());
        let original=crate::ship_physics_shaders::FILL_WATER;
        let diagnostic=original.replace("out_water.x += newvel;", "out_water.x += newvel; out_water = vec4(sqrt(2 * -gravity.y * abs(diff)), newvel, out_water.x, diff);");
        assert_ne!(original,diagnostic);
        let mut outputs=[Vec::new(),Vec::new()];
        for (slot,source) in [original,diagnostic.as_str()].into_iter().enumerate() {
            let water=objects.texture(128,1,34836,6408,5126,water_values.as_ptr().cast());
            let fill=objects.program(source,&["out_water"])?;objects.target(&[water])?;
            objects.source_state(&crate::ship_physics_stencil::states().exec);
            unsafe {(api.glUseProgram)(fill);}
            objects.bind(fill,"in_mask_struts",mask,0);objects.bind(fill,"in_pos_vel",pos,1);objects.bind(fill,"in_water",water,2);
            objects.scalar(fill,"u_flow",flow);objects.scalar(fill,"u_inflow",inflow);
            unsafe {(api.glUniform2f)(objects.location(fill,"gravity"),0.0,-9.81);(api.glUniform2f)(objects.location(fill,"waveSize"),40.0,1.0);}
            objects.draw();outputs[slot]=vec![[0.0;4];128];
            unsafe {(api.glBindTexture)(3553,water);(api.glGetTexImage)(3553,0,6408,5126,outputs[slot].as_mut_ptr().cast());}
        }
        Ok::<_,String>(outputs)
    }).unwrap()
}

#[test]
#[ignore = "Requires native OpenGL arithmetic diagnostic"]
fn original_gl_reciprocal_buoyancy_arithmetic_diagnostic() {
    crate::native_gl_mips::with_current(|| {
        let api = Api::load()?;
        let mut objects = Objects::new(&api, 1, 1);
        let input = [[1.225f32, 9.81, 0.0, 0.0]];
        let texture = objects.texture(1,1,34836,6408,5126,input.as_ptr().cast());
        let output = objects.texture(1,1,34836,6408,5126,std::ptr::null());
        let program = objects.program("#version 150 core\nuniform sampler2D values; out vec4 result; void main(){ vec2 v=texelFetch(values,ivec2(0),0).xy; float invMass=1/v.x; float density=mix(1025,1.225,float(v.y>=0)); result=vec4(invMass,density*v.y*invMass-v.y,density,v.y); }", &["result"])?;
        objects.target(&[output])?;
        unsafe { (api.glDisable)(2960); (api.glUseProgram)(program); }
        objects.bind(program,"values",texture,0);objects.draw();
        let mut result = [[0.0f32;4]];
        unsafe { (api.glBindTexture)(3553,output);(api.glGetTexImage)(3553,0,6408,5126,result.as_mut_ptr().cast()); }
        println!("Original GL reciprocal/arithmetic probe: {:?}; CPU reciprocal {} residual {}", result[0],1.0f32/1.225,1.225f32*9.81*(1.0/1.225)-9.81);
        let flow_input = [[2.0f32,0.8,4.429447,-0.2]];
        let texture = objects.texture(1,1,34836,6408,5126,flow_input.as_ptr().cast());
        let program = objects.program("#version 150 core\nuniform sampler2D values; out vec4 result; void main(){ vec4 v=texelFetch(values,ivec2(0),0); float funk=1+v.y*(v.x-1); result=vec4(funk,v.w+funk*v.z,0,0); }", &["result"])?;
        unsafe { (api.glUseProgram)(program); }
        objects.bind(program,"values",texture,0);objects.draw();
        unsafe { (api.glBindTexture)(3553,output);(api.glGetTexImage)(3553,0,6408,5126,result.as_mut_ptr().cast()); }
        let funk=1.0f32+0.8*(2.0-1.0);
        println!("Original GL flow multiply/add: {:?}; split {}, fused {}",result[0],funk*4.429447-0.2,funk.mul_add(4.429447,-0.2));
        assert_eq!(result[0][1],funk.mul_add(4.429447,-0.2));
        let mut integration=Objects::new(&api,128,1);
        let dt=(1.0f32/60.0)/50.0;
        let inputs:Vec<[f32;4]>=(0..128).map(|i|[-7.5367827+i as f32*0.013,dt,(i as f32-64.0)*0.00003137,(i as f32-64.0)*0.0000001711]).collect();
        let input=integration.texture(128,1,34836,6408,5126,inputs.as_ptr().cast());
        let output=integration.texture(128,1,34836,6408,5126,std::ptr::null());
        let program=integration.program("#version 150 core\nuniform sampler2D values; out vec4 result; void main(){ vec4 v=texelFetch(values,ivec2(gl_FragCoord.xy),0); float velocity=v.x*v.y+v.z; result=vec4(velocity,velocity*v.y+v.w,0,0); }", &["result"])?;
        integration.target(&[output])?;unsafe { (api.glDisable)(2960);(api.glUseProgram)(program); }
        integration.bind(program,"values",input,0);integration.draw();
        let mut results=vec![[0.0f32;4];128];
        unsafe { (api.glBindTexture)(3553,output);(api.glGetTexImage)(3553,0,6408,5126,results.as_mut_ptr().cast()); }
        let mut split_differences=0;
        for (value,result) in inputs.iter().zip(&results) {
            let velocity=value[0].mul_add(value[1],value[2]);
            let position=velocity.mul_add(value[1],value[3]);
            assert_eq!(result[0],velocity,"original velocity contraction");
            assert_eq!(result[1],position,"original position contraction");
            if result[0]!=(value[0]*value[1]+value[2]) || result[1]!=(result[0]*value[1]+value[3]) {split_differences+=1;}
        }
        println!("Original GL integration multiply/add: 128 inputs match fused CPU arithmetic; {split_differences} differ from separate operations");
        Ok::<_,String>(())
    }).unwrap();
    crate::native_gl_mips::release_current_thread();
}
/// One original force/integration/reciprocal-mask-repair cycle. Settings match
/// the production adapter's packed settings; caller disables water/mass changes
/// when comparing this isolated cycle with the full Bevy dispatcher.
pub(crate) fn step(
    width: i32,
    height: i32,
    positions: &[[f32; 4]],
    masks: &[[u32; 4]],
    materials: &[[f32; 4]],
    forces: &[[f32; 4]],
    settings: &[[f32; 4]],
) -> Result<Output, String> {
    step_with_water(
        width, height, positions, masks, materials, forces, settings, None,
    )
}
pub(crate) fn step_with_water(
    width: i32,
    height: i32,
    positions: &[[f32; 4]],
    masks: &[[u32; 4]],
    materials: &[[f32; 4]],
    forces: &[[f32; 4]],
    settings: &[[f32; 4]],
    water: Option<&[[f32; 4]]>,
) -> Result<Output, String> {
    run_plan(
        width, height, positions, masks, materials, forces, settings, water, 1, 1, 1,
    )
}
/// Retain native textures/programs/stencil across the complete source schedule.
/// Time and other settings remain fixed, matching controlled Bevy fixtures.
pub(crate) fn run_frames(
    width: i32,
    height: i32,
    positions: &[[f32; 4]],
    masks: &[[u32; 4]],
    materials: &[[f32; 4]],
    forces: &[[f32; 4]],
    settings: &[[f32; 4]],
    water: Option<&[[f32; 4]]>,
    frames: usize,
    water_steps: usize,
) -> Result<Output, String> {
    let iterations = settings.first().ok_or("Missing settings")?[2] as usize;
    if iterations == 0 || water_steps == 0 || water_steps > iterations || frames == 0 {
        return Err("Invalid source scheduling counts".into());
    }
    run_plan(
        width,
        height,
        positions,
        masks,
        materials,
        forces,
        settings,
        water,
        frames,
        iterations,
        iterations / water_steps,
    )
}
fn run_plan(
    width: i32,
    height: i32,
    positions: &[[f32; 4]],
    masks: &[[u32; 4]],
    materials: &[[f32; 4]],
    forces: &[[f32; 4]],
    settings: &[[f32; 4]],
    water: Option<&[[f32; 4]]>,
    frames: usize,
    iterations: usize,
    interval: usize,
) -> Result<Output, String> {
    let count = (width as usize)
        .checked_mul(height as usize)
        .ok_or("GL physics dimensions overflow")?;
    if width <= 0
        || height <= 0
        || [positions.len(), masks.len(), materials.len(), forces.len()]
            .iter()
            .any(|&len| len != count)
        || settings.len() < 5
        || water.is_some_and(|values| values.len() != count)
    {
        return Err("Invalid GL physics reference input".into());
    }
    crate::native_gl_mips::with_current(|| {
        use crate::ship_physics_shaders as source;
        let api = Api::load()?;
        static INFO: std::sync::Once = std::sync::Once::new();
        INFO.call_once(|| {
            let values: Vec<_> = [7936, 7937, 7938]
                .into_iter()
                .map(|name| {
                    let text = unsafe { (api.glGetString)(name) };
                    if text.is_null() {
                        "unavailable".to_owned()
                    } else {
                        unsafe { std::ffi::CStr::from_ptr(text) }
                            .to_string_lossy()
                            .into_owned()
                    }
                })
                .collect();
            println!(
                "Original GLSL driver: vendor={}, renderer={}, version={}",
                values[0], values[1], values[2]
            );
        });
        let mut objects = Objects::new(&api, width, height);
        let pos = objects.texture(width, height, 34836, 6408, 5126, positions.as_ptr().cast());
        let material = objects.texture(width, height, 34836, 6408, 5126, materials.as_ptr().cast());
        let force_values: Vec<[f32; 2]> = forces.iter().map(|force| [force[0], force[1]]).collect();
        let force = objects.texture(
            width,
            height,
            33328,
            33319,
            5126,
            force_values.as_ptr().cast(),
        );
        let mask_bytes: Vec<[u8; 4]> = masks.iter().map(|mask| mask.map(|x| x as u8)).collect();
        let mask = objects.texture(
            width,
            height,
            36220,
            36249,
            5121,
            mask_bytes.as_ptr().cast(),
        );
        // Source ProviderPass/TargetPass reference the same data holders for
        // reads and writes. Preserve that aliasing, including the original
        // framebuffer feedback semantics; don't invent a ping-pong snapshot.
        let pos_out = pos;
        let states = crate::ship_physics_stencil::states();
        let force_out = force;
        let mask_out = mask;
        let integrated = pos;
        let repaired = mask;
        let dynamic = objects.program(source::FILTER_DYNAMIC, &[])?;
        let occupied = objects.program(source::FILTER_OCCUPIED, &[])?;
        let compute = objects.program(source::FORCES, &["out_force", "out_mask"])?;
        let integrate = objects.program(source::INTEGRATE, &["out_pos_vel"])?;
        let repair = objects.program(source::REPAIR_MASK, &["out_mask"])?;
        objects.target(&[force_out, mask_out])?;
        unsafe {
            (api.glStencilMask)(255);
            (api.glClearStencil)(0);
            (api.glClear)(1024);
        }
        objects.source_filter(dynamic, mask, &states.set7);
        let water_passes = if let Some(values) = water {
            let water = objects.texture(width, height, 34836, 6408, 5126, values.as_ptr().cast());
            let zero = vec![[0.0f32; 4]; count];
            let scratch: Vec<_> = (0..4)
                .map(|_| objects.texture(width, height, 34836, 6408, 5126, zero.as_ptr().cast()))
                .collect();
            let fill = objects.program(source::FILL_WATER, &["out_water"])?;
            let flow = objects.program(
                source::FLOW_WATER,
                &[
                    "out_water_out_1",
                    "out_water_out_2",
                    "out_water_vel_1",
                    "out_water_vel_2",
                ],
            )?;
            let transport = objects.program(source::TRANSPORT_WATER, &["out_water"])?;
            let mass = objects.program(source::UPDATE_MASS, &["out_mass_strength"])?;
            let permeable = objects.program(source::FILTER_PERMEABLE, &[])?;
            Some((water, scratch, fill, flow, transport, mass, permeable))
        } else {
            None
        };
        // Retain the source repair target separately from shared dynamics/water.
        let mut repair_stencil = 0;
        unsafe {
            (api.glGenRenderbuffers)(1, &mut repair_stencil);
            (api.glBindRenderbuffer)(36161, repair_stencil);
            (api.glRenderbufferStorage)(36161, 35056, width, height);
            (api.glFramebufferRenderbuffer)(36160, 33306, 36161, repair_stencil);
            (api.glStencilMask)(255);
            (api.glClearStencil)(0);
            (api.glClear)(1024);
        }
        objects.extra_stencils.push(repair_stencil);
        objects.target(&[repaired])?;
        objects.source_filter(occupied, mask_out, &states.set7);
        let mut physics_steps = 0;
        let mut water_updates = 0;
        for _frame in 0..frames {
            for iteration in 0..iterations {
                unsafe {
                    (api.glFramebufferRenderbuffer)(36160, 33306, 36161, objects.stencil);
                }
                objects.target(&[force_out, mask_out])?;
                objects.source_state(&states.execIf7);
                unsafe {
                    (api.glUseProgram)(compute);
                }
                for (unit, (name, texture)) in [
                    ("in_pos_vel", pos),
                    ("in_force", force),
                    ("in_mask_struts", mask),
                    ("in_mass_strength", material),
                ]
                .into_iter()
                .enumerate()
                {
                    objects.bind(compute, name, texture, unit as u32);
                }
                let dt = settings[0][3] / settings[0][2];
                let fps = settings[0][2] / settings[0][3];
                for (name, value) in [
                    ("floorHeight", settings[3][1]),
                    ("time", settings[3][0]),
                    ("deltaT", dt),
                    ("fps", fps),
                    ("u_rigidity", settings[1][1]),
                    ("u_dampening", settings[1][2]),
                    ("u_strength", settings[1][3]),
                    ("u_drag", settings[2][0]),
                    ("u_buoyancy", settings[2][1]),
                ] {
                    objects.scalar(compute, name, value);
                }
                unsafe {
                    (api.glUniform1i)(objects.location(compute, "iter"), settings[0][2] as i32);
                    (api.glUniform2f)(objects.location(compute, "gravity"), 0., -settings[1][0]);
                    (api.glUniform2f)(
                        objects.location(compute, "waveSize"),
                        settings[2][2],
                        settings[2][3],
                    );
                }
                objects.draw();
                objects.target(&[integrated])?;
                unsafe {
                    (api.glUseProgram)(integrate);
                }
                objects.bind(integrate, "in_pos_vel", pos_out, 0);
                objects.bind(integrate, "in_force", force_out, 1);
                objects.bind(integrate, "in_mask_struts", mask_out, 2);
                objects.scalar(integrate, "deltaT", dt);
                objects.scalar(integrate, "floorHeight", settings[3][1]);
                objects.draw();
                physics_steps += 1;
                if iteration % interval == interval - 1 {
                    if let Some((water, scratch, fill, flow, transport, mass, permeable)) =
                        &water_passes
                    {
                        let (water, fill, flow, transport, mass, permeable) =
                            (*water, *fill, *flow, *transport, *mass, *permeable);
                        objects.target(&[water])?;
                        objects.source_filter(permeable, mask_out, &states.set1If7);
                        objects.source_state(&states.execIf7);
                        unsafe {
                            (api.glUseProgram)(fill);
                        }
                        for (unit, (name, texture)) in [
                            ("in_mask_struts", mask_out),
                            ("in_pos_vel", integrated),
                            ("in_water", water),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            objects.bind(fill, name, texture, unit as u32);
                        }
                        for (name, value) in [
                            ("time", settings[3][0]),
                            ("deltaT", settings[3][2]),
                            ("u_buoyancy", settings[2][1]),
                            ("u_inflow", settings[3][3]),
                            ("u_flow", settings[4][0]),
                        ] {
                            objects.scalar(fill, name, value);
                        }
                        // Exact source case mismatch: deltaT is absent; DeltaT stays 1.
                        unsafe {
                            (api.glUniform2f)(
                                objects.location(fill, "gravity"),
                                0.,
                                -settings[1][0],
                            );
                            (api.glUniform2f)(
                                objects.location(fill, "waveSize"),
                                settings[2][2],
                                settings[2][3],
                            );
                        }
                        objects.draw();
                        objects.target(&scratch)?;
                        objects.source_state(&states.execIf1);
                        unsafe {
                            (api.glUseProgram)(flow);
                        }
                        for (unit, (name, texture)) in [
                            ("in_mask_struts", mask_out),
                            ("in_pos_vel", integrated),
                            ("in_water", water),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            objects.bind(flow, name, texture, unit as u32);
                        }
                        for (name, value) in [
                            ("deltaT", settings[3][2]),
                            ("u_funk", settings[4][1]),
                            ("u_flow", settings[4][0]),
                        ] {
                            objects.scalar(flow, name, value);
                        }
                        unsafe {
                            (api.glUniform2f)(
                                objects.location(flow, "gravity"),
                                0.,
                                -settings[1][0],
                            );
                        }
                        objects.draw();
                        objects.target(&[water])?;
                        unsafe {
                            (api.glUseProgram)(transport);
                        }
                        for (unit, (name, texture)) in [
                            ("in_mask_struts", mask_out),
                            ("in_pos_vel", integrated),
                            ("in_water", water),
                            ("in_water_out_1", scratch[0]),
                            ("in_water_out_2", scratch[1]),
                            ("in_water_vel_1", scratch[2]),
                            ("in_water_vel_2", scratch[3]),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            objects.bind(transport, name, texture, unit as u32);
                        }
                        objects.draw();
                        objects.target(&[material])?;
                        objects.source_state(&states.execIf7);
                        unsafe {
                            (api.glUseProgram)(mass);
                        }
                        for (unit, (name, texture)) in [
                            ("in_water", water),
                            ("in_mass_strength", material),
                            ("in_mask_struts", mask_out),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            objects.bind(mass, name, texture, unit as u32);
                        }
                        objects.scalar(mass, "u_thickness", settings[4][3]);
                        objects.scalar(mass, "u_waterweight", settings[4][2]);
                        objects.draw();
                        water_updates += 1;
                    }
                    unsafe {
                        (api.glFramebufferRenderbuffer)(36160, 33306, 36161, repair_stencil);
                    }
                    objects.target(&[repaired])?;
                    objects.source_state(&states.exec);
                    unsafe {
                        (api.glUseProgram)(repair);
                    }
                    objects.bind(repair, "in_mask_struts", mask_out, 0);
                    objects.draw();
                }
            }
        }
        let water_texture = water_passes.as_ref().map(|passes| passes.0);
        let mut positions = vec![[0f32; 4]; count];
        let mut bytes = vec![[0u8; 4]; count];
        let mut water_values = vec![[0f32; 4]; count];
        let mut material_values = vec![[0f32; 4]; count];
        let mut force_values = vec![[0f32;4];count];
        let mut scratch_values: [Vec<[f32;4]>;4] = std::array::from_fn(|_|vec![[0.0;4];count]);
        unsafe {
            (api.glBindTexture)(3553, integrated);
            (api.glGetTexImage)(3553, 0, 6408, 5126, positions.as_mut_ptr().cast());
            (api.glBindTexture)(3553, repaired);
            (api.glGetTexImage)(3553, 0, 36249, 5121, bytes.as_mut_ptr().cast());
            if let Some(water) = water_texture {
                (api.glBindTexture)(3553, water);
                (api.glGetTexImage)(3553, 0, 6408, 5126, water_values.as_mut_ptr().cast());
            }
            (api.glBindTexture)(3553, material);
            (api.glGetTexImage)(3553, 0, 6408, 5126, material_values.as_mut_ptr().cast());
            (api.glBindTexture)(3553,force_out);
            (api.glGetTexImage)(3553,0,6408,5126,force_values.as_mut_ptr().cast());
            if let Some(passes)=&water_passes {
                for (texture,values) in passes.1.iter().zip(&mut scratch_values) {
                    (api.glBindTexture)(3553,*texture);
                    (api.glGetTexImage)(3553,0,6408,5126,values.as_mut_ptr().cast());
                }
            }
            let error = (api.glGetError)();
            if error != 0 {
                return Err(format!("Original physics GL error: {error:#x}"));
            }
        }
        Ok(Output {
            positions,
            masks: bytes.into_iter().map(|mask| mask.map(u32::from)).collect(),
            water: water_values,
            materials: material_values,
            physics_steps,
            water_updates,
            forces: force_values,
            water_scratch: scratch_values,
        })
    })
}
