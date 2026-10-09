//! GUI.java uses the bundled FiraSans-Regular face for both F18 and F12.
//! Framebuffer/content-scale font sizes remain part of the pending DPI adapter.
use bevy::prelude::*;
use bevy::text::{FontFeatureTag, FontFeatures, FontSize, LineHeight};

/// Preserve the source pixel-height size for logical UI operations.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct SourceSize(pub FontSize);

/// STBTT_ScaleForPixelHeight uses hhea ascent minus descent, whereas Bevy uses
/// unitsPerEm. Bundled FiraSans has 1000 units/em and hhea 935/-265, gap 0.
/// Keep the operation order of pixel-height division followed by em scaling.
pub(crate) fn em_size(pixel_height:f32)->f32 {(pixel_height/1200.0)*1000.0}


/// Original Font.renderText emits one glyph per character and sums advances.
/// Bevy shaping must not insert Latin ligatures, contextual substitutions or kerning.
/// This does not replace the original atlas, Unicode errors or full layout rules.
pub(crate) fn independent_features() -> FontFeatures {
    FontFeatures::builder()
        .set(FontFeatureTag::STANDARD_LIGATURES, 0)
        .set(FontFeatureTag::CONTEXTUAL_LIGATURES, 0)
        .set(FontFeatureTag::DISCRETIONARY_LIGATURES, 0)
        .set(FontFeatureTag::new(b"hlig"), 0)
        .set(FontFeatureTag::CONTEXTUAL_ALTERNATES, 0)
        .set(FontFeatureTag::new(b"kern"), 0)
        .build()
}
pub(crate) fn apply(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut fonts: Query<(Entity, &mut TextFont, Option<&SourceSize>), (With<Text2d>, Added<TextFont>)>,
) {
    let source_font = assets.load("FiraSans-Regular.ttf");
    for (entity, mut font, source_size) in &mut fonts {
        font.font = bevy::text::FontSource::Handle(source_font.clone());
        font.font_features = independent_features();
        // Cloned layouts carry SourceSize and must not be scaled a second time.
        let logical=source_size.map_or(font.font_size,|size|size.0);
        if let FontSize::Px(height)=logical {
            font.font_size=FontSize::Px(em_size(height));
            commands.entity(entity).insert((SourceSize(logical),LineHeight::Px(height)));
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{app::PluginsState, camera::RenderTarget, render::render_resource::TextureFormat,
        text::TextLayoutInfo, window::ExitCondition};
    #[test]
    #[ignore = "Requires actual GPU and bundled FiraSans; explicit source UI regression gate"]
    fn actual_gpu_source_ui_font_keeps_latin_glyphs_and_disables_pair_kerning() {
        let mut app=App::new();
        app.add_plugins(DefaultPlugins
            .set(AssetPlugin {file_path:format!("{}/assets",env!("CARGO_MANIFEST_DIR")),..default()})
            .set(WindowPlugin {primary_window:None,exit_condition:ExitCondition::DontExit,..default()})
            .disable::<bevy::winit::WinitPlugin>()
            .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>())
            .add_systems(PostUpdate,apply.before(bevy::sprite::update_text2d_layout));
        while app.plugins_state()!=PluginsState::Ready {bevy::tasks::tick_global_task_pools_on_main_thread();std::thread::sleep(std::time::Duration::from_millis(10));}
        app.finish();app.cleanup();
        let target=app.world_mut().resource_mut::<Assets<Image>>().add(Image::new_target_texture(640,360,TextureFormat::Rgba8UnormSrgb,None));
        app.world_mut().spawn((Camera2d,RenderTarget::Image(target.into())));
        // Text2d only computes a target scale when the camera sees a Sprite.
        app.world_mut().spawn(Sprite::from_color(Color::WHITE,Vec2::ONE));
        let sentinel=app.world_mut().spawn((Text2d::new("."),TextFont::from_font_size(18.0),Visibility::Hidden)).id();
        let mut samples=Vec::new();
        for sample in ["AV","To","1000 floors","office","ffi","fi","fl","Ball of steel"] {
            let whole=app.world_mut().spawn((Text2d::new(sample),TextFont::from_font_size(18.0),Visibility::Hidden)).id();
            let singles:Vec<_>=sample.chars().map(|c|app.world_mut().spawn((Text2d::new(format!("{c}.")),TextFont::from_font_size(18.0),Visibility::Hidden)).id()).collect();
            samples.push((sample,whole,singles));
        }
        let start=std::time::Instant::now();
        loop {
            app.update();std::thread::sleep(std::time::Duration::from_millis(10));
            if samples.iter().all(|(_,e,s)|app.world().get::<TextLayoutInfo>(*e).is_some_and(|i|i.size.x>0.0)&&s.iter().all(|e|app.world().get::<TextLayoutInfo>(*e).is_some_and(|i|i.size.x>0.0))) {break;}
            assert!(start.elapsed()<std::time::Duration::from_secs(20),"Original font did not load");
        }
        let identity=|i:&bevy::text::PositionedGlyph|(i.atlas_info.texture,i.atlas_info.rect,i.atlas_info.offset,i.atlas_info.is_alpha_mask);
        for (sample,whole,singles) in &samples {
            assert_eq!(app.world().get::<TextFont>(*whole).unwrap().font_features,independent_features());
            let whole=app.world().get::<TextLayoutInfo>(*whole).unwrap();
            let mut expected=Vec::new();let mut width=0.0;
            let sentinel=app.world().get::<TextLayoutInfo>(sentinel).unwrap();
            for single in singles {
                let info=app.world().get::<TextLayoutInfo>(*single).unwrap();
                // A trailing dot preserves whitespace advance instead of Parley trimming a single space.
                assert_eq!(identity(info.glyphs.last().unwrap()),identity(&sentinel.glyphs[0]));
                expected.extend(info.glyphs[..info.glyphs.len()-1].iter().map(|g|(identity(g),g.position+Vec2::new(width,0.0))));
                // TextLayoutInfo.size is ceiled; the dot origin retains the fractional advance.
                width+=info.glyphs.last().unwrap().position.x-sentinel.glyphs[0].position.x;
            }
            assert_eq!(whole.glyphs.len(),expected.len(),"Ligature changed independent glyph count in {sample}");
            for (glyph,(id,position)) in whole.glyphs.iter().zip(expected) {
                assert_eq!(identity(glyph),id,"Glyph substitution in {sample}");
                assert!(glyph.position.distance(position)<0.001,"Kerning/context position in {sample}: {:?} != {position:?}",glyph.position);
            }
            assert!((whole.size.x-width.ceil()).abs()<0.001,"Non-independent width for {sample}");
            let source=crate::source_font_text::Font {font_size:18.0,advances:crate::source_font_text::F18}
                .measure(18.0,f32::MAX,0.0,&sample.encode_utf16().collect::<Vec<_>>(),-1).unwrap();
            let live_width=whole.run_geometry.last().unwrap().bounds.max.x;
            assert!((live_width-source.size.x).abs()<0.00005,"Native/source advance difference for {sample}: {live_width} != {}",source.size.x);
            assert_eq!(whole.size.y,source.size.y,"Source line-height difference for {sample}");
            println!("Source-independent native font sample={sample:?}, glyphs={}, width={}, summed_width={width}, source_width={}, live_width={live_width}",whole.glyphs.len(),whole.size.x,source.size.x);
        }
        // A default-shaped control proves this fixture detects the prior ligature/kerning behavior.
        let controls:Vec<_>=samples.iter().filter(|(s,_,_)|matches!(*s,"AV"|"office")).map(|(s,e,_)|(*s,*e,app.world().get::<TextLayoutInfo>(*e).unwrap().clone())).collect();
        for (_,entity,_) in &controls {app.world_mut().get_mut::<TextFont>(*entity).unwrap().font_features=FontFeatures::default();}
        for _ in 0..8 {app.update();std::thread::sleep(std::time::Duration::from_millis(10));}
        for (sample,entity,independent) in controls {
            let shaped=app.world().get::<TextLayoutInfo>(entity).unwrap();
            if sample=="office" {assert!(shaped.glyphs.len()<independent.glyphs.len(),"Baseline ligatures were not detected");}
            else {assert!((shaped.run_geometry.last().unwrap().bounds.max.x-independent.run_geometry.last().unwrap().bounds.max.x).abs()>0.01,"Baseline kerning was not detected");}
            println!("Default-shaped control={sample}, glyphs={}, width={}",shaped.glyphs.len(),shaped.size.x);
        }
    }
}
