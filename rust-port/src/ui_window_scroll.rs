//! Bundled Window scroll targets and calcNextScrollFromScrollTargetAndClamp.
//! Native windows supply metrics; child sizing and shared Context remain partial.
use bevy::prelude::*;
#[derive(Debug,Clone)]
pub(crate) struct WindowScroll {
    pub scroll:Vec2,pub maximum:Vec2,pub target:Vec2,pub center_ratio:Vec2,
    pub size_full:Vec2,pub scrollbar_sizes:Vec2,pub content_size:Vec2,
    pub padding:Vec2,pub item_spacing:Vec2,pub decoration_up_height:f32,
    pub collapsed:bool,pub skip_items:bool,
}
impl Default for WindowScroll {
    fn default()->Self {Self {scroll:Vec2::ZERO,maximum:Vec2::ZERO,target:Vec2::splat(f32::MAX),center_ratio:Vec2::ZERO,
        size_full:Vec2::ZERO,scrollbar_sizes:Vec2::ZERO,content_size:Vec2::ZERO,padding:Vec2::ZERO,
        item_spacing:Vec2::ZERO,decoration_up_height:0.0,collapsed:false,skip_items:false}}
}
impl WindowScroll {
    pub fn set_scroll_x(&mut self,value:f32) {self.target.x=value;self.center_ratio.x=0.0;}
    pub fn set_scroll_y(&mut self,value:f32) {self.target.y=value;self.center_ratio.y=0.0;}
    pub fn set_scroll_from_pos_x(&mut self,local:f32,ratio:f32,assertions:bool)->Result<(),&'static str> {
        if assertions&&!(0.0..=1.0).contains(&ratio) {return Err("AssertionError");}
        self.target.x=crate::ui_numeric::floor(local+self.scroll.x);self.center_ratio.x=ratio;Ok(())
    }
    /// Source setScrollFromPosY writes the CURRENT Context window, not receiver.
    /// Invoke this operation on that resolved current window.
    pub fn set_current_scroll_from_pos_y(&mut self,local:f32,ratio:f32,assertions:bool)->Result<(),&'static str> {
        if assertions&&!(0.0..=1.0).contains(&ratio) {return Err("AssertionError");}
        let local=local-self.decoration_up_height;
        self.target.y=crate::ui_numeric::floor(local+self.scroll.y);self.center_ratio.y=ratio;Ok(())
    }
    pub fn next(&self,snap:bool)->Vec2 {
        let mut result=self.scroll;
        for axis in 0..2 {
            let ratio=self.center_ratio[axis];let mut target=self.target[axis];
            if target<f32::MAX {
                if snap&&ratio<=0.0&&target<=self.padding[axis] {target=0.0;}
                else if axis==0&&snap&&ratio>=1.0&&target>=self.content_size[axis]+self.padding[axis]+self.item_spacing[axis] {
                    target=self.content_size[axis]+self.padding[axis]*2.0;
                }
                if axis==1&&snap&&ratio>=1.0&&target>=self.content_size[axis]+self.padding[axis]+self.item_spacing[axis] {
                    target=self.content_size[axis]+self.padding[axis]*2.0;
                }
                let decoration=if axis==1 {self.decoration_up_height}else {0.0};
                // Preserve source X subtraction grouping; Y also subtracts decorations.
                let available=self.size_full[axis]-self.scrollbar_sizes[axis]-decoration;
                result[axis]=target-ratio*available;
            }
        }
        result=Vec2::new(crate::ui_numeric::max(result.x,0.0),crate::ui_numeric::max(result.y,0.0));
        if !self.collapsed&&!self.skip_items {result=Vec2::new(crate::ui_numeric::min(result.x,self.maximum.x),crate::ui_numeric::min(result.y,self.maximum.y));}
        result
    }
    /// Window.begin applies then resets targets; next itself does not mutate.
    pub fn begin_apply(&mut self) {self.scroll=self.next(true);self.target=Vec2::splat(f32::MAX);}
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn targets_are_deferred_clamped_and_reset_only_by_begin() {
        let mut s=WindowScroll {maximum:Vec2::new(100.0,200.0),scroll:Vec2::new(7.0,8.0),..default()};
        s.set_scroll_y(400.0);assert_eq!(s.scroll.y,8.0);assert_eq!(s.next(true),Vec2::new(7.0,200.0));
        assert_eq!(s.target.y,400.0);s.begin_apply();assert_eq!(s.target,Vec2::splat(f32::MAX));
        s.collapsed=true;s.set_scroll_y(400.0);assert_eq!(s.next(true).y,400.0);
        s.collapsed=false;s.skip_items=true;assert_eq!(s.next(true).y,400.0);
    }
    #[test] fn position_targets_preserve_integer_truncation_and_current_window_decoration() {
        let mut s=WindowScroll {scroll:Vec2::new(0.0,2.0),decoration_up_height:18.0,..default()};
        s.set_scroll_from_pos_x(-1.9,0.5,false).unwrap();assert_eq!(s.target.x,-1.0);
        s.set_current_scroll_from_pos_y(15.1,0.5,false).unwrap();assert_eq!(s.target.y,0.0);
        assert_eq!(s.set_scroll_from_pos_x(0.0,2.0,true),Err("AssertionError"));
    }
    #[test] fn original_jvm_scroll_snap_center_skip_and_nan_snapshots_match() {
        let mut s=WindowScroll {scroll:Vec2::new(7.0,8.0),maximum:Vec2::new(100.0,200.0),size_full:Vec2::new(120.0,144.0),
            scrollbar_sizes:Vec2::new(14.0,0.0),padding:Vec2::new(4.0,3.0),content_size:Vec2::new(250.0,400.0),
            item_spacing:Vec2::new(8.0,4.0),..default()};
        s.set_scroll_y(400.0);assert_eq!(s.next(true),Vec2::new(7.0,200.0));
        s.collapsed=true;assert_eq!(s.next(true),Vec2::new(7.0,400.0));s.collapsed=false;s.skip_items=true;
        assert_eq!(s.next(true),Vec2::new(7.0,400.0));s.skip_items=false;
        s.target=Vec2::splat(100.0);s.center_ratio=Vec2::splat(0.5);assert_eq!(s.next(true),Vec2::new(47.0,28.0));
        s.target=Vec2::new(4.0,3.0);s.center_ratio=Vec2::ZERO;assert_eq!(s.next(true),Vec2::ZERO);
        s.maximum=Vec2::splat(500.0);s.target=Vec2::new(300.0,450.0);s.center_ratio=Vec2::ONE;
        assert_eq!(s.next(true),Vec2::new(152.0,262.0));assert_eq!(s.next(false),Vec2::new(194.0,306.0));
        s.scroll.x=f32::NAN;s.target=Vec2::splat(f32::MAX);assert!(s.next(true).x.is_nan());assert_eq!(s.next(true).y,8.0);
    }
}
