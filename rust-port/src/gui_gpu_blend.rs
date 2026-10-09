//! ImGui ImplGL3.setupRenderState: FUNC_ADD, SRC_ALPHA / ONE_MINUS_SRC_ALPHA
//! for both RGB and alpha, plus encoded tint/texture multiplication.
//! Preserve Bevy's geometry and bindings; replace GUI sprite/color shaders.
use bevy::{prelude::*,camera::CompositingSpace,core_pipeline::core_2d::Transparent2d,
    render::{Render,RenderApp,RenderStartup,RenderSystems,camera::ExtractedCamera,
        render_phase::ViewSortedRenderPhases,
        render_resource::{BlendState,CachedRenderPipelineId,PipelineCache,RenderPipelineDescriptor},
        view::ExtractedView}};
use std::collections::HashMap;

pub(crate) struct SourceGuiBlendPlugin;
impl Plugin for SourceGuiBlendPlugin {
    fn build(&self,app:&mut App) {
        app.add_plugins(crate::gui_icon_mips::IconMipPlugin);
        app.init_resource::<crate::gui_texture::GuiTextures>()
            .add_systems(PostUpdate,crate::gui_texture::sync.after(bevy::sprite::update_text2d_layout));
        if let Some(render)=app.get_sub_app_mut(RenderApp) {
            render.add_systems(RenderStartup,load_shaders);
            render.add_systems(Render,source_gui_blend.after(RenderSystems::Queue).before(RenderSystems::PhaseSort));
        }
    }
}

#[derive(Resource)]
struct GuiShaders {sprite:Handle<Shader>,color:Handle<Shader>,original_color:Handle<Shader>}
fn load_shaders(mut commands:Commands,server:Res<AssetServer>) {
    use bevy::{shader::ShaderRef,sprite_render::Material2d};
    let ShaderRef::Path(path)=ColorMaterial::fragment_shader() else {unreachable!("Pinned Bevy ColorMaterial shader path")};
    commands.insert_resource(GuiShaders {sprite:server.load("shaders/gui_sprite.wgsl"),
        color:server.load("shaders/gui_color_material.wgsl"),original_color:server.load(path)});
}

fn source_descriptor(mut descriptor:RenderPipelineDescriptor)->Option<RenderPipelineDescriptor> {
    let fragment=descriptor.fragment.as_mut()?;
    let mut changed=false;
    for target in fragment.targets.iter_mut().flatten() {
        if target.blend==Some(BlendState::ALPHA_BLENDING) {
            let mut blend=BlendState::ALPHA_BLENDING;
            blend.alpha=blend.color;
            target.blend=Some(blend);changed=true;
        }
    }
    changed.then_some(descriptor)
}

fn source_gui_blend(mut cache:ResMut<PipelineCache>,shaders:Res<GuiShaders>,
    views:Query<(&ExtractedCamera,&ExtractedView)>,
    mut phases:ResMut<ViewSortedRenderPhases<Transparent2d>>,
    mut replacements:Local<HashMap<CachedRenderPipelineId,CachedRenderPipelineId>>) {
    // Publish newly queued descriptors before reading them. Bevy's normal
    // later Render step compiles the replacement pipelines queued below.
    cache.process_queue();
    for (camera,view) in &views {
        if camera.compositing_space!=Some(CompositingSpace::Srgb) {continue;}
        let Some(phase)=phases.get_mut(&view.retained_view_entity) else {continue};
        for item in phase.items.values_mut() {
            let source=item.pipeline;
            if let Some(&replacement)=replacements.get(&source) {item.pipeline=replacement;continue;}
            if let Some(mut descriptor)=source_descriptor(cache.get_render_pipeline_descriptor(source).clone()) {
                if descriptor.label.as_deref()==Some("sprite_pipeline") {
                    descriptor.vertex.shader=shaders.sprite.clone();
                    descriptor.fragment.as_mut().unwrap().shader=shaders.sprite.clone();
                } else if descriptor.fragment.as_ref().unwrap().shader==shaders.original_color {
                    descriptor.fragment.as_mut().unwrap().shader=shaders.color.clone();
                }
                let replacement=cache.queue_render_pipeline(descriptor);
                replacements.insert(source,replacement);item.pipeline=replacement;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::render_resource::{ColorTargetState,ColorWrites,FragmentState,TextureFormat};
    #[test]
    fn source_gui_descriptor_changes_both_blend_factors_without_touching_opaque_targets() {
        let descriptor=RenderPipelineDescriptor {fragment:Some(FragmentState {
            targets:vec![Some(ColorTargetState {format:TextureFormat::Rgba8Unorm,
                blend:Some(BlendState::ALPHA_BLENDING),write_mask:ColorWrites::ALL}),
                Some(ColorTargetState {format:TextureFormat::Rgba8Unorm,
                blend:None,write_mask:ColorWrites::ALL})],..default()}),..default()};
        let converted=source_descriptor(descriptor).unwrap();
        let targets=&converted.fragment.as_ref().unwrap().targets;
        let blend=targets[0].as_ref().unwrap().blend.unwrap();
        assert_eq!(blend.color,BlendState::ALPHA_BLENDING.color);
        assert_eq!(blend.alpha,blend.color);
        assert_eq!(targets[1].as_ref().unwrap().blend,None);
        assert!(source_descriptor(converted).is_none(),"Already converted pipelines stay unchanged");
    }
}
