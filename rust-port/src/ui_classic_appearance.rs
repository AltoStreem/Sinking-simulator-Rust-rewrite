//! Bundled ImGui Style.styleColorsClassic, applied to semantic native UI sprites.
//! Geometry/DPI, rounded borders and the complete ImGui active-ID protocol remain pending.
use crate::*;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind { Window, Popup, Invisible, Title, Tab(bool), Selectable(bool), Button, Frame, Scroll(bool), Close }

// Original getColorU32 packs each saturated float channel into an 8-bit vertex color.
pub(crate) fn rgba(v:[f32;4])->Color {
    let v=v.map(|channel|((channel.clamp(0.0,1.0)*255.0+0.5) as u32) as f32/255.0);
    Color::srgba(v[0],v[1],v[2],v[3])
}
fn mix(a:[f32;4],b:[f32;4],t:f32)->[f32;4] { std::array::from_fn(|i|a[i]+(b[i]-a[i])*t) }
fn color(kind:Kind, hovered:bool, held:bool)->Color {
    let header=[0.4,0.4,0.9,0.45];
    let title=[0.32,0.32,0.63,0.87];
    rgba(match kind {
        Kind::Window=>[0.0,0.0,0.0,0.7],
        Kind::Popup=>[0.11,0.11,0.14,0.92],
        Kind::Invisible=>[0.0;4],
        Kind::Title=>title,
        Kind::Tab(selected)=>if hovered { [0.45,0.45,0.9,0.8] }
            else if selected {mix([0.53,0.53,0.87,0.8],title,0.6)}
            else {mix(header,title,0.8)},
        Kind::Selectable(selected)=>if hovered&&held {[0.53,0.53,0.87,0.8]} else if hovered {[0.45,0.45,0.9,0.8]} else if selected {header} else {[0.0;4]},
        Kind::Button=>if held&&hovered {[0.46,0.54,0.8,1.0]}
            else if hovered {[0.4,0.48,0.71,0.79]} else {[0.35,0.4,0.61,0.62]},
        Kind::Frame=>if held {[0.42,0.41,0.64,0.69]}
            else if hovered {[0.47,0.47,0.69,0.4]} else {[0.43,0.43,0.43,0.39]},
        Kind::Scroll(thumb)=>if !thumb {[0.2,0.25,0.3,0.6]}
            else if held {[0.41,0.39,0.8,0.6]}
            else if hovered {[0.4,0.4,0.8,0.4]} else {[0.4,0.4,0.8,0.3]},
        // Original close-button circle is drawn only on hover/hold; its X is separate text.
        Kind::Close=>if held&&hovered {[0.46,0.54,0.8,1.0]}
            else if hovered {[0.4,0.48,0.71,0.79]} else {[0.0;4]},
    })
}

fn classify(entity:&bevy::ecs::world::EntityRef, simulation:Option<&Simulation>,
    ui:Option<&ShipUploadUiState>, transform:&Transform)->Option<Kind> {
    use ship_upload_layout::Role;
    if let Some(role)=entity.get::<Role>() {
        return match role {
            Role::Chrome(_) if entity.contains::<ShipUploadButton>()=>Some(Kind::Close),
            Role::Chrome(local)=>Some(if *local==Vec2::ZERO {Kind::Window}else {Kind::Title}),
            Role::Field(_)=>Some(Kind::Frame),
            Role::Resource(_)|Role::Add|Role::Save|Role::TestFrame=>Some(Kind::Button),
            Role::Tab=>Some(Kind::Tab(entity.get::<ShipUploadLayerTab>()
                .zip(ui).is_some_and(|(tab,ui)|tab.0==ui.active_layer))),
            Role::Hidden=>Some(Kind::Invisible),
            _=>None, // Actual preview images and text are never recolored.
        };
    }
    if entity.contains::<toolbox_viewport::Backdrop>() {
        // Existing border is a filled larger rectangle, not an outline. Making it
        // translucent gray would incorrectly tint the entire pane under WindowBg.
        return Some(if transform.translation.z<20.0 {Kind::Invisible}else {Kind::Window});
    }
    if entity.contains::<toolbox_viewport::PageBackdrop>() {return Some(Kind::Invisible);}
    if entity.contains::<ToolboxHeader>() {return Some(Kind::Title);}
    if let Some(tab)=entity.get::<TabButton>() {return Some(Kind::Tab(simulation.is_some_and(|s|s.active_tab==tab.0)));}
    if entity.contains::<toolbox_viewport::ScrollThumb>() {return Some(Kind::Scroll(true));}
    if entity.contains::<toolbox_viewport::ScrollTrack>() {return Some(Kind::Scroll(false));}
    if entity.contains::<ship_upload_layout::Bar>() {return Some(Kind::Scroll(entity.get::<Sprite>().and_then(|sprite|sprite.custom_size).is_some_and(|size|size.x<14.0)));}
    if entity.contains::<ShipUploadButton>()||entity.contains::<ToolCard>()||entity.contains::<ShipCard>() {return Some(Kind::Button);}
    if let Some(button)=entity.get::<SettingButton>() {
        return Some(if matches!(button.0,SettingAction::ToggleTools|SettingAction::ToggleCycle|SettingAction::ToggleWater) {Kind::Frame}else {Kind::Button});
    }
    if entity.contains::<SettingRow>() {return Some(Kind::Frame);}
    if entity.contains::<ShipLayerDropdownBackground>() {return Some(Kind::Popup);}
    if let Some(option)=entity.get::<ShipLayerDropdownOption>() {return Some(Kind::Selectable(simulation.is_some_and(|s|s.selected_layer==option.0)));}
    if entity.contains::<ToolPanelUi>() {return Some(if entity.get::<ToolPanelPosition>().is_some_and(|p|p.0.x== -40.0) {Kind::Button}else {Kind::Frame});}
    if entity.contains::<ToolboxContent>()&&entity.get::<TabPage>().is_some_and(|page|page.0==ToolboxTab::Ships) {
        return Some(Kind::Frame); // Untagged search field background.
    }
    if entity.contains::<ToolboxContent>()&&entity.get::<TabPage>().is_none() {return Some(Kind::Invisible);} // Artificial tab-strip rectangle.
    None
}

/// Schedule after the legacy UI sync systems so their opaque fallback colors
/// cannot overwrite source Classic alpha. Does not change scene assets or geometry.
pub(crate) fn apply(world:&mut World) {
    let mut windows=world.query::<&Window>();
    let mut sprites=world.query::<(Entity,&Sprite,&Transform,Option<&Visibility>)>();
    let simulation=world.get_resource::<Simulation>();
    let ui=world.get_resource::<ShipUploadUiState>();
    let mouse=world.get_resource::<ButtonInput<MouseButton>>();
    let held=mouse.is_some_and(|mouse|mouse.pressed(MouseButton::Left));
    let point=windows.iter(world).next().and_then(|window|window.cursor_position().map(|cursor|
        Vec2::new(cursor.x-window.width()*0.5,window.height()*0.5-cursor.y)*(720.0/window.height().max(1.0))));
    let updates:Vec<_>=sprites.iter(world).filter_map(|(id,sprite,transform,visibility)| {
        // Source image tint is separate from frame styling; never recolor textures.
        if sprite.image!=Handle::<Image>::default() {return None;}
        let entity=world.entity(id);
        let kind=classify(&entity,simulation,ui,transform)?;
        let local_point=point.map(|point| {
            if let Some(role)=entity.get::<ship_upload_layout::Role>() {
                if !matches!(role,ship_upload_layout::Role::Chrome(_)) {
                    return world.get_resource::<ship_upload_layout::Layout>().and_then(|layout|ship_upload_layout::content_point(layout,point));
                }
            } else if entity.contains::<TabPage>() {
                if let Some(simulation)=simulation {return toolbox_viewport::content_point(simulation,point);}
            }
            Some(point)
        }).flatten();
        let hovered=visibility!=Some(&Visibility::Hidden)&&local_point.zip(sprite.custom_size).is_some_and(|(point,size)|
            Rect::from_center_size(transform.translation.truncate(),size*transform.scale.truncate()).contains(point));
        let focused=entity.get::<ShipUploadFieldControl>().zip(ui).is_some_and(|(field,ui)|ui.active_focus==Some(field.0));
        Some((id,color(kind,hovered,focused||(held&&hovered))))
    }).collect();
    for (entity,color) in updates {if let Some(mut sprite)=world.get_mut::<Sprite>(entity) {sprite.color=color;}}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundled_classic_palette_preserves_original_alpha_and_tab_lerp() {
        assert_eq!(color(Kind::Window,false,false),rgba([0.0,0.0,0.0,0.7]));
        assert_eq!(color(Kind::Frame,false,false),rgba([0.43,0.43,0.43,0.39]));
        assert_eq!(color(Kind::Button,true,true),rgba([0.46,0.54,0.8,1.0]));
        assert_eq!(color(Kind::Tab(false),false,false).to_srgba(),rgba(mix([0.4,0.4,0.9,0.45],[0.32,0.32,0.63,0.87],0.8)).to_srgba());
        assert_eq!(color(Kind::Close,false,false).to_srgba().alpha,0.0);
    }
    #[test]
    fn popup_uses_original_popup_palette_and_packed_vertex_channels() {
        let popup=color(Kind::Popup,false,false).to_srgba();
        assert_eq!([popup.red,popup.green,popup.blue,popup.alpha],[28.0/255.0,28.0/255.0,36.0/255.0,235.0/255.0]);
        let mut world=World::new();world.insert_resource(Simulation::default());
        let id=world.spawn((ShipLayerDropdownBackground,Sprite::default(),Transform::default())).id();
        apply(&mut world);
        assert_eq!(world.get::<Sprite>(id).unwrap().color,color(Kind::Popup,false,false));
        assert_ne!(world.get::<Sprite>(id).unwrap().color,color(Kind::Window,false,false));
        assert_eq!(rgba([-0.5,0.5,1.5,1.0]).to_srgba(),Color::srgba(0.0,128.0/255.0,1.0,1.0).to_srgba());
    }
    #[test]
    fn semantic_ui_style_changes_panels_without_touching_scene_or_texture_tint() {
        let mut world=World::new();world.insert_resource(Simulation::default());
        let pane=world.spawn((toolbox_viewport::Backdrop,Sprite::default(),Transform::from_xyz(0.0,0.0,20.0))).id();
        let border=world.spawn((toolbox_viewport::Backdrop,Sprite::default(),Transform::from_xyz(0.0,0.0,19.0))).id();
        let scene=world.spawn((Sprite::from_color(Color::srgb(1.0,0.0,0.0),Vec2::ONE),Transform::default())).id();
        let mut images=Assets::<Image>::default();let image=images.add(Image::default());world.insert_resource(images);
        let texture=world.spawn((ToolCard(Tool::Break),Sprite::from_image(image),Transform::default())).id();
        let before=world.get::<Sprite>(texture).unwrap().color;
        apply(&mut world);
        assert_eq!(world.get::<Sprite>(pane).unwrap().color,color(Kind::Window,false,false));
        assert_eq!(world.get::<Sprite>(border).unwrap().color,color(Kind::Invisible,false,false));
        assert_eq!(world.get::<Sprite>(scene).unwrap().color,Color::srgb(1.0,0.0,0.0));
        assert_eq!(world.get::<Sprite>(texture).unwrap().color,before);
    }
    #[test]
    fn source_drag_labels_begin_after_numeric_field_without_reanchoring_other_labels() {
        let mut app=App::new();
        app.add_systems(Startup,|mut commands:Commands| {
            spawn_page_label(&mut commands,ToolboxTab::Physics,"Wave Width",Vec3::new(-391.0,248.0,25.0),15.0);
            spawn_page_label(&mut commands,ToolboxTab::Graphics,"Show Tools",Vec3::new(-535.0,220.0,25.0),15.0);
        });
        app.update();
        let mut query=app.world_mut().query::<(&Text2d,&Transform,&bevy::sprite::Anchor)>();
        for (text,transform,anchor) in query.iter(app.world()) {
            if text.0=="Wave Width" {
                assert_eq!(*anchor,bevy::sprite::Anchor::CENTER_LEFT);
                let frame_right=-514.0+225.0*0.5;
                assert_eq!(transform.translation.x-frame_right,4.0);
                assert_eq!(transform.translation.y,248.0);
            } else {
                assert_eq!(*anchor,bevy::sprite::Anchor::CENTER);
                assert_eq!(transform.translation.x,-535.0);
            }
        }
    }
}




