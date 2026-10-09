//! Live Bevy window queries and ordered native-event bridge for SourceWindow.
//! Bevy owns presentation; SourceWindow::start's OpenGL loop is not used here.
use bevy::{prelude::*, window::{PrimaryWindow, WindowEvent as NativeEvent, WindowPosition}};
use crate::{resource::ResourceRuntime, window::{SourceWindow, WindowBackend, WindowEvent}};
use std::{collections::{BTreeSet,HashMap,VecDeque}, rc::Rc, sync::{Arc, Mutex}};

#[derive(Default)]
struct State {
    position: [i32;2],
    screen: [i32;2],
    framebuffer: [i32;2],
    cursor: [f64;2],
    close: bool,
    title_request: Option<String>,
    position_request: Option<[i32;2]>,
    destroy: bool,
    callbacks_freed: bool,
    events: Vec<WindowEvent>,
    modifier_keys: u8,
    native_modifiers: i32,
    native_mouse: VecDeque<(i32,i32,i32)>,
    native_keys: VecDeque<(KeyCode,i32,i32,i32)>,
    pressed_scans: HashMap<KeyCode,i32>,
    pressed_mouse: BTreeSet<i32>,
    native_commits: VecDeque<(String,i32)>,
}
struct Backend(Arc<Mutex<State>>);
impl WindowBackend for Backend {
    fn set_title(&mut self, _: i64, title: &str) { self.0.lock().unwrap().title_request=Some(title.into()); }
    fn position(&mut self, _: i64) -> [i32;2] { self.0.lock().unwrap().position }
    fn set_position(&mut self, _: i64, pos: [i32;2]) { self.0.lock().unwrap().position_request=Some(pos); }
    fn framebuffer_size(&mut self, _: i64) -> [i32;2] { self.0.lock().unwrap().framebuffer }
    fn screen_size(&mut self, _: i64) -> [i32;2] { self.0.lock().unwrap().screen }
    fn mouse_position(&mut self, _: i64) -> [f64;2] { self.0.lock().unwrap().cursor }
    fn should_close(&mut self, _: i64) -> bool { self.0.lock().unwrap().close }
    fn set_should_close(&mut self, _: i64, value: bool) {
        self.0.lock().unwrap().close=value;
    }
    fn set_clear_color(&mut self, _: [f32;4]) { panic!("Bevy presentation uses render passes, not SourceWindow::start"); }
    fn clear(&mut self, _: i32) { panic!("Bevy presentation uses render passes, not OpenGL clear"); }
    fn swap_buffers(&mut self, _: i64) { panic!("Bevy owns native presentation"); }
    fn poll_events(&mut self) -> Vec<WindowEvent> { std::mem::take(&mut self.0.lock().unwrap().events) }
    fn free_callbacks(&mut self, _: i64) { self.0.lock().unwrap().callbacks_freed=true; }
    fn destroy_window(&mut self, _: i64) { self.0.lock().unwrap().destroy=true; }
}

pub(crate) struct LiveWindow {
    pub(crate) source: Rc<SourceWindow>,
    entity: Entity,
    state: Arc<Mutex<State>>,
    text_hook:std::cell::RefCell<Option<crate::window_native_characters::Hook>>,
}
pub(crate) struct WindowBridgePlugin;
#[derive(Message, Clone, Debug)]
pub(crate) struct OrderedInput {
    pub(crate) event: WindowEvent,
    pub(crate) cursor: [f64; 2],
    pub(crate) screen: [i32; 2],
    pub(crate) frame_screen: [i32; 2],
}
impl Plugin for WindowBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResourceRuntime>()
            .init_resource::<crate::ui_input_capture::Capture>()
            .add_message::<bevy::winit::RawWinitWindowEvent>()
            .add_message::<crate::window_characters::CharacterInput>()
            .add_message::<crate::window_characters::ResetShip>()
            .add_message::<OrderedInput>()
            .add_message::<crate::camera_control::WorldInput>()
            .init_resource::<crate::tools::tool::NativeBrushInput>()
            .add_systems(Startup, initialize)
            .add_systems(PreUpdate, route.after(bevy::input::InputSystems))
            .add_systems(PreUpdate,crate::ui_input_capture::update.after(route).before(crate::tools::tool::native_brush_input))
            .add_systems(PreUpdate,crate::tools::tool::native_brush_input.after(route))
            .add_systems(PostUpdate, flush);
    }
}
fn initialize(world: &mut World) {
    let Some((entity,window))=world.query_filtered::<(Entity,&Window),With<PrimaryWindow>>()
        .iter(world).next() else {return};
    let state=Arc::new(Mutex::new(State::default()));
    {
        let mut snapshot=state.lock().unwrap();
        snapshot.screen=[window.width() as i32,window.height() as i32];
        snapshot.framebuffer=[window.physical_width() as i32,window.physical_height() as i32];
        if let WindowPosition::At(pos)=window.position {snapshot.position=pos.to_array();}
        if let Some(cursor)=window.cursor_position() {snapshot.cursor=[cursor.x as f64,cursor.y as f64];}
    }
    let runtime=world.resource::<ResourceRuntime>();
    // Parent represents the native window owner for dependency cleanup. Bevy
    // still owns its platform event loop and renderer, not a fake GL context.
    let parent=runtime.allocate(&[],||{});
    let source=Rc::new(SourceWindow::from_handle(entity.to_bits() as i64,window.title.clone(),
        Arc::new(Mutex::new(Backend(state.clone()))),parent,runtime));
    let text_hook=crate::window_native_characters::Hook::install(crate::editor_clipboard::owner(world.get::<bevy::window::RawHandleWrapper>(entity)));
    if text_hook.is_some() {bevy::log::info!("Installed source Win32 character adapter");}
    world.insert_non_send_resource(LiveWindow {source,entity,state,text_hook:std::cell::RefCell::new(text_hook)});
}

fn modifier_key(key: KeyCode) -> Option<u8> {
    Some(match key {
        KeyCode::ShiftLeft=>1, KeyCode::ShiftRight=>2,
        KeyCode::ControlLeft=>4, KeyCode::ControlRight=>8,
        KeyCode::AltLeft=>16, KeyCode::AltRight=>32,
        KeyCode::SuperLeft=>64, KeyCode::SuperRight=>128,
        _=>return None,
    })
}
fn glfw_modifiers(keys: u8) -> i32 {
    i32::from(keys&3!=0) | (i32::from(keys&12!=0)<<1)
        | (i32::from(keys&48!=0)<<2) | (i32::from(keys&192!=0)<<3)
}
fn glfw_button(button: MouseButton) -> i32 {
    match button {
        MouseButton::Left=>0, MouseButton::Right=>1, MouseButton::Middle=>2,
        MouseButton::Back=>3, MouseButton::Forward=>4, MouseButton::Other(id)=>i32::from(id),
    }
}
fn raw_mouse_metadata(event: &winit::event::WindowEvent, state: &mut State) {
    use winit::event::WindowEvent as W;
    match event {
        W::ModifiersChanged(modifiers) => {
            let mods=modifiers.state();
            state.native_modifiers=i32::from(mods.shift_key()) | (i32::from(mods.control_key())<<1)
                | (i32::from(mods.alt_key())<<2) | (i32::from(mods.super_key())<<3);
        },
        W::Focused(false) => state.native_modifiers=0,
        W::Ime(winit::event::Ime::Commit(text)) => {
            state.native_commits.push_back((text.clone(),state.native_modifiers));
        },
        W::MouseInput {state:action,button,..} => {
            use winit::event::MouseButton as B;
            let button=match button {B::Left=>0,B::Right=>1,B::Middle=>2,B::Back=>3,B::Forward=>4,B::Other(id)=>i32::from(*id)};
            state.native_mouse.push_back((button,i32::from(action.is_pressed()),state.native_modifiers));
        },
        W::KeyboardInput {event,is_synthetic:false,..} => {
            use winit::platform::scancode::PhysicalKeyExtScancode;
            #[cfg(target_os="windows")]
            if let Some(scan)=event.physical_key.to_scancode() {
                record_key(state,bevy::winit::converters::convert_physical_key_code(event.physical_key),
                    scan,event.state.is_pressed(),event.repeat);
            }
        },
        _=>{},
    }
}
fn record_key(state: &mut State, key: KeyCode, scan: u32, pressed: bool, repeat: bool) {
    let action=if !pressed {0} else if repeat {2} else {1};
    state.native_keys.push_back((key,crate::window_keyboard::glfw_scan(scan),action,state.native_modifiers));
}
fn translated(event: &NativeEvent, target: Entity, state: &mut State) -> Option<WindowEvent> {
    use NativeEvent as N;
    match event {
        N::WindowMoved(e) if e.window==target => Some(WindowEvent::Position {xpos:e.position.x,ypos:e.position.y}),
        N::WindowResized(e) if e.window==target => Some(WindowEvent::Size {width:e.width as i32,height:e.height as i32}),
        N::WindowFocused(e) if e.window==target => {
            if !e.focused {state.modifier_keys=0;}
            Some(WindowEvent::Focus {focus:e.focused})
        },
        N::KeyboardInput(e) if e.window==target => {
            // Track transitions in native order, not the final per-frame
            // ButtonInput snapshot. Releasing one side preserves the other.
            if let Some(bit)=modifier_key(e.key_code) {
                if e.state==bevy::input::ButtonState::Pressed {state.modifier_keys|=bit;}
                else {state.modifier_keys&=!bit;}
            }
            let action=if e.state==bevy::input::ButtonState::Released {0} else if e.repeat {2} else {1};
            if state.native_keys.front().is_some_and(|&(key,_,a,_)|key==e.key_code && a==action) {
                let (_,scancode,action,mods)=state.native_keys.pop_front().unwrap();
                if action==0 {state.pressed_scans.remove(&e.key_code);}
                else if crate::window_keyboard::glfw_key(scancode)>=0 {state.pressed_scans.insert(e.key_code,scancode);}
                Some(WindowEvent::Key {key:crate::window_keyboard::glfw_key(scancode),scancode,action,mods})
            } else {None}
        },
        N::KeyboardFocusLost(_) => {state.modifier_keys=0; None},
        N::MouseButtonInput(e) if e.window==target => {
            let button=glfw_button(e.button);
            let action=i32::from(e.state==bevy::input::ButtonState::Pressed);
            if action==1 {state.pressed_mouse.insert(button);} else {state.pressed_mouse.remove(&button);}
            // Correlate each raw packet with exactly one typed event. Preserve
            // the typed stream's ordering relative to cursor/resize callbacks.
            let mods=if state.native_mouse.front().is_some_and(|&(b,a,_)|b==button && a==action) {
                state.native_mouse.pop_front().unwrap().2
            } else {glfw_modifiers(state.modifier_keys)};
            Some(WindowEvent::MouseButton {button,action,mods})
        },
        N::WindowCloseRequested(e) if e.window==target => Some(WindowEvent::Close {}),
        N::CursorMoved(e) if e.window==target => Some(WindowEvent::CursorPos {xpos:e.position.x as f64,ypos:e.position.y as f64}),
        N::CursorEntered(e) if e.window==target => Some(WindowEvent::CursorEnter {enter:true}),
        N::CursorLeft(e) if e.window==target => Some(WindowEvent::CursorEnter {enter:false}),
        N::MouseWheel(e) if e.window==target => Some(WindowEvent::Scroll {x:e.x as f64,y:e.y as f64}),
        N::FileDragAndDrop(bevy::window::FileDragAndDrop::DroppedFile {window,path_buf}) if *window==target =>
            Some(WindowEvent::Drop {data:vec![path_buf.to_string_lossy().into_owned()]}),
        // Physical framebuffer resize is queried separately. Native iconify
        // and refresh callbacks still need their original dispatch adapter.
        _ => None,
    }
}
fn route(mut events: MessageReader<NativeEvent>, live: Option<NonSend<LiveWindow>>,
    windows: Query<(&Window,Option<&bevy::window::RawHandleWrapper>)>, mut raw: MessageReader<bevy::winit::RawWinitWindowEvent>,
    native_windows: Option<NonSend<bevy::winit::WinitWindows>>,
    mut characters: MessageWriter<crate::window_characters::CharacterInput>,
    mut ordered: MessageWriter<OrderedInput>,
    mut reset:MessageWriter<crate::window_characters::ResetShip>) {
    let Some(live)=live else {events.clear(); raw.clear(); return};
    if live.source.freed() {events.clear(); raw.clear(); return;}
    if live.text_hook.borrow().is_none() {
        if let Ok((_,raw_handle))=windows.get(live.entity) {
            let owner=crate::editor_clipboard::owner(raw_handle);
            if owner!=0 {
                *live.text_hook.borrow_mut()=crate::window_native_characters::Hook::install(owner);
                if live.text_hook.borrow().is_some() {bevy::log::info!("Installed source Win32 character adapter");}
            }
        }
    }
    let native_text=live.text_hook.borrow().is_some();
    {
        let mut state=live.state.lock().unwrap();
        state.native_mouse.clear();
        state.native_keys.clear();
        state.native_commits.clear();
        for event in raw.read() {
            if native_windows.as_ref().and_then(|windows|windows.get_window_entity(event.window_id))==Some(live.entity) {
                raw_mouse_metadata(&event.event,&mut state);
            }
        }
    }
    live.source.before_events();
    let frame_screen = live.state.lock().unwrap().screen;
    for event in events.read() {
        if let NativeEvent::Ime(bevy::window::Ime::Commit {window,value})=event {
            if *window==live.entity {
                let mods={let mut state=live.state.lock().unwrap();
                    if state.native_commits.front().is_some_and(|(text,_)|text==value) {
                        state.native_commits.pop_front().unwrap().1
                    } else {glfw_modifiers(state.modifier_keys)}};
                if native_text {
                    // The source IME helper fills GUI input separately from GLFW character callbacks.
                    for c in value.chars() {characters.write(crate::window_characters::CharacterInput(c as u32));}
                } else {crate::window_characters::dispatch(&live.source,value,mods,true,|c| {
                    characters.write(crate::window_characters::CharacterInput(c as u32));
                });}
            }
            continue;
        }
        let translated={let mut state=live.state.lock().unwrap(); translated(event,live.entity,&mut state)};
        if let Some(event)=translated {
            let preceding=live.text_hook.borrow().as_ref().map(|hook|hook.before(&event)).unwrap_or_default();
            for packet in preceding {crate::window_characters::native_packet(&live.source,packet,|c| {characters.write(crate::window_characters::CharacterInput(c));},|| {reset.write(crate::window_characters::ResetShip);});}
            {
                let mut state=live.state.lock().unwrap();
                match &event {
                    WindowEvent::Position {xpos,ypos} => state.position=[*xpos,*ypos],
                    WindowEvent::Size {width,height} => state.screen=[*width,*height],
                    WindowEvent::CursorPos {xpos,ypos} => state.cursor=[*xpos,*ypos],
                    WindowEvent::Close {} => state.close=true,
                    _ => {}
                }
            }
            // No backend lock is held while source callbacks query or mutate it.
            {
                let state = live.state.lock().unwrap();
                ordered.write(OrderedInput {event: event.clone(), cursor: state.cursor,
                    screen: state.screen, frame_screen});
            }
            let lost_focus=matches!(&event,WindowEvent::Focus {focus:false});
            live.source.dispatch(event);
            if lost_focus {
                // GLFW notifies focus first, then releases keys in key-ID order
                // and buttons in button-ID order with mods=0. Bevy's generated
                // key releases later have no raw packets and are not redelivered.
                let (keys,buttons)={
                    let mut state=live.state.lock().unwrap();
                    let mut keys:Vec<_>=state.pressed_scans.drain().map(|(_,scan)|
                        (crate::window_keyboard::glfw_key(scan),scan)).collect();
                    keys.sort_unstable();
                    let buttons=std::mem::take(&mut state.pressed_mouse);
                    (keys,buttons)
                };
                for (key,scancode) in keys {live.source.emit_key(key,scancode,0,0);}
                for button in buttons {
                    live.source.emit_mouse_button(button,0,0);
                    let state = live.state.lock().unwrap();
                    ordered.write(OrderedInput {event: WindowEvent::MouseButton {button,action:0,mods:0},
                        cursor:state.cursor, screen:state.screen, frame_screen});
                }
            }
        }
        if !native_text {
            if let NativeEvent::KeyboardInput(key)=event {
                if key.window==live.entity && key.state==bevy::input::ButtonState::Pressed {
                    let mods=glfw_modifiers(live.state.lock().unwrap().modifier_keys);
                    if mods&6==0 {
                        if let Some(text)=&key.text {for c in text.chars().filter(|&c|crate::window_characters::accepted(c)) {
                            crate::window_characters::native_packet(&live.source,crate::window_native_characters::Packet {codepoint:c as u32,mods,plain:true,composing:false},|c| {characters.write(crate::window_characters::CharacterInput(c));},|| {reset.write(crate::window_characters::ResetShip);});
                        }}
                    }
                }
            }
        }
    }
    let trailing=live.text_hook.borrow().as_ref().map(|hook|hook.finish()).unwrap_or_default();
    for packet in trailing {crate::window_characters::native_packet(&live.source,packet,|c| {characters.write(crate::window_characters::CharacterInput(c));},|| {reset.write(crate::window_characters::ResetShip);});}
    // Bevy exposes physical size through Window, separately from logical
    // WindowResized. Dispatch a physical callback only when it actually changes.
    if let Ok((window,_))=windows.get(live.entity) {
        let size=[window.physical_width() as i32,window.physical_height() as i32];
        let changed={let mut state=live.state.lock().unwrap(); let changed=state.framebuffer!=size;
            state.framebuffer=size; changed};
        if changed {live.source.emit_framebuffer_size(size[0],size[1]);}
    }
    live.source.after_events();
    // Raw/typed batches are delivered together by Winit. Never apply an
    // unmatched packet to a subsequent frame's unrelated click.
    live.state.lock().unwrap().native_mouse.clear();
    live.state.lock().unwrap().native_keys.clear();
    live.state.lock().unwrap().native_commits.clear();
}
fn flush(live: Option<NonSend<LiveWindow>>, mut windows: Query<&mut Window>,
    mut commands: Commands, mut exit: MessageWriter<bevy::app::AppExit>) {
    let Some(live)=live else {return};
    let mut state=live.state.lock().unwrap();
    if state.destroy {commands.entity(live.entity).try_despawn(); state.destroy=false; return;}
    if let Ok(mut window)=windows.get_mut(live.entity) {
        if let Some(title)=state.title_request.take() {window.title=title;}
        if let Some(pos)=state.position_request.take() {window.position=WindowPosition::At(IVec2::from_array(pos));}
    }
    if state.close {exit.write(bevy::app::AppExit::Success);}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key(window: Entity, code: KeyCode, pressed: bool) -> NativeEvent {
        NativeEvent::KeyboardInput(bevy::input::keyboard::KeyboardInput {
            key_code:code, logical_key:bevy::input::keyboard::Key::Unidentified(
                bevy::input::keyboard::NativeKey::Unidentified),
            state:if pressed {bevy::input::ButtonState::Pressed} else {bevy::input::ButtonState::Released},
            text:None, repeat:false, window,
        })
    }
    fn click(window: Entity, button: MouseButton, pressed: bool) -> NativeEvent {
        NativeEvent::MouseButtonInput(bevy::input::mouse::MouseButtonInput {
            window,button,state:if pressed {bevy::input::ButtonState::Pressed} else {bevy::input::ButtonState::Released},
        })
    }
    #[test]
    fn ime_commit_uses_event_local_modifiers_filters_controls_and_ignores_preedit() {
        let mut app=App::new();
        app.add_plugins(MinimalPlugins).add_message::<NativeEvent>().add_message::<bevy::app::AppExit>()
            .add_plugins(WindowBridgePlugin);
        let entity=app.world_mut().spawn((Window::default(),PrimaryWindow)).id(); app.update();
        let id=winit::window::WindowId::dummy(); let mut windows=bevy::winit::WinitWindows::default();
        windows.winit_to_entity.insert(id,entity); app.world_mut().insert_non_send_resource(windows);
        let source=app.world().non_send_resource::<LiveWindow>().source.clone();
        let observed=Rc::new(std::cell::RefCell::new(Vec::new())); let captured=observed.clone();
        source.char_mods_callbacks.borrow_mut().push(Rc::new(move |_,c,mods|captured.borrow_mut().push((c,mods))));
        let text="船🚢\u{7f}".to_string();
        for event in [winit::event::WindowEvent::ModifiersChanged(winit::keyboard::ModifiersState::SHIFT.into()),
            winit::event::WindowEvent::Ime(winit::event::Ime::Commit(text.clone())),
            winit::event::WindowEvent::ModifiersChanged(winit::keyboard::ModifiersState::CONTROL.into())] {
            app.world_mut().write_message(bevy::winit::RawWinitWindowEvent {window_id:id,event});
        }
        app.world_mut().write_message(NativeEvent::Ime(bevy::window::Ime::Preedit {window:entity,value:"uncommitted".into(),cursor:Some((0,0))}));
        app.world_mut().write_message(NativeEvent::Ime(bevy::window::Ime::Commit {window:entity,value:text}));
        app.update();
        assert_eq!(*observed.borrow(),vec![(33337,1),(128674,1)]);
        let mut cursor=bevy::ecs::message::MessageCursor::default();
        assert_eq!(crate::window_characters::read(Some(app.world().resource::<Messages<crate::window_characters::CharacterInput>>()),&mut cursor),"船🚢");
    }
    #[test]
    fn actual_ship_search_accepts_commits_enforces_utf16_capacity_and_drains_unfocused_text() {
        let mut app=App::new();
        app.add_plugins(MinimalPlugins).add_message::<NativeEvent>().add_message::<bevy::app::AppExit>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .init_resource::<ButtonInput<KeyCode>>().init_resource::<ButtonInput<MouseButton>>()
            .insert_resource(crate::Simulation::default()).add_plugins(WindowBridgePlugin)
            .add_systems(Update,crate::select_ship_from_panel);
        let entity=app.world_mut().spawn((Window::default(),PrimaryWindow)).id();
        app.world_mut().spawn((Text2d::new(""),crate::ShipSearchText)); app.update();
        app.world_mut().resource_mut::<crate::Simulation>().ship_search_active=true;
        app.world_mut().write_message(NativeEvent::Ime(bevy::window::Ime::Commit {window:entity,value:"船🚢".into()})); app.update();
        assert_eq!(app.world().resource::<crate::Simulation>().ship_search,"船🚢");
        app.world_mut().resource_mut::<crate::Simulation>().ship_search="a".repeat(254);
        app.world_mut().write_message(NativeEvent::Ime(bevy::window::Ime::Commit {window:entity,value:"🚢".into()})); app.update();
        assert_eq!(app.world().resource::<crate::Simulation>().ship_search.encode_utf16().count(),254);
        app.world_mut().resource_mut::<crate::Simulation>().ship_search_active=false;
        app.world_mut().write_message(NativeEvent::Ime(bevy::window::Ime::Commit {window:entity,value:"ignored".into()})); app.update();
        app.world_mut().resource_mut::<crate::Simulation>().ship_search_active=true; app.update();
        assert_eq!(app.world().resource::<crate::Simulation>().ship_search,"a".repeat(254));
    }
    #[test]
    fn native_keys_preserve_scan_action_modifiers_unknown_keys_and_no_raw_fallback() {
        let target=Entity::from_bits(1); let mut state=State::default();
        state.native_modifiers=5;
        record_key(&mut state,KeyCode::ControlRight,0xe01d,true,false);
        let event=translated(&key(target,KeyCode::ControlRight,true),target,&mut state).unwrap();
        let WindowEvent::Key {key,scancode,action,mods}=event else {panic!()};
        assert_eq!((key,scancode,action,mods),(345,0x11d,1,5));
        record_key(&mut state,KeyCode::ControlRight,0xe01d,true,true);
        let mut repeat=super::tests::key(target,KeyCode::ControlRight,true);
        if let NativeEvent::KeyboardInput(e)=&mut repeat {e.repeat=true;}
        let Some(WindowEvent::Key {action:2,..})=translated(&repeat,target,&mut state) else {panic!()};
        record_key(&mut state,KeyCode::ControlRight,0xe01d,false,false);
        let Some(WindowEvent::Key {action:0,..})=translated(&super::tests::key(target,KeyCode::ControlRight,false),target,&mut state) else {panic!()};
        assert!(state.pressed_scans.is_empty());
        assert!(translated(&super::tests::key(target,KeyCode::KeyA,true),target,&mut state).is_none());
        record_key(&mut state,KeyCode::KeyA,0x79,true,false);
        let Some(WindowEvent::Key {key:-1,scancode:0x79,..})=translated(&super::tests::key(target,KeyCode::KeyA,true),target,&mut state) else {panic!()};
        assert!(state.pressed_scans.is_empty());
    }
    #[test]
    fn focus_callback_precedes_sorted_source_key_and_mouse_releases_once() {
        let mut app=App::new();
        app.add_plugins(MinimalPlugins).add_message::<NativeEvent>().add_message::<bevy::app::AppExit>()
            .add_plugins(WindowBridgePlugin);
        let entity=app.world_mut().spawn((Window::default(),PrimaryWindow)).id(); app.update();
        let live=app.world().non_send_resource::<LiveWindow>(); let source=live.source.clone();
        {let mut state=live.state.lock().unwrap(); state.pressed_scans.insert(KeyCode::KeyB,0x30);
            state.pressed_scans.insert(KeyCode::KeyA,0x1e); state.pressed_mouse.extend([1,0]);}
        let observed=Rc::new(std::cell::RefCell::new(Vec::new()));
        let captured=observed.clone(); source.focus_callbacks.borrow_mut().push(Rc::new(move |_,_|captured.borrow_mut().push("focus".into())));
        let captured=observed.clone(); source.key_callbacks.borrow_mut().push(Rc::new(move |_,key,scan,action,mods|
            captured.borrow_mut().push(format!("key {key} {scan} {action} {mods}"))));
        let captured=observed.clone(); source.mouse_button_callbacks.borrow_mut().push(Rc::new(move |_,button,action,mods|
            captured.borrow_mut().push(format!("mouse {button} {action} {mods}"))));
        app.world_mut().write_message(NativeEvent::WindowFocused(bevy::window::WindowFocused {window:entity,focused:false}));
        app.world_mut().write_message(key(entity,KeyCode::KeyA,false));
        app.world_mut().write_message(key(entity,KeyCode::KeyB,false));
        app.update();
        assert_eq!(*observed.borrow(),vec!["focus","key 65 30 0 0","key 66 48 0 0","mouse 0 0 0","mouse 1 0 0"]);
    }
    #[test]
    fn raw_platform_masks_override_key_reconstruction_once_per_click_in_native_order() {
        use winit::event::WindowEvent as W;
        let mut app=App::new();
        app.add_plugins(MinimalPlugins).add_message::<NativeEvent>().add_message::<bevy::app::AppExit>()
            .add_plugins(WindowBridgePlugin);
        let entity=app.world_mut().spawn((Window::default(),PrimaryWindow)).id(); app.update();
        let id=winit::window::WindowId::dummy();
        let mut windows=bevy::winit::WinitWindows::default();
        windows.winit_to_entity.insert(id,entity);
        let foreign_id=winit::window::WindowId::from(99);
        let foreign_entity=app.world_mut().spawn_empty().id();
        windows.winit_to_entity.insert(foreign_id,foreign_entity);
        app.world_mut().insert_non_send_resource(windows);
        let source=app.world().non_send_resource::<LiveWindow>().source.clone();
        let observed=Rc::new(std::cell::RefCell::new(Vec::new())); let captured=observed.clone();
        source.mouse_button_callbacks.borrow_mut().push(Rc::new(move |_,button,action,mods|captured.borrow_mut().push((button,action,mods))));
        // No Shift/Control keyboard transitions: masks must come from the OS.
        for event in [
            W::ModifiersChanged(winit::keyboard::ModifiersState::SHIFT.into()),
            W::MouseInput {device_id:winit::event::DeviceId::dummy(),state:winit::event::ElementState::Pressed,button:winit::event::MouseButton::Left},
            W::ModifiersChanged(winit::keyboard::ModifiersState::CONTROL.into()),
            W::MouseInput {device_id:winit::event::DeviceId::dummy(),state:winit::event::ElementState::Released,button:winit::event::MouseButton::Left},
        ] {app.world_mut().write_message(bevy::winit::RawWinitWindowEvent {window_id:id,event});}
        app.world_mut().write_message(click(entity,MouseButton::Left,true));
        app.world_mut().write_message(click(entity,MouseButton::Left,false));
        app.world_mut().write_message(bevy::winit::RawWinitWindowEvent {window_id:foreign_id,
            event:W::ModifiersChanged(winit::keyboard::ModifiersState::SUPER.into())});
        app.update();
        assert_eq!(*observed.borrow(),vec![(0,1,1),(0,0,2)]);
        // The mask survives between batches; raw packets never dispatch alone.
        app.world_mut().write_message(bevy::winit::RawWinitWindowEvent {window_id:id,event:
            W::MouseInput {device_id:winit::event::DeviceId::dummy(),state:winit::event::ElementState::Pressed,button:winit::event::MouseButton::Right}});
        app.world_mut().write_message(click(entity,MouseButton::Right,true)); app.update();
        assert_eq!(*observed.borrow(),vec![(0,1,1),(0,0,2),(1,1,2)]);
        app.world_mut().write_message(bevy::winit::RawWinitWindowEvent {window_id:id,event:
            W::MouseInput {device_id:winit::event::DeviceId::dummy(),state:winit::event::ElementState::Released,button:winit::event::MouseButton::Right}});
        app.update();
        assert_eq!(observed.borrow().len(),3); // raw metadata alone is not input
        app.world_mut().write_message(click(entity,MouseButton::Right,false)); app.update();
        assert_eq!(observed.borrow().last(),Some(&(1,0,0))); // stale packet retired
    }
    #[test]
    fn raw_focus_loss_clears_masks_and_unmatched_packets_do_not_steal_other_events() {
        use winit::event::WindowEvent as W;
        let target=Entity::from_bits(1); let mut state=State::default();
        raw_mouse_metadata(&W::ModifiersChanged((winit::keyboard::ModifiersState::ALT|winit::keyboard::ModifiersState::SUPER).into()),&mut state);
        raw_mouse_metadata(&W::Focused(false),&mut state);
        raw_mouse_metadata(&W::MouseInput {device_id:winit::event::DeviceId::dummy(),state:winit::event::ElementState::Pressed,button:winit::event::MouseButton::Right},&mut state);
        let Some(WindowEvent::MouseButton {mods,..})=translated(&click(target,MouseButton::Left,true),target,&mut state) else {panic!()};
        assert_eq!(mods,0); assert_eq!(state.native_mouse.len(),1);
        let Some(WindowEvent::MouseButton {mods,..})=translated(&click(target,MouseButton::Right,true),target,&mut state) else {panic!()};
        assert_eq!(mods,0); assert!(state.native_mouse.is_empty());
    }
    #[test]
    fn ordered_mouse_callbacks_keep_each_click_modifier_state_and_both_key_sides() {
        let mut app=App::new();
        app.add_plugins(MinimalPlugins).add_message::<NativeEvent>().add_message::<bevy::app::AppExit>()
            .add_plugins(WindowBridgePlugin);
        let entity=app.world_mut().spawn((Window::default(),PrimaryWindow)).id(); app.update();
        let source=app.world().non_send_resource::<LiveWindow>().source.clone();
        let observed=Rc::new(std::cell::RefCell::new(Vec::new())); let captured=observed.clone();
        source.mouse_button_callbacks.borrow_mut().push(Rc::new(move |_,button,action,mods| {
            captured.borrow_mut().push((button,action,mods));
        }));
        for event in [key(entity,KeyCode::ShiftLeft,true),click(entity,MouseButton::Left,true),
            key(entity,KeyCode::ShiftRight,true),key(entity,KeyCode::ShiftLeft,false),
            click(entity,MouseButton::Left,false),key(entity,KeyCode::ControlLeft,true),
            click(entity,MouseButton::Right,true),key(entity,KeyCode::ShiftRight,false),
            click(entity,MouseButton::Right,false),key(entity,KeyCode::ControlLeft,false),
            click(entity,MouseButton::Middle,true)] {app.world_mut().write_message(event);}
        app.update();
        assert_eq!(*observed.borrow(),vec![(0,1,1),(0,0,1),(1,1,3),(1,0,2),(2,1,0)]);
    }
    #[test]
    fn foreign_keys_do_not_change_modifiers_and_focus_loss_clears_them() {
        let target=Entity::from_bits(1); let foreign=Entity::from_bits(2);
        let mut state=State::default();
        translated(&key(target,KeyCode::AltLeft,true),target,&mut state);
        translated(&key(target,KeyCode::SuperRight,true),target,&mut state);
        translated(&key(foreign,KeyCode::ShiftLeft,true),target,&mut state);
        let Some(WindowEvent::MouseButton {button,action,mods})=
            translated(&click(target,MouseButton::Back,true),target,&mut state) else {panic!()};
        assert_eq!((button,action,mods),(3,1,12));
        translated(&NativeEvent::WindowFocused(bevy::window::WindowFocused {window:target,focused:false}),target,&mut state);
        let Some(WindowEvent::MouseButton {button,mods,..})=
            translated(&click(target,MouseButton::Forward,false),target,&mut state) else {panic!()};
        assert_eq!((button,mods),(4,0));
        assert_eq!(glfw_button(MouseButton::Other(u16::MAX)),65535);
    }
    #[test]
    fn live_callbacks_query_event_local_state_and_mutate_native_window() {
        let mut app=App::new();
        app.add_plugins(MinimalPlugins).add_message::<NativeEvent>().add_message::<bevy::app::AppExit>()
            .add_plugins(WindowBridgePlugin);
        let entity=app.world_mut().spawn((Window::default(),PrimaryWindow)).id();
        app.update();
        let observations=Rc::new(std::cell::RefCell::new(Vec::new()));
        let captured=observations.clone();
        let source=app.world().non_send_resource::<LiveWindow>().source.clone();
        source.cursor_pos_callbacks.borrow_mut().push(Rc::new(move |window,x,y| {
            captured.borrow_mut().push((x,y,window.mouse_position()));
            window.set_title(format!("cursor {x}"));
            window.set_position([12,34]);
        }));
        for x in [10.0,20.0] {app.world_mut().write_message(NativeEvent::CursorMoved(bevy::window::CursorMoved {
            window:entity,position:Vec2::new(x,7.0),delta:None}));}
        app.update();
        assert_eq!(*observations.borrow(),vec![(10.0,7.0,[10.0,7.0]),(20.0,7.0,[20.0,7.0])]);
        let window=app.world().get::<Window>(entity).unwrap();
        assert_eq!(window.title,"cursor 20");
        assert_eq!(window.position,WindowPosition::At(IVec2::new(12,34)));
        assert_eq!(source.position(),[0,0]); // request awaits the actual OS event
    }
    #[test]
    fn native_physical_resize_is_distinct_from_logical_resize_and_foreign_windows() {
        let mut app=App::new();
        app.add_plugins(MinimalPlugins).add_message::<NativeEvent>().add_message::<bevy::app::AppExit>()
            .add_plugins(WindowBridgePlugin);
        let entity=app.world_mut().spawn((Window::default(),PrimaryWindow)).id(); app.update();
        let source=app.world().non_send_resource::<LiveWindow>().source.clone();
        let observed=Rc::new(std::cell::RefCell::new(Vec::new())); let captured=observed.clone();
        source.framebuffer_size_callbacks.borrow_mut().push(Rc::new(move |window,w,h| {
            captured.borrow_mut().push((w,h,window.framebuffer_size()));
        }));
        app.world_mut().get_mut::<Window>(entity).unwrap().resolution.set_physical_resolution(900,600);
        let foreign=app.world_mut().spawn_empty().id();
        app.world_mut().write_message(NativeEvent::WindowResized(bevy::window::WindowResized {window:foreign,width:1.0,height:2.0}));
        app.update(); app.update();
        assert_eq!(*observed.borrow(),vec![(900,600,[900,600])]);
        assert_ne!(source.screen_size(),[1,2]);
    }
    #[test]
    fn actual_camera_consumes_source_resize_and_zero_dimension_gate() {
        let mut app=App::new();
        app.add_plugins(MinimalPlugins).add_message::<NativeEvent>().add_message::<bevy::app::AppExit>()
            .add_message::<bevy::input::mouse::MouseWheel>().add_message::<bevy::input::keyboard::KeyboardInput>()
            .init_resource::<ButtonInput<KeyCode>>().init_resource::<ButtonInput<MouseButton>>()
            .insert_resource(crate::Simulation::default()).init_resource::<crate::camera_control::CameraControlState>()
            .add_plugins(WindowBridgePlugin).add_systems(Update,crate::camera_control::handle_camera_control);
        let entity=app.world_mut().spawn((Window::default(),PrimaryWindow)).id();
        let camera=app.world_mut().spawn((crate::WorldCamera,
            Projection::Orthographic(OrthographicProjection::default_2d()),Transform::default())).id();
        app.update();
        app.world_mut().write_message(NativeEvent::WindowResized(bevy::window::WindowResized {
            window:entity,width:800.0,height:400.0}));
        app.update();
        let Projection::Orthographic(projection)=app.world().get::<Projection>(camera).unwrap() else {panic!()};
        let prior=projection.area;
        assert_eq!(prior.size(),Vec2::new(800.0/16.0,400.0/16.0));
        app.world_mut().write_message(NativeEvent::WindowResized(bevy::window::WindowResized {
            window:entity,width:0.0,height:0.0}));
        app.update();
        let Projection::Orthographic(projection)=app.world().get::<Projection>(camera).unwrap() else {panic!()};
        assert_eq!(projection.area,prior);
    }
    #[test]
    fn source_close_callback_can_cancel_native_close_and_cleanup_destroys_owner() {
        let mut app=App::new();
        app.add_plugins(MinimalPlugins).add_message::<NativeEvent>().add_message::<bevy::app::AppExit>()
            .add_plugins(WindowBridgePlugin);
        let entity=app.world_mut().spawn((Window::default(),PrimaryWindow)).id(); app.update();
        let source=app.world().non_send_resource::<LiveWindow>().source.clone();
        source.close_callbacks.borrow_mut().push(Rc::new(|window| window.set_should_close(false)));
        app.world_mut().write_message(NativeEvent::WindowCloseRequested(bevy::window::WindowCloseRequested {window:entity}));
        app.update();
        assert!(!source.should_close());
        assert!(app.world().resource::<Messages<bevy::app::AppExit>>().is_empty());
        assert!(app.world().get::<Window>(entity).is_some());
        source.close();
        app.world().resource::<ResourceRuntime>().run_main();
        app.update();
        assert!(app.world().get::<Window>(entity).is_none());
    }
    #[test]
    fn ordinary_native_key_text_has_one_search_insertion_and_shift_r_reset_ignores_text_capture() {
        let mut app=App::new();
        app.add_plugins(MinimalPlugins).init_resource::<crate::Simulation>()
            .init_resource::<ButtonInput<MouseButton>>().init_resource::<ButtonInput<KeyCode>>()
            .add_message::<bevy::input::keyboard::KeyboardInput>().add_message::<NativeEvent>()
            .add_message::<bevy::app::AppExit>().add_plugins(WindowBridgePlugin)
            .add_systems(Update,crate::select_ship_from_panel);
        let target=app.world_mut().spawn((Window::default(),PrimaryWindow)).id();app.update();
        app.world_mut().resource_mut::<crate::Simulation>().ship_search_active=true;
        let mut a=key(target,KeyCode::KeyA,true);
        if let NativeEvent::KeyboardInput(event)=&mut a {event.text=Some("a".into());}
        if let NativeEvent::KeyboardInput(event)=&a {app.world_mut().write_message(event.clone());}
        app.world_mut().write_message(a);app.update();
        assert_eq!(app.world().resource::<crate::Simulation>().ship_search,"a");
        let mut plain_r=key(target,KeyCode::KeyR,true);
        if let NativeEvent::KeyboardInput(event)=&mut plain_r {event.text=Some("r".into());}
        if let NativeEvent::KeyboardInput(event)=&plain_r {app.world_mut().write_message(event.clone());}
        app.world_mut().write_message(plain_r);app.update();
        let mut resets=bevy::ecs::message::MessageCursor::<crate::window_characters::ResetShip>::default();
        assert_eq!(resets.read(app.world().resource::<Messages<crate::window_characters::ResetShip>>()).count(),0,"Plain R is not a source reset shortcut");
        let mut r=key(target,KeyCode::KeyR,true);
        if let NativeEvent::KeyboardInput(event)=&mut r {event.text=Some("R".into());}
        if let NativeEvent::KeyboardInput(event)=&r {app.world_mut().write_message(event.clone());}
        app.world_mut().write_message(key(target,KeyCode::ShiftLeft,true));app.world_mut().write_message(r);app.update();
        assert_eq!(app.world().resource::<crate::Simulation>().ship_search,"arR");
        let mut cursor=bevy::ecs::message::MessageCursor::<crate::window_characters::ResetShip>::default();
        assert_eq!(cursor.read(app.world().resource::<Messages<crate::window_characters::ResetShip>>()).count(),1);
        app.update();assert_eq!(app.world().resource::<crate::Simulation>().ship_search,"arR");
    }
    #[test]
    fn native_search_keeps_isolated_utf16_units_and_never_matches_lossy_replacement() {
        let mut app=App::new();app.init_resource::<crate::Simulation>()
            .init_resource::<ButtonInput<MouseButton>>().init_resource::<ButtonInput<KeyCode>>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_message::<crate::window_characters::CharacterInput>().add_systems(Update,crate::select_ship_from_panel);
        app.world_mut().spawn(Window::default());app.world_mut().resource_mut::<crate::Simulation>().ship_search_active=true;
        for point in [0xd83d,0xde80,0xd83d] {app.world_mut().write_message(crate::window_characters::CharacterInput(point));}
        app.update();let simulation=app.world().resource::<crate::Simulation>();
        assert_eq!(simulation.ship_search_units,vec![0xd83d,0xde80,0xd83d]);
        assert_eq!(simulation.ship_search,"🚀�");
        assert!(!crate::java_string::java_contains_ignore_case_units(&"🚀�".encode_utf16().collect::<Vec<_>>(),&simulation.ship_search_units));
    }

}
