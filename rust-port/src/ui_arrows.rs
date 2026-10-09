//! DrawList.renderArrow and its original AntiAliasedFill triangle tessellation.
//! Geometry is checked against the retained JVM, including its 0.5 normal clamp.
use crate::*;
use bevy::{asset::RenderAssetUsages, mesh::{Indices,PrimitiveTopology},sprite_render::AlphaMode2d};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum Direction {Down,Up,Right,Left}
#[derive(Debug)]
struct Geometry {positions:[Vec2;6],colors:[u32;6],indices:Vec<u32>}
fn geometry(pos:Vec2,font:f32,scale:f32,direction:Direction,color:u32)->Geometry {
    let h=font*1.0;let mut r=h*0.4*scale;
    let center=pos+Vec2::new(h*0.5,h*0.5*scale);
    let points=match direction {
        Direction::Down|Direction::Up=>{if direction==Direction::Up {r=-r;}
            [Vec2::new(0.0,0.75),Vec2::new(-0.866,-0.75),Vec2::new(0.866,-0.75)]},
        Direction::Right|Direction::Left=>{if direction==Direction::Left {r=-r;}
            [Vec2::new(0.75,0.0),Vec2::new(-0.75,0.866),Vec2::new(-0.75,-0.866)]},
    }.map(|point|center+point*r);
    let mut normals=[Vec2::ZERO;3];
    for i in 0..3 {
        let mut edge=points[(i+1)%3]-points[i];let d2=edge.x*edge.x+edge.y*edge.y;
        if d2>0.0 {let inv=1.0/((d2 as f64).sqrt() as f32);edge.x*=inv;edge.y*=inv;}
        normals[i]=Vec2::new(edge.y,-edge.x);
    }
    let mut result=Geometry {positions:[Vec2::ZERO;6],colors:[0;6],indices:vec![0,2,4]};
    for i in 0..3 {
        let prev=(i+2)%3;let mut dm=(normals[prev]+normals[i])*0.5;
        let mut d2=dm.x*dm.x+dm.y*dm.y;if d2<0.5 {d2=0.5;}
        let inv=1.0/d2;dm.x*=inv;dm.y*=inv;dm*=0.5;
        result.positions[i*2]=points[i]-dm;result.positions[i*2+1]=points[i]+dm;
        result.colors[i*2]=color;result.colors[i*2+1]=color&0x00ffffff;
        let inner=i as u32*2;let previous=prev as u32*2;
        result.indices.extend([inner,previous,previous+1,previous+1,inner+1,inner]);
    }
    result
}
// Original AA_SIZE is one ImGui window pixel. The Windows GLFW framebuffer
// uses the same pixel coordinates. Build the fringe in physical coordinates,
// then return to the fixed 720-unit camera space so resizing does not widen it.
fn screen_geometry(font:f32,direction:Direction,pixels_per_unit:f32)->Geometry {
    let mut result=geometry(Vec2::splat(-font*pixels_per_unit*0.5),font*pixels_per_unit,1.0,direction,0xffe6e6e6);
    for position in &mut result.positions {*position/=pixels_per_unit;}
    result
}
fn mesh(font:f32,direction:Direction,pixels_per_unit:f32)->Mesh {
    let geometry=screen_geometry(font,direction,pixels_per_unit);
    Mesh::new(PrimitiveTopology::TriangleList,RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION,geometry.positions.map(|p|[p.x,-p.y,0.0]).to_vec())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0,vec![[0.0f32;2];6])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR,geometry.colors.map(|color|[
            (color&255) as f32/255.0,((color>>8)&255) as f32/255.0,((color>>16)&255) as f32/255.0,(color>>24) as f32/255.0]).to_vec())
        .with_inserted_indices(Indices::U32(geometry.indices))
}
#[derive(Component)]
pub(crate) enum Arrow {Toolbox,Layer}
#[derive(Resource)]
pub(crate) struct ArrowAssets {down:Handle<Mesh>,right:Handle<Mesh>,layer:Handle<Mesh>,_material:Handle<ColorMaterial>,pixels_per_unit:f32,layer_font:f32}
pub(crate) fn spawn(mut commands:Commands,mut meshes:ResMut<Assets<Mesh>>,mut materials:ResMut<Assets<ColorMaterial>>) {
    let down=meshes.add(mesh(20.0,Direction::Down,1.0));let right=meshes.add(mesh(20.0,Direction::Right,1.0));
    let layer=meshes.add(mesh(18.0,Direction::Down,1.0));
    let material=materials.add(ColorMaterial {alpha_mode:AlphaMode2d::Blend,..default()});
    commands.spawn((Arrow::Toolbox,Mesh2d(down.clone()),MeshMaterial2d(material.clone()),Transform::from_xyz(-623.0,333.0,22.0),RenderLayers::layer(1)));
    commands.spawn((Arrow::Layer,source_tools_layout::Role::Arrow,ToolPanelUi,ToolPanelPosition(Vec2::new(-40.0,310.0)),Mesh2d(layer.clone()),MeshMaterial2d(material.clone()),Transform::from_xyz(-40.0,310.0,22.0),RenderLayers::layer(1)));
    commands.insert_resource(ArrowAssets {down,right,layer,_material:material,pixels_per_unit:1.0,layer_font:18.0});
}
pub(crate) fn sync(simulation:Res<Simulation>,mut assets:ResMut<ArrowAssets>,
    windows:Query<&Window,With<bevy::window::PrimaryWindow>>,mut meshes:ResMut<Assets<Mesh>>,
    mut arrows:Query<(&Arrow,&mut Mesh2d)>) {
    if let Ok(window)=windows.single() {
        let scale=window.physical_height() as f32/720.0;
        let layer_font=simulation.source_tools.map_or(18.0,|layout|layout.font(false));
        if scale>0.0 && (scale!=assets.pixels_per_unit||layer_font!=assets.layer_font) {
            for (handle,font,direction) in [(&assets.down,20.0,Direction::Down),(&assets.right,20.0,Direction::Right),(&assets.layer,layer_font,Direction::Down)] {
                if let Some(mut existing)=meshes.get_mut(handle) {*existing=mesh(font,direction,scale);}
            }
            assets.pixels_per_unit=scale;assets.layer_font=layer_font;
        }
    }
    for (arrow,mut mesh) in &mut arrows {
        let handle=match arrow {Arrow::Toolbox=>if simulation.toolbox_collapsed {&assets.right}else {&assets.down},Arrow::Layer=>&assets.layer};
        if mesh.0!=*handle {mesh.0=handle.clone();}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_jvm_arrow_vertices_colors_and_indices_match_bits() {
        let cases:serde_json::Value=serde_json::from_str(include_str!("../tools/fixtures/source-arrow-geometry.json")).unwrap();
        assert_eq!(cases.as_array().unwrap().len(),12);
        for case in cases.as_array().unwrap() {
            let direction=match case["direction"].as_str().unwrap() {"Down"=>Direction::Down,"Up"=>Direction::Up,"Right"=>Direction::Right,"Left"=>Direction::Left,_=>unreachable!()};
            let result=geometry(Vec2::new(3.0,4.0),case["font"].as_f64().unwrap() as f32,case["scale"].as_f64().unwrap() as f32,direction,u32::MAX);
            for (i,vertex) in case["vertices"].as_array().unwrap().iter().enumerate() {
                assert_eq!(result.positions[i].x.to_bits(),(vertex[0].as_f64().unwrap() as f32).to_bits(),"{direction:?} vertex {i} X");
                assert_eq!(result.positions[i].y.to_bits(),(vertex[1].as_f64().unwrap() as f32).to_bits(),"{direction:?} vertex {i} Y");
                assert_eq!(result.colors[i],vertex[2].as_u64().unwrap() as u32);
            }
            assert_eq!(result.indices,case["indices"].as_array().unwrap().iter().map(|i|i.as_u64().unwrap() as u32).collect::<Vec<_>>());
        }
    }
}

