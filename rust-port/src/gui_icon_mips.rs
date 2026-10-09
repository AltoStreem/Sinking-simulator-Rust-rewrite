//! Source Texture2D(..., mipmap=true): regenerate encoded icon mips on upload.
use bevy::{prelude::*,core_pipeline::{schedule::Core2d,Core2dSystems},
    render::{RenderApp,extract_resource::{ExtractResource,ExtractResourcePlugin},
        render_asset::RenderAssets,render_resource::{PipelineCache,TextureId},
        renderer::RenderContext,texture::GpuImage}};
use std::collections::HashMap;

#[derive(Resource,Default,Clone,ExtractResource)]
pub(crate) struct IconMipRequests(pub(crate) HashMap<AssetId<Image>,u64>);
pub(crate) struct IconMipPlugin;
impl Plugin for IconMipPlugin {
    fn build(&self,app:&mut App) {
        app.init_resource::<IconMipRequests>().add_plugins(ExtractResourcePlugin::<IconMipRequests>::default());
        if !app.is_plugin_added::<crate::render_fbo::MipmapPipelinePlugin>() {
            app.add_plugins(crate::render_fbo::MipmapPipelinePlugin);
        }
        if let Some(render)=app.get_sub_app_mut(RenderApp) {
            render.add_systems(Core2d,generate_icon_mips.before(Core2dSystems::MainPass));
        }
    }
}
fn generate_icon_mips(requests:Res<IconMipRequests>,images:Res<RenderAssets<GpuImage>>,
    pipeline:Res<crate::render_fbo::MipmapPipeline>,cache:Res<PipelineCache>,
    mut context:RenderContext,mut completed:Local<HashMap<AssetId<Image>,(u64,TextureId)>>) {
    // Windows uploads the exact source GL-generated mip bytes. Recomputing
    // them in WGSL would replace those bytes with different rounding results.
    if cfg!(windows) {return;}
    completed.retain(|id,_|requests.0.contains_key(id));
    for (&id,&revision) in &requests.0 {
        let Some(image)=images.get(id) else {continue};
        let state=(revision,image.texture.id());
        if completed.get(&id)==Some(&state) {continue;}
        if crate::render_fbo::encode_texture_mips(image,&pipeline,&cache,&mut context) {
            completed.insert(id,state);
        }
    }
}

#[cfg(test)]
#[path="gui_icon_mips_gpu_tests.rs"]
mod gpu_tests;
