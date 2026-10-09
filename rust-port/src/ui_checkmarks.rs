//! Bundled ImGui renderCheckMark/pathStroke geometry; no font glyph substitution.
//! Current checkbox extents are retained; complete source DPI/layout is separate.
use crate::*;
use bevy::{asset::RenderAssetUsages,mesh::{Indices,PrimitiveTopology},sprite_render::AlphaMode2d};
use std::collections::HashMap;

#[derive(Debug)]
struct Geometry {positions:Vec<Vec2>,colors:Vec<[f32;4]>,indices:Vec<u32>}
fn path(pos:Vec2,size:f32)->([Vec2;3],f32) {
    let thickness=(size/5.0).max(1.0);
    let size=size-thickness*0.5;
    let pos=pos+Vec2::splat(thickness*0.25);
    let third=size/3.0;
    let bx=pos.x+third;let by=pos.y+size-third*0.5;
    ([Vec2::new(bx-third,by-third),Vec2::new(bx,by),Vec2::new(bx+third*2.0,by-third*2.0)],thickness)
}
fn geometry(square:f32)->Geometry {
    let pad=(square/6.0).floor().max(1.0);
    let (points,thickness)=path(Vec2::splat(-square*0.5+pad),square-pad*2.0);
    let mut normals=[Vec2::ZERO;3];
    for i in 0..2 {
        let mut dx=points[i+1].x-points[i].x;let mut dy=points[i+1].y-points[i].y;
        let d2=dx*dx+dy*dy;
        if d2>0.0 {let inv=1.0/((d2 as f64).sqrt() as f32);dx*=inv;dy*=inv;}
        normals[i]=Vec2::new(dy,-dx);
    }
    normals[2]=normals[1];
    let mut joined=normals;
    for i in 1..3 {
        let mut dx=(normals[i-1].x+normals[i].x)*0.5;
        let mut dy=(normals[i-1].y+normals[i].y)*0.5;
        let mut d2=dx*dx+dy*dy;if d2<0.5 {d2=0.5;}
        let inv=1.0/d2;dx*=inv;dy*=inv;joined[i]=Vec2::new(dx,dy);
    }
    // Original getColorU32 rounds Classic CheckMark(.9,.9,.9,.5) to 0x80e6e6e6.
    let color=[230.0/255.0,230.0/255.0,230.0/255.0,128.0/255.0];
    let transparent=[color[0],color[1],color[2],0.0];
    let mut result=Geometry {positions:Vec::new(),colors:Vec::new(),indices:Vec::new()};
    if thickness>1.0 {
        let inner=(thickness-1.0)*0.5;
        for i in 0..3 {
            let outer=joined[i]*(inner+1.0);let inside=joined[i]*inner;
            result.positions.extend([points[i]+outer,points[i]+inside,points[i]-inside,points[i]-outer]);
            result.colors.extend([transparent,color,color,transparent]);
        }
        for i in 0..2u32 {let a=i*4;let b=(i+1)*4;
            result.indices.extend([b+1,a+1,a+2,a+2,b+2,b+1,b+1,a+1,a,a,b,b+1,b+2,a+2,a+3,a+3,b+3,b+2]);
        }
    } else {
        for i in 0..3 {
            result.positions.extend([points[i],points[i]+joined[i],points[i]-joined[i]]);
            result.colors.extend([color,transparent,transparent]);
        }
        for i in 0..2u32 {let a=i*3;let b=(i+1)*3;
            result.indices.extend([b,a,a+2,a+2,b+2,b,b+1,a+1,a,a,b,b+1]);
        }
    }
    result
}
fn mesh(square:f32,pixels_per_unit:f32)->Mesh {
    let data=geometry(square*pixels_per_unit);
    Mesh::new(PrimitiveTopology::TriangleList,RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION,data.positions.iter().map(|p|[p.x/pixels_per_unit,-p.y/pixels_per_unit,0.0]).collect::<Vec<_>>())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0,vec![[0.0f32;2];data.positions.len()])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR,data.colors)
        .with_inserted_indices(Indices::U32(data.indices))
}
#[derive(Resource)]
pub(crate) struct Checkmarks {material:Handle<ColorMaterial>,meshes:HashMap<(u32,u32),Handle<Mesh>>}
pub(crate) fn spawn(mut commands:Commands,mut materials:ResMut<Assets<ColorMaterial>>) {
    let material=materials.add(ColorMaterial {alpha_mode:AlphaMode2d::Blend,..default()});
    commands.insert_resource(Checkmarks {material,meshes:HashMap::new()});
}
/// After sync_settings_ui/route_layers and before Text2d layout/extraction.
/// Attach to the existing semantic mark entity so its page visibility and render
/// layers remain owned by the existing content/scroll camera adapter.
pub(crate) fn sync(mut commands:Commands,simulation:Res<Simulation>,
    windows:Query<&Window,With<bevy::window::PrimaryWindow>>,
    buttons:Query<(&SettingButton,&Sprite,&Transform),Without<SettingToggleMark>>,
    mut marks:Query<(Entity,&SettingToggleMark,&mut Text2d,&mut Transform,Option<&Mesh2d>),Without<SettingButton>>,
    mut assets:ResMut<Checkmarks>,mut meshes:ResMut<Assets<Mesh>>) {
    let scale=windows.iter().next().map_or(1.0,|window|window.physical_height() as f32/720.0).max(f32::MIN_POSITIVE);
    for (entity,mark,mut text,mut transform,current) in &mut marks {
        let checked=match mark.0 {SettingAction::ToggleTools=>simulation.show_tools,SettingAction::ToggleCycle=>simulation.cycle_enabled,
            SettingAction::ToggleWater=>simulation.show_internal_water,_=>continue};
        if !text.0.is_empty() {text.0.clear();}
        let button=buttons.iter().find(|(button,_,_)|std::mem::discriminant(&button.0)==std::mem::discriminant(&mark.0));
        let Some((_,sprite,button_transform))=button else {continue};
        transform.translation.x=button_transform.translation.x;transform.translation.y=button_transform.translation.y;
        let square=sprite.custom_size.map_or(20.0,|size|size.x.min(size.y));
        // Source unchecked boxes emit no checkmark primitive. In pinned Bevy,
        // a zero-vertex render mesh is not allocated but is still uploaded,
        // producing allocator errors. Remove the draw components instead.
        if !checked {
            if current.is_some() {commands.entity(entity).remove::<(Mesh2d,MeshMaterial2d<ColorMaterial>)>();}
            continue;
        }
        let handle=assets.meshes.entry((square.to_bits(),scale.to_bits())).or_insert_with(||meshes.add(mesh(square,scale))).clone();
        if current.is_none_or(|current|current.0!=handle) {commands.entity(entity).insert((Mesh2d(handle),MeshMaterial2d(assets.material.clone())));}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_checkmark_path_and_thick_stroke_keep_source_sizes_colors_and_indices() {
        let (points,t)=path(Vec2::splat(4.0),16.0);
        assert_eq!(t,3.2);
        for (got,expected) in points.into_iter().zip([Vec2::new(4.8,12.0),Vec2::new(9.6,16.8),Vec2::new(19.2,7.2)]) {
            assert!((got-expected).abs().max_element()<0.00001);
        }
        let data=geometry(24.0);assert_eq!(data.positions.len(),12);assert_eq!(data.indices.len(),36);
        assert_eq!(&data.indices[..18],&[5,1,2,2,6,5,5,1,0,0,4,5,6,2,3,3,7,6]);
        assert_eq!(data.colors[0][3],0.0);assert_eq!(data.colors[1][3],128.0/255.0);
        assert_eq!(data.colors[1][0],230.0/255.0);
    }
    #[test]
    fn small_checkmark_uses_original_thin_line_branch() {
        let data=geometry(6.0);assert_eq!(data.positions.len(),9);assert_eq!(data.indices.len(),24);
        assert_eq!(&data.indices[..12],&[3,0,2,2,5,3,4,1,0,0,3,4]);
        assert_eq!(data.colors[0][3],128.0/255.0);assert_eq!(data.colors[1][3],0.0);
        assert!(data.positions.iter().all(|p|p.is_finite()));
    }
    #[test]
    fn semantic_checkmark_replaces_text_and_preserves_page_visibility_and_render_layers() {
        let mut app=App::new();
        app.init_resource::<Assets<Mesh>>().init_resource::<Assets<ColorMaterial>>().insert_resource(Simulation::default());
        app.add_systems(Startup,spawn).add_systems(PostUpdate,sync);
        app.world_mut().spawn((SettingButton(SettingAction::ToggleCycle),Sprite::from_color(Color::WHITE,Vec2::splat(20.0)),Transform::from_xyz(5.0,7.0,24.5)));
        let mark=app.world_mut().spawn((SettingToggleMark(SettingAction::ToggleCycle),Text2d::new("✓"),Transform::from_xyz(5.0,6.0,25.5),Visibility::Hidden,RenderLayers::layer(2),TabPage(ToolboxTab::Graphics))).id();
        app.world_mut().resource_mut::<Simulation>().cycle_enabled=true;
        app.update();
        assert!(app.world().get::<Text2d>(mark).unwrap().0.is_empty());
        assert_eq!(*app.world().get::<Visibility>(mark).unwrap(),Visibility::Hidden);
        assert_eq!(*app.world().get::<RenderLayers>(mark).unwrap(),RenderLayers::layer(2));
        assert_eq!(app.world().get::<Transform>(mark).unwrap().translation,Vec3::new(5.0,7.0,25.5));
        let checked=app.world().get::<Mesh2d>(mark).unwrap().0.clone();
        assert_eq!(app.world().resource::<Assets<Mesh>>().get(&checked).unwrap().count_vertices(),12);
        app.world_mut().resource_mut::<Simulation>().cycle_enabled=false;
        app.update();
        assert!(app.world().get::<Mesh2d>(mark).is_none());
        assert!(app.world().get::<MeshMaterial2d<ColorMaterial>>(mark).is_none());
        assert_eq!(*app.world().get::<Visibility>(mark).unwrap(),Visibility::Hidden);
    }
}


