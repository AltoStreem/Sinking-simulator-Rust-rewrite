//! Real Windows OpenGL operations for the translated engine objects.
//! Construct and use inside native_gl_mips::with_current on the owning thread.
//! This is a native backend foundation, not a replacement for Bevy presentation.
#![allow(dead_code)]
use std::{collections::HashMap, ffi::{CStr,CString,c_void}, thread::{self,ThreadId}};
#[link(name="opengl32")]
unsafe extern "system" { fn wglGetCurrentContext()->*mut c_void; }
#[link(name="kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name:*const u16)->*mut c_void;
    fn GetProcAddress(module:*mut c_void,name:*const u8)->*const c_void;
}
macro_rules! gl {
    ($s:expr,$name:literal,$ty:ty,$($arg:expr),* $(,)?) => {{
        $s.assert_current();
        // Function signatures are the Windows OpenGL ABI; pointers are validated
        // at construction and may only be invoked in their original context.
        let function:$ty=unsafe {std::mem::transmute($s.functions[$name])};
        unsafe {function($($arg),*)}
    }};
}
#[derive(Clone)]
pub(crate) struct NativeGlBackend {
    thread:ThreadId,
    context:usize,
    functions:HashMap<&'static str,usize>,
}
impl NativeGlBackend {
    pub(crate) fn new()->Result<Self,String> {
        let context=unsafe {wglGetCurrentContext()} as usize;
        if context==0 {return Err("Native GL backend requires a current context".into());}
        let mut functions=HashMap::new();
        let library:Vec<u16>="opengl32.dll\0".encode_utf16().collect();
        let module=unsafe {GetModuleHandleW(library.as_ptr())};
        for name in [
            "glCreateShader","glShaderSource","glCompileShader","glGetShaderiv","glGetShaderInfoLog",
            "glAttachShader","glDetachShader","glDeleteShader","glCreateProgram","glBindAttribLocation",
            "glBindFragDataLocation","glLinkProgram","glGetProgramiv","glGetProgramInfoLog","glUseProgram",
            "glGetUniformLocation","glGetAttribLocation","glGetActiveAttrib","glUniform1fv","glUniform2fv",
            "glUniform3fv","glUniform4fv","glUniform1iv","glUniform2iv","glUniform3iv","glUniform4iv",
            "glUniformMatrix4fv","glValidateProgram","glDeleteProgram","glGenBuffers","glBindBuffer",
            "glBufferData","glDeleteBuffers","glGenVertexArrays","glBindVertexArray","glVertexAttribPointer",
            "glDeleteVertexArrays","glEnableVertexAttribArray","glDisableVertexAttribArray","glDrawElements",
            "glGenTextures","glBindTexture","glActiveTexture","glTexParameteri","glTexImage1D","glTexImage2D",
            "glTexImage3D","glGetTexImage","glGetTexLevelParameteriv","glFramebufferTexture","glGetError",
            "glGenerateMipmap","glDeleteTextures","glGenRenderbuffers","glBindRenderbuffer","glRenderbufferStorage",
            "glFramebufferRenderbuffer","glDeleteRenderbuffers","glGenFramebuffers","glBindFramebuffer",
            "glDrawBuffers","glViewport","glDeleteFramebuffers","glGetIntegerv","glStencilMask","glStencilFunc",
            "glStencilOp","glEnable","glDisable","glColorMask","glClear","glBlendEquation","glBlendFunc",
            "glGetString","glClearColor","glReadPixels","glFinish","glCheckFramebufferStatus","glGetUniformfv","glIsProgram",
        ] {
            let cname=CString::new(name).unwrap();
            let mut address=crate::native_gl_mips::procedure(&cname) as usize;
            if address<=3 || address==usize::MAX {
                address=unsafe {GetProcAddress(module,cname.as_ptr().cast())} as usize;
            }
            if address<=3 || address==usize::MAX {return Err(format!("OpenGL procedure {name} is unavailable"));}
            functions.insert(name,address);
        }
        Ok(Self {thread:thread::current().id(),context,functions})
    }
    fn assert_current(&self) {
        assert_eq!(thread::current().id(),self.thread,"Native GL backend used off its engine thread");
        assert_eq!(unsafe {wglGetCurrentContext()} as usize,self.context,"Native GL backend used in a foreign context");
    }
    pub(crate) fn integer(&self,name:u32)->i32 {
        let mut value=0;gl!(self,"glGetIntegerv",unsafe extern "system" fn(u32,*mut i32),name,&mut value);value
    }
    pub(crate) fn error(&self)->u32 {gl!(self,"glGetError",unsafe extern "system" fn()->u32,)}
    pub(crate) fn driver_string(&self,name:u32)->String {
        let value=gl!(self,"glGetString",unsafe extern "system" fn(u32)->*const u8,name);
        if value.is_null(){return String::new();} unsafe {CStr::from_ptr(value.cast())}.to_string_lossy().into_owned()
    }
    pub(crate) fn clear_color(&self,color:[f32;4]) {
        gl!(self,"glClearColor",unsafe extern "system" fn(f32,f32,f32,f32),color[0],color[1],color[2],color[3]);
    }
    pub(crate) fn finish(&self){gl!(self,"glFinish",unsafe extern "system" fn(),);}
    pub(crate) fn framebuffer_status(&self)->u32 {
        gl!(self,"glCheckFramebufferStatus",unsafe extern "system" fn(u32)->u32,36160)
    }
    pub(crate) fn is_program(&self,id:i32)->bool {gl!(self,"glIsProgram",unsafe extern "system" fn(u32)->u8,id as u32)!=0}
    /// # Safety
    /// Location must identify one mat4 uniform, never an array or larger value.
    pub(crate) unsafe fn matrix_uniform_value(&self,id:i32,location:i32)->[f32;16] {
        let mut value=[0.;16];gl!(self,"glGetUniformfv",unsafe extern "system" fn(u32,i32,*mut f32),id as u32,location,value.as_mut_ptr());value
    }
    /// Preserve native bottom-to-top row order; presentation converts separately.
    pub(crate) fn read_rgba8(&self,size:[i32;2])->Vec<u8> {
        assert_eq!(self.integer(35053),0,"Client framebuffer readback requires no pack PBO");
        let bytes=self.pixel_capacity(size[0],size[1],1,6408,5121,true);
        let mut result=vec![0;bytes];
        gl!(self,"glReadPixels",unsafe extern "system" fn(i32,i32,i32,i32,u32,u32,*mut c_void),
            0,0,size[0],size[1],6408,5121,result.as_mut_ptr().cast());result
    }
    fn log(&self,id:i32,shader:bool)->String {
        let mut length=0;
        if shader {gl!(self,"glGetShaderiv",unsafe extern "system" fn(u32,u32,*mut i32),id as u32,35716,&mut length);}
        else {gl!(self,"glGetProgramiv",unsafe extern "system" fn(u32,u32,*mut i32),id as u32,35716,&mut length);}
        if length<=0{return String::new();}
        let mut bytes=vec![0;length as usize];let mut written=0;
        if shader {gl!(self,"glGetShaderInfoLog",unsafe extern "system" fn(u32,i32,*mut i32,*mut u8),id as u32,length,&mut written,bytes.as_mut_ptr());}
        else {gl!(self,"glGetProgramInfoLog",unsafe extern "system" fn(u32,i32,*mut i32,*mut u8),id as u32,length,&mut written,bytes.as_mut_ptr());}
        bytes.truncate((written.max(0) as usize).min(bytes.len()));String::from_utf8_lossy(&bytes).into_owned()
    }
    fn check(&self,label:&str) {
        let error=self.error();
        // Bundled gln.GlnKt.checkError$default sets throws=true (mask=2).
        // Consume one native error and propagate; do not clear/repair state.
        if error!=0 {
            let message=match error {1280=>"GL_INVALID_ENUM",1281=>"GL_INVALID_VALUE",
                1282=>"GL_INVALID_OPERATION",1286=>"GL_INVALID_FRAMEBUFFER_OPERATION",
                1285=>"GL_OUT_OF_MEMORY",1284=>"GL_STACK_UNDERFLOW",1283=>"GL_STACK_OVERFLOW",
                _=>panic!("java.lang.IllegalStateException")};
            panic!("OpenGL Error ({message}) at {label}");
        }
    }
    // Bounds checks keep FFI writes/reads inside Rust allocations while retaining
    // the current native pixel-store stride, padding and skips.
    fn pixel_capacity(&self,width:i32,height:i32,depth:i32,format:i32,kind:i32,pack:bool)->usize {
        assert!(width>=0 && height>=0 && depth>=0,"Negative native texture extent");
        if width==0 || height==0 || depth==0{return 0;}
        let components=match format {6401|6402|6403|6404|6405|6406|6409|36244|36245|36246=>1,
            33319|33320|6410=>2,6407|32992|36248|36250=>3,6408|32993|36249|36251=>4,
            34041=>1,_=>panic!("Unsupported native pixel format {format}")};
        let element=match kind {5120|5121=>components,5122|5123|5131=>components*2,
            5124|5125|5126=>components*4,5130=>components*8,
            32818|33634=>1,32819|32820|33635|33636|33637|33638=>2,
            32821|32822|33639|33640|34042|35899|35902=>4,36269=>8,
            _=>panic!("Unsupported native pixel type {kind}")} as usize;
        let alignment=self.integer(if pack{3333}else{3317}) as usize;
        assert!([1,2,4,8].contains(&alignment));
        let row=self.integer(if pack{3330}else{3314}).max(0) as usize;
        let image=self.integer(if pack{32876}else{32878}).max(0) as usize;
        let skip_rows=self.integer(if pack{3331}else{3315}).max(0) as usize;
        let skip_pixels=self.integer(if pack{3332}else{3316}).max(0) as usize;
        let skip_images=self.integer(if pack{32875}else{32877}).max(0) as usize;
        let checked=|a:usize,b:usize|a.checked_mul(b).expect("Native pixel capacity overflow");
        let row_bytes=checked(if row==0{width as usize}else{row},element);
        let stride=row_bytes.checked_add(alignment-1).unwrap()/alignment*alignment;
        let image_stride=checked(stride,if image==0{height as usize}else{image});
        checked(skip_images+depth as usize-1,image_stride)
            .checked_add(checked(skip_rows+height as usize-1,stride)).unwrap()
            .checked_add(checked(skip_pixels+width as usize,element)).unwrap()
    }
}
impl crate::shader::ShaderBackend for NativeGlBackend {
    fn create_shader(&mut self,kind:i32)->i32 {gl!(self,"glCreateShader",unsafe extern "system" fn(u32)->u32,kind as u32) as i32}
    fn shader_source(&mut self,id:i32,source:&str) {
        let pointer=source.as_ptr();let length=i32::try_from(source.len()).expect("GL shader source too long");
        gl!(self,"glShaderSource",unsafe extern "system" fn(u32,i32,*const *const u8,*const i32),id as u32,1,&pointer,&length);
    }
    fn compile_shader(&mut self,id:i32){gl!(self,"glCompileShader",unsafe extern "system" fn(u32),id as u32);}
    fn shader_info_log(&mut self,id:i32)->String {self.log(id,true)}
    fn shader_compiled(&mut self,id:i32)->bool {
        let mut value=0;gl!(self,"glGetShaderiv",unsafe extern "system" fn(u32,u32,*mut i32),id as u32,35713,&mut value);value!=0
    }
    fn attach_shader(&mut self,program:i32,shader:i32){gl!(self,"glAttachShader",unsafe extern "system" fn(u32,u32),program as u32,shader as u32);}
    fn detach_shader(&mut self,program:i32,shader:i32){gl!(self,"glDetachShader",unsafe extern "system" fn(u32,u32),program as u32,shader as u32);}
    fn delete_shader(&mut self,id:i32){gl!(self,"glDeleteShader",unsafe extern "system" fn(u32),id as u32);}
}
impl crate::shader_program::ProgramBackend for NativeGlBackend {
    fn create_program(&mut self)->i32 {gl!(self,"glCreateProgram",unsafe extern "system" fn()->u32,) as i32}
    fn bind_attribute(&mut self,id:i32,index:i32,name:&str){let name=CString::new(name).expect("NUL in GL attribute name");gl!(self,"glBindAttribLocation",unsafe extern "system" fn(u32,u32,*const u8),id as u32,index as u32,name.as_ptr().cast());}
    fn bind_fragment_output(&mut self,id:i32,index:i32,name:&str){let name=CString::new(name).expect("NUL in GL output name");gl!(self,"glBindFragDataLocation",unsafe extern "system" fn(u32,u32,*const u8),id as u32,index as u32,name.as_ptr().cast());}
    fn link_program(&mut self,id:i32){gl!(self,"glLinkProgram",unsafe extern "system" fn(u32),id as u32);}
    fn program_info_log(&mut self,id:i32)->String{self.log(id,false)}
    fn use_program(&mut self,id:i32){gl!(self,"glUseProgram",unsafe extern "system" fn(u32),id as u32);}
    fn uniform_location(&mut self,id:i32,name:&str)->i32 {let name=CString::new(name).unwrap();gl!(self,"glGetUniformLocation",unsafe extern "system" fn(u32,*const u8)->i32,id as u32,name.as_ptr().cast())}
    fn attribute_location(&mut self,id:i32,name:&str)->i32 {let name=CString::new(name).unwrap();gl!(self,"glGetAttribLocation",unsafe extern "system" fn(u32,*const u8)->i32,id as u32,name.as_ptr().cast())}
    fn active_attribute_count(&mut self,id:i32)->i32 {let mut value=0;gl!(self,"glGetProgramiv",unsafe extern "system" fn(u32,u32,*mut i32),id as u32,35721,&mut value);value}
    fn active_attribute(&mut self,id:i32,index:i32,max_bytes:usize)->String {
        let length=i32::try_from(max_bytes).unwrap();let mut bytes=vec![0;max_bytes];let(mut written,mut size,mut kind)=(0,0,0);
        gl!(self,"glGetActiveAttrib",unsafe extern "system" fn(u32,u32,i32,*mut i32,*mut i32,*mut u32,*mut u8),id as u32,index as u32,length,&mut written,&mut size,&mut kind,bytes.as_mut_ptr());
        bytes.truncate((written.max(0) as usize).min(max_bytes));String::from_utf8_lossy(&bytes).into_owned()
    }
    fn float_uniform(&mut self,location:i32,values:&[f32]) {
        match values.len(){
            1=>gl!(self,"glUniform1fv",unsafe extern "system" fn(i32,i32,*const f32),location,1,values.as_ptr()),
            2=>gl!(self,"glUniform2fv",unsafe extern "system" fn(i32,i32,*const f32),location,1,values.as_ptr()),
            3=>gl!(self,"glUniform3fv",unsafe extern "system" fn(i32,i32,*const f32),location,1,values.as_ptr()),
            4=>gl!(self,"glUniform4fv",unsafe extern "system" fn(i32,i32,*const f32),location,1,values.as_ptr()),_=>panic!("GL uniform vector size")}
    }
    fn int_uniform(&mut self,location:i32,values:&[i32]) {
        match values.len(){
            1=>gl!(self,"glUniform1iv",unsafe extern "system" fn(i32,i32,*const i32),location,1,values.as_ptr()),
            2=>gl!(self,"glUniform2iv",unsafe extern "system" fn(i32,i32,*const i32),location,1,values.as_ptr()),
            3=>gl!(self,"glUniform3iv",unsafe extern "system" fn(i32,i32,*const i32),location,1,values.as_ptr()),
            4=>gl!(self,"glUniform4iv",unsafe extern "system" fn(i32,i32,*const i32),location,1,values.as_ptr()),_=>panic!("GL uniform vector size")}
    }
    fn matrix_uniform(&mut self,location:i32,transposed:bool,values:&[f32;16]){gl!(self,"glUniformMatrix4fv",unsafe extern "system" fn(i32,i32,u8,*const f32),location,1,u8::from(transposed),values.as_ptr());}
    fn bind_vertex_array(&mut self,id:i32){gl!(self,"glBindVertexArray",unsafe extern "system" fn(u32),id as u32);}
    fn validate_program(&mut self,id:i32){gl!(self,"glValidateProgram",unsafe extern "system" fn(u32),id as u32);}
    fn delete_program(&mut self,id:i32){gl!(self,"glDeleteProgram",unsafe extern "system" fn(u32),id as u32);}
}
impl crate::vbo::BufferBackend for NativeGlBackend {
    fn create_buffer(&mut self)->i32 {let mut id=0;gl!(self,"glGenBuffers",unsafe extern "system" fn(i32,*mut u32),1,&mut id);id as i32}
    fn bind_buffer(&mut self,target:i32,id:i32){gl!(self,"glBindBuffer",unsafe extern "system" fn(u32,u32),target as u32,id as u32);}
    fn upload(&mut self,target:i32,data:crate::vbo::BufferData<'_>,mode:i32){
        use crate::vbo::BufferData::*;
        let (bytes,pointer)=match data {Byte(v)=>(std::mem::size_of_val(v),v.as_ptr().cast::<c_void>()),Short(v)=>(std::mem::size_of_val(v),v.as_ptr().cast()),Int(v)=>(std::mem::size_of_val(v),v.as_ptr().cast()),Float(v)=>(std::mem::size_of_val(v),v.as_ptr().cast()),Double(v)=>(std::mem::size_of_val(v),v.as_ptr().cast())};
        gl!(self,"glBufferData",unsafe extern "system" fn(u32,isize,*const c_void,u32),target as u32,isize::try_from(bytes).unwrap(),pointer,mode as u32);
    }
    fn delete_buffer(&mut self,id:i32){gl!(self,"glDeleteBuffers",unsafe extern "system" fn(i32,*const u32),1,&(id as u32));}
}
impl crate::vao::VertexArrayBackend for NativeGlBackend {
    fn create_vertex_array(&mut self)->i32 {let mut id=0;gl!(self,"glGenVertexArrays",unsafe extern "system" fn(i32,*mut u32),1,&mut id);id as i32}
    fn bind_vertex_array(&mut self,id:i32){gl!(self,"glBindVertexArray",unsafe extern "system" fn(u32),id as u32);}
    fn vertex_attribute_pointer(&mut self,index:i32,components:i32,kind:i32,normalized:bool,stride:i32,offset:u64){gl!(self,"glVertexAttribPointer",unsafe extern "system" fn(u32,i32,u32,u8,i32,*const c_void),index as u32,components,kind as u32,u8::from(normalized),stride,usize::try_from(offset).unwrap() as *const c_void);}
    fn delete_vertex_array(&mut self,id:i32){gl!(self,"glDeleteVertexArrays",unsafe extern "system" fn(i32,*const u32),1,&(id as u32));}
}
impl crate::model::ModelBackend for NativeGlBackend {
    fn enable_attribute(&mut self,index:i32){gl!(self,"glEnableVertexAttribArray",unsafe extern "system" fn(u32),index as u32);}
    fn disable_attribute(&mut self,index:i32){gl!(self,"glDisableVertexAttribArray",unsafe extern "system" fn(u32),index as u32);}
    fn draw_elements(&mut self,style:i32,count:usize,kind:i32,offset:u64){gl!(self,"glDrawElements",unsafe extern "system" fn(u32,i32,u32,*const c_void),style as u32,i32::try_from(count).unwrap(),kind as u32,usize::try_from(offset).unwrap() as *const c_void);}
}
impl crate::texture::TextureBackend for NativeGlBackend {
    fn create_texture(&mut self)->i32 {let mut id=0;gl!(self,"glGenTextures",unsafe extern "system" fn(i32,*mut u32),1,&mut id);id as i32}
    fn bind_texture(&mut self,target:i32,id:i32){gl!(self,"glBindTexture",unsafe extern "system" fn(u32,u32),target as u32,id as u32);}
    fn active_texture(&mut self,unit:i32){gl!(self,"glActiveTexture",unsafe extern "system" fn(u32),unit as u32);}
    fn parameter(&mut self,target:i32,parameter:i32,value:i32){gl!(self,"glTexParameteri",unsafe extern "system" fn(u32,u32,i32),target as u32,parameter as u32,value);}
    fn image_1d(&mut self,target:i32,internal:i32,width:i32,format:i32,kind:i32,data:&[u8]){
        assert!(data.len()>=self.pixel_capacity(width,1,1,format,kind,false));
        assert_eq!(self.integer(35055),0,"Client texture data requires no unpack PBO");
        gl!(self,"glTexImage1D",unsafe extern "system" fn(u32,i32,i32,i32,i32,u32,u32,*const c_void),target as u32,0,internal,width,0,format as u32,kind as u32,data.as_ptr().cast());
    }
    fn image_2d(&mut self,target:i32,internal:i32,size:[i32;2],format:i32,kind:i32,data:Option<&[u8]>){
        if let Some(data)=data {assert!(data.len()>=self.pixel_capacity(size[0],size[1],1,format,kind,false),"Native 2D pixel upload exceeds allocation");}
        assert_eq!(self.integer(35055),0,"Client texture data requires no unpack PBO");
        gl!(self,"glTexImage2D",unsafe extern "system" fn(u32,i32,i32,i32,i32,i32,u32,u32,*const c_void),target as u32,0,internal,size[0],size[1],0,format as u32,kind as u32,data.map_or(std::ptr::null(),|data|data.as_ptr().cast()));
    }
    fn image_3d(&mut self,target:i32,internal:i32,size:[i32;3],format:i32,kind:i32,data:Option<&[u8]>){
        if let Some(data)=data {assert!(data.len()>=self.pixel_capacity(size[0],size[1],size[2],format,kind,false));}
        assert_eq!(self.integer(35055),0,"Client texture data requires no unpack PBO");
        gl!(self,"glTexImage3D",unsafe extern "system" fn(u32,i32,i32,i32,i32,i32,i32,u32,u32,*const c_void),target as u32,0,internal,size[0],size[1],size[2],0,format as u32,kind as u32,data.map_or(std::ptr::null(),|data|data.as_ptr().cast()));
    }
    fn get_image_floats(&mut self,target:i32,level:i32,format:i32,kind:i32,output:&mut [f32]){
        let dimension=|name:u32|{let mut value=0;gl!(self,"glGetTexLevelParameteriv",unsafe extern "system" fn(u32,i32,u32,*mut i32),target as u32,level,name,&mut value);value};
        let width=dimension(4096);let height=if target==3552{1}else{dimension(4097)};
        let depth=if target==32879{dimension(32881)}else{1};
        assert!(std::mem::size_of_val(output)>=self.pixel_capacity(width,height,depth,format,kind,true),"Native texture readback exceeds allocation");
        assert_eq!(self.integer(35053),0,"Client texture readback requires no pack PBO");
        gl!(self,"glGetTexImage",unsafe extern "system" fn(u32,i32,u32,u32,*mut c_void),target as u32,level,format as u32,kind as u32,output.as_mut_ptr().cast());
    }
    fn framebuffer_texture(&mut self,target:i32,attachment:i32,id:i32,level:i32){gl!(self,"glFramebufferTexture",unsafe extern "system" fn(u32,u32,u32,i32),target as u32,attachment as u32,id as u32,level);}
    fn check_error(&mut self,label:&str){self.check(label);}
    fn generate_mipmaps(&mut self,target:i32){gl!(self,"glGenerateMipmap",unsafe extern "system" fn(u32),target as u32);}
    fn delete_texture(&mut self,id:i32){gl!(self,"glDeleteTextures",unsafe extern "system" fn(i32,*const u32),1,&(id as u32));}
}
impl crate::render_buffer::RenderBufferBackend for NativeGlBackend {
    fn create_renderbuffer(&mut self)->i32 {let mut id=0;gl!(self,"glGenRenderbuffers",unsafe extern "system" fn(i32,*mut u32),1,&mut id);id as i32}
    fn bind_renderbuffer(&mut self,target:i32,id:i32){gl!(self,"glBindRenderbuffer",unsafe extern "system" fn(u32,u32),target as u32,id as u32);}
    fn storage(&mut self,target:i32,format:i32,width:i32,height:i32){gl!(self,"glRenderbufferStorage",unsafe extern "system" fn(u32,u32,i32,i32),target as u32,format as u32,width,height);}
    fn attach_renderbuffer(&mut self,target:i32,attachment:i32,rb_target:i32,id:i32){gl!(self,"glFramebufferRenderbuffer",unsafe extern "system" fn(u32,u32,u32,u32),target as u32,attachment as u32,rb_target as u32,id as u32);}
    fn delete_renderbuffer(&mut self,id:i32){gl!(self,"glDeleteRenderbuffers",unsafe extern "system" fn(i32,*const u32),1,&(id as u32));}
}
impl crate::fbo::FramebufferBackend for NativeGlBackend {
    fn create_framebuffer(&mut self)->i32 {let mut id=0;gl!(self,"glGenFramebuffers",unsafe extern "system" fn(i32,*mut u32),1,&mut id);id as i32}
    fn bind_framebuffer(&mut self,target:i32,id:i32){gl!(self,"glBindFramebuffer",unsafe extern "system" fn(u32,u32),target as u32,id as u32);}
    fn check_error(&mut self,label:&str){self.check(label);}
    fn viewport(&mut self)->[i32;4] {let mut value=[0;4];gl!(self,"glGetIntegerv",unsafe extern "system" fn(u32,*mut i32),2978,value.as_mut_ptr());value}
    fn draw_buffers(&mut self,attachments:&[i32]){gl!(self,"glDrawBuffers",unsafe extern "system" fn(i32,*const u32),i32::try_from(attachments.len()).unwrap(),attachments.as_ptr().cast());}
    fn set_viewport(&mut self,value:[i32;4]){gl!(self,"glViewport",unsafe extern "system" fn(i32,i32,i32,i32),value[0],value[1],value[2],value[3]);}
    fn delete_framebuffer(&mut self,id:i32){gl!(self,"glDeleteFramebuffers",unsafe extern "system" fn(i32,*const u32),1,&(id as u32));}
}
impl crate::gl_state::StateBackend for NativeGlBackend {
    fn get_integer(&mut self,name:i32)->i32 {self.integer(name as u32)}
    fn stencil_mask(&mut self,mask:i32){gl!(self,"glStencilMask",unsafe extern "system" fn(u32),mask as u32);}
    fn stencil_func(&mut self,func:i32,reference:i32,mask:i32){gl!(self,"glStencilFunc",unsafe extern "system" fn(u32,i32,u32),func as u32,reference,mask as u32);}
    fn stencil_op(&mut self,failure:i32,depth_failure:i32,success:i32){gl!(self,"glStencilOp",unsafe extern "system" fn(u32,u32,u32),failure as u32,depth_failure as u32,success as u32);}
}
impl crate::passes::native_pass_factory::PassStateBackend for NativeGlBackend {
    fn stencil_test(&mut self,enabled:bool){if enabled {gl!(self,"glEnable",unsafe extern "system" fn(u32),2960);}else {gl!(self,"glDisable",unsafe extern "system" fn(u32),2960);}}
    fn color_mask(&mut self,mask:[bool;4]){gl!(self,"glColorMask",unsafe extern "system" fn(u8,u8,u8,u8),u8::from(mask[0]),u8::from(mask[1]),u8::from(mask[2]),u8::from(mask[3]));}
}
impl crate::source_ship::ShipSceneStateBackend for NativeGlBackend {
    fn clear(&mut self,mask:i32){gl!(self,"glClear",unsafe extern "system" fn(u32),mask as u32);}
    fn enable(&mut self,capability:i32){gl!(self,"glEnable",unsafe extern "system" fn(u32),capability as u32);}
    fn disable(&mut self,capability:i32){gl!(self,"glDisable",unsafe extern "system" fn(u32),capability as u32);}
    fn blend_equation(&mut self,equation:i32){gl!(self,"glBlendEquation",unsafe extern "system" fn(u32),equation as u32);}
    fn blend_func(&mut self,source:i32,destination:i32){gl!(self,"glBlendFunc",unsafe extern "system" fn(u32,u32),source as u32,destination as u32);}
}
