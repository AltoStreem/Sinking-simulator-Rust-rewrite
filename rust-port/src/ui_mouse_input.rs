//! Partial imgui.static.MiscKt.updateMouseInputs translation for native widgets.
//! Source IO defaults and double-click pairing, independent of widget focus.
use bevy::prelude::*;
#[derive(Clone,Copy,Debug)]
pub(crate) struct Button {
    pub clicked:bool,pub released:bool,pub double_clicked:bool,
    pub down_was_double_click:bool,pub duration:f32,pub duration_prev:f32,
    pub clicked_time:f64,pub clicked_pos:Vec2,
    pub drag_max_abs:Vec2,pub drag_max_squared:f32,
}
impl Default for Button {
    fn default()->Self {Self {clicked:false,released:false,double_clicked:false,
        down_was_double_click:false,duration:-1.0,duration_prev:-1.0,
        clicked_time:0.0,clicked_pos:Vec2::ZERO,drag_max_abs:Vec2::ZERO,drag_max_squared:0.0}}
}
pub(crate) struct MouseInput {
    pub time:f64,pub position:Vec2,pub previous:Vec2,pub delta:Vec2,
    pub last_valid:Vec2,pub buttons:[Button;5],
    pub double_click_time:f32,pub double_click_max_dist:f32,
}
impl Default for MouseInput {
    fn default()->Self {Self {time:0.0,position:Vec2::splat(-f32::MAX),previous:Vec2::splat(-f32::MAX),
        delta:Vec2::ZERO,last_valid:Vec2::ZERO,buttons:[Button::default();5],
        double_click_time:0.3,double_click_max_dist:6.0}}
}
fn valid(position:Vec2)->bool {position.x> -256000.0 && position.y> -256000.0}
fn squared(vector:Vec2)->f32 {vector.x*vector.x+vector.y*vector.y}
impl MouseInput {
    pub fn update(&mut self,delta_time:f32,position:Option<Vec2>,down:[bool;5]) {
        self.time+=f64::from(delta_time);
        self.position=position.unwrap_or(Vec2::splat(-f32::MAX));
        if valid(self.position) {self.last_valid=Vec2::new(crate::ui_numeric::floor(self.position.x),crate::ui_numeric::floor(self.position.y));self.position=self.last_valid;}
        self.delta=if valid(self.position)&&valid(self.previous) {self.position-self.previous}else {Vec2::ZERO};
        self.previous=self.position;
        for (button,down) in self.buttons.iter_mut().zip(down) {
            button.clicked=down&&button.duration<0.0;
            button.released=!down&&button.duration>=0.0;
            button.duration_prev=button.duration;
            button.duration=if down {if button.duration<0.0 {0.0}else {button.duration+delta_time}}else {-1.0};
            button.double_clicked=false;
            if button.clicked {
                if self.time-button.clicked_time<f64::from(self.double_click_time) {
                    let offset=if valid(self.position) {self.position-button.clicked_pos}else {Vec2::ZERO};
                    button.double_clicked=squared(offset)<self.double_click_max_dist*self.double_click_max_dist;
                    button.clicked_time=-f64::MAX;
                }else {button.clicked_time=self.time;}
                button.clicked_pos=self.position;
                button.down_was_double_click=button.double_clicked;
                button.drag_max_abs=Vec2::ZERO;button.drag_max_squared=0.0;
            }else if down {
                let offset=if valid(self.position) {self.position-button.clicked_pos}else {Vec2::ZERO};
                button.drag_max_squared=button.drag_max_squared.max(squared(offset));
                button.drag_max_abs=button.drag_max_abs.max(offset.abs());
                let offset=self.position-button.clicked_pos;
                button.drag_max_abs=button.drag_max_abs.max(offset.abs());
                button.drag_max_squared=button.drag_max_squared.max(squared(offset));
            }
            if !down&&!button.released {button.down_was_double_click=false;}
        }
    }
}
#[cfg(test)] mod tests {
    use super::*;
    fn frame(m:&mut MouseInput,dt:f32,pos:Option<Vec2>,down:bool) {m.update(dt,pos,[down,false,false,false,false]);}
    #[test] fn source_pairs_consume_far_click_and_third_click_starts_new_pair() {
        let mut m=MouseInput::default();let p=Some(Vec2::new(20.9,30.4));
        frame(&mut m,0.4,p,true);assert!(m.buttons[0].clicked);assert!(!m.buttons[0].double_clicked);
        assert_eq!(m.position,Vec2::new(20.0,30.0));
        frame(&mut m,0.01,p,false);frame(&mut m,0.01,p,true);
        assert!(m.buttons[0].double_clicked);assert_eq!(m.buttons[0].clicked_time,-f64::MAX);
        frame(&mut m,0.01,p,false);frame(&mut m,0.01,p,true);assert!(!m.buttons[0].double_clicked);
        frame(&mut m,0.01,p,false);frame(&mut m,0.01,Some(Vec2::new(26.0,30.0)),true);
        assert!(!m.buttons[0].double_clicked);assert_eq!(m.buttons[0].clicked_time,-f64::MAX);
    }
    #[test] fn source_hold_release_invalid_pointer_and_strict_time_limit() {
        let mut m=MouseInput::default();let p=Some(Vec2::new(20.0,30.0));
        frame(&mut m,0.4,p,true);frame(&mut m,0.1,Some(Vec2::new(23.0,34.0)),true);
        assert!(!m.buttons[0].clicked);assert_eq!(m.buttons[0].drag_max_squared,25.0);
        frame(&mut m,0.0,p,false);frame(&mut m,0.3,p,true);assert!(!m.buttons[0].double_clicked);
        frame(&mut m,0.01,None,false);assert_eq!(m.delta,Vec2::ZERO);
        assert!(m.buttons[0].released);frame(&mut m,0.01,None,false);
        assert!(!m.buttons[0].down_was_double_click);
    }
    #[test] fn original_jvm_mouse_pair_snapshots_match_flags_and_click_times() {
        let mut m=MouseInput::default();
        for (time,down,x,expected,clicked_time) in [
            (0.4,true,20.9,[true,false,false,false],0.4),
            (0.41,false,20.9,[false,true,false,false],0.4),
            (0.42,true,20.9,[true,false,true,true],-f64::MAX),
            (0.43,false,20.9,[false,true,false,true],-f64::MAX),
            (0.44,true,20.9,[true,false,false,false],0.44),
            (0.45,false,20.9,[false,true,false,false],0.44),
            (0.46,true,26.0,[true,false,false,false],-f64::MAX),
        ] {
            // Original probe fixes Context.time directly; keep identical inputs.
            m.time=time;m.update(0.0,Some(Vec2::new(x,30.4)),[down,false,false,false,false]);
            let b=m.buttons[0];
            assert_eq!([b.clicked,b.released,b.double_clicked,b.down_was_double_click],expected);
            assert_eq!(b.clicked_time,clicked_time);
        }
        let mut m=MouseInput::default();m.update(0.3,Some(Vec2::ZERO),[true,false,false,false,false]);
        assert!(!m.buttons[0].double_clicked); // Equality is outside the source time window.
        assert!(!valid(Vec2::new(-256000.0,0.0)));
    }
    #[test] fn source_pointer_floor_truncates_negative_coordinates_toward_zero() {
        let mut m=MouseInput::default();frame(&mut m,0.4,Some(Vec2::new(-1.9,-0.9)),false);
        assert_eq!(m.position,Vec2::new(-1.0,0.0));assert_eq!(m.position.y.to_bits(),0);
        frame(&mut m,0.1,Some(Vec2::new(-2.9,-0.9)),false);assert_eq!(m.delta,Vec2::new(-1.0,0.0));
    }
}
