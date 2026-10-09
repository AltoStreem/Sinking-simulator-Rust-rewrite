//! Partial bundled imgui.internal.classes.TextEditState translation.
//! UTF-16 keyboard/mouse editing with source glyph layout and bounded undo.
//! Full widget activation, scrolling and dynamic font/DPI parity remain pending.
//! Bounded source undo records are retained on every insertion/deletion.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Key { Left, Right, WordLeft, WordRight, TextStart, TextEnd, LineStart, LineEnd, Backspace, Delete, Undo, Redo, Up, Down }
#[derive(Debug, Default)]
pub(crate) struct TextEditState {
    pub text: Vec<u16>, pub cursor: usize, pub select_start: usize, pub select_end: usize,
    single_line: bool, initialized: bool,
    pub undo: crate::text_edit_undo::UndoState,
    pub buffer: crate::text_edit_undo::EditBuffer,
    pub error: Option<&'static str>,
    published: Vec<u16>,
    initial: Vec<u16>,
    pub has_preferred_x: bool, pub preferred_x: f32,
    pub cursor_anim: f32, pub selected_all_mouse_lock: bool,
    pub cursor_follow:bool,pub scroll_x:f32,pub scroll_y:f32,
    font_height:Option<f32>,
}
impl TextEditState {
    pub(crate) fn font_height(&self)->f32 {self.font_height.unwrap_or(18.0)}
    /// Preserve source pixel scroll and preferred-X when the renderer projection changes.
    pub(crate) fn set_font_height(&mut self,height:f32) {
        let old=self.font_height();
        if old!=height {let ratio=height/old;self.scroll_x*=ratio;self.scroll_y*=ratio;self.preferred_x*=ratio;}
        self.font_height=Some(height);
    }

    pub fn begin_focus(&mut self,published:&[u16],single_line:bool) {
        self.synchronize(published,single_line);
        self.initial=published.to_vec();self.cursor_anim=-0.3;
    }
    pub fn cancel_units(&self)->Vec<u16> {self.initial.clone()}

    pub fn synchronize(&mut self, published: &[u16], single_line: bool) {
        if !self.initialized || self.published!=published {
            self.undo.clear();self.error=None;self.has_preferred_x=false;self.preferred_x=0.0;self.buffer=Default::default();
            let n=published.len().min(255);
            self.buffer.units[..n].copy_from_slice(&published[..n]);
            self.buffer.len_w=n as isize;self.buffer.len_a=n as isize;
            self.refresh_text();self.published=published[..n].to_vec();
            self.cursor=n;self.clear_selection();self.initialized=true;
            self.scroll_x=0.0;self.cursor_follow=false; // Vertical scroll belongs to the child window.
        }
        self.single_line=single_line;
    }
    fn refresh_text(&mut self) {
        let n=self.buffer.len_w.max(0) as usize;
        self.text=self.buffer.units[..n.min(self.buffer.units.len())].to_vec();
    }
    // Original strncpy copies curLenA+1 units without adding a terminator.
    // Retain old user-buffer tail when source undo leaves physical text longer.
    pub fn publish(&mut self)->Option<Vec<u16>> {
        if self.error.is_some() {return None;}
        let mut output=self.published.clone();output.resize(256,0);
        let n=(self.buffer.len_a+1).max(0).min(256) as usize;
        output[..n].copy_from_slice(&self.buffer.units[..n]);
        let end=output.iter().position(|&unit|unit==0).unwrap_or(256).min(255);
        self.published=output[..end].to_vec();Some(self.published.clone())
    }
    fn delete(&mut self,start:usize,count:usize) {
        self.has_preferred_x=false;
        self.undo.record_delete(start,&self.buffer.units[start..start+count]);
        if let Err(error)=self.buffer.delete_chars(start,count) {self.error=Some(error);}else {self.refresh_text();}
    }
    pub fn click(&mut self,x:f32,y:f32) {
        self.cursor=crate::text_edit_layout::locate(&self.text,x,y,self.font_height());self.clear_selection();self.has_preferred_x=false;self.cursor_anim=-0.3;
    }
    pub fn drag(&mut self,x:f32,y:f32) {
        let position=crate::text_edit_layout::locate(&self.text,x,y,self.font_height());
        if !self.has_selection() {self.select_start=self.cursor;}
        self.cursor=position;self.select_end=position;
    }
    pub fn widget_drag(&mut self,x:f32,y:f32) {
        self.select_start=self.cursor;
        self.select_end=crate::text_edit_layout::locate(&self.text,x,y,self.font_height());
        self.cursor_anim=-0.3;
        self.cursor_follow=true;
    }
    pub fn clipboard_units(&self)->Option<Vec<u16>> {
        if self.error.is_some() || !self.single_line && !self.has_selection() {return None;}
        let (start,end)=if self.has_selection() {(self.select_start.min(self.select_end),self.select_start.max(self.select_end))}
            else {(0,self.text.len())};
        self.buffer.units.get(start..end).map(|units|units.to_vec())
    }
    pub fn cut(&mut self)->bool {
        if self.error.is_some() || !self.has_selection() {return false;}
        self.delete_selection();self.has_preferred_x=false;true
    }
    pub fn paste(&mut self,units:&[u16])->bool {
        if self.error.is_some() {return false;}
        self.clamp();self.delete_selection();
        match self.buffer.insert_chars(self.cursor,units) {
            Ok(true)=>{self.undo.record_insert(self.cursor,units.len());self.cursor+=units.len();
                self.has_preferred_x=false;self.refresh_text();true}
            Ok(false)=>{if self.undo.undo_point!=0 {self.undo.undo_point-=1;}false}
            Err(error)=>{self.error=Some(error);false}
        }
    }
    pub fn on_key_pressed(&mut self,key:Key,shift:bool) {
        self.key(key,shift);if self.error.is_none() {self.cursor_anim=-0.3;self.cursor_follow=true;}
    }
    pub fn has_selection(&self)->bool {self.select_start!=self.select_end}
    pub fn clear_selection(&mut self) {self.select_start=self.cursor;self.select_end=self.cursor;}
    pub fn select_all(&mut self) {self.select_start=0;self.select_end=self.text.len();self.cursor=self.text.len();self.has_preferred_x=false;}
    pub fn clamp(&mut self) {
        let n=self.text.len();
        if self.has_selection() {
            self.select_start=self.select_start.min(n);self.select_end=self.select_end.min(n);
            if !self.has_selection() {self.cursor=self.select_start;}
        }
        self.cursor=self.cursor.min(n);
    }
    fn prepare_selection(&mut self) {
        if !self.has_selection() {self.clear_selection();}else {self.cursor=self.select_end;}
    }
    fn collapse(&mut self,last:bool) {
        if self.has_selection() {
            self.cursor=if last {self.select_start.max(self.select_end)}else {self.select_start.min(self.select_end)};
            self.clear_selection();self.has_preferred_x=false;
        }
    }
    pub fn delete_selection(&mut self)->bool {
        self.clamp();if !self.has_selection() {return false;}
        let start=self.select_start.min(self.select_end);let end=self.select_start.max(self.select_end);
        self.delete(start,end-start);self.cursor=start;self.clear_selection();true
    }
    fn separator(unit:u16)->bool {matches!(unit,32|9|12288|44|59|40|41|123|125|91|93|124)}
    fn word_boundary(&self,index:usize)->bool {index==0 || Self::separator(self.text[index-1]) && !Self::separator(self.text[index])}
    fn word_left(&self)->usize {
        let mut index=self.cursor.saturating_sub(1);
        while index>0 && !self.word_boundary(index) {index-=1;}index
    }
    fn word_right(&self)->usize {
        let mut index=self.cursor+1;
        while index<self.text.len() && !self.word_boundary(index) {index+=1;}index.min(self.text.len())
    }
    fn vertical(&mut self,down:bool,shift:bool) {
        if self.single_line {self.key(if down {Key::Right}else {Key::Left},shift);}
        if shift {self.prepare_selection();}else {self.collapse(down);}
        self.clamp();
        let find=crate::text_edit_layout::find_at_font(&self.text,self.cursor,self.single_line,self.font_height());
        if down && find.length==0 || !down && find.previous==find.first {return;}
        let goal=if self.has_preferred_x {self.preferred_x}else {find.x};
        let start=if down {find.first+find.length}else {find.previous};
        self.cursor=start;
        let row=crate::text_edit_layout::row(&self.text,start,self.font_height());
        let mut x=0.0;
        for i in 0..row.count {
            let unit=self.text[start+i];
            let width=if unit==10 {-1.0}else {crate::source_font_advances::advance(unit)*(self.font_height()/18.0)};
            if width==-1.0 {break;}
            x+=width;if x>goal {break;}
            self.cursor+=1;
        }
        self.clamp();self.has_preferred_x=true;self.preferred_x=goal;
        if shift {self.select_end=self.cursor;}
    }
    pub fn key(&mut self,key:Key,shift:bool) {
        if self.error.is_some() {return;}
        if matches!(key,Key::Up|Key::Down) {self.vertical(matches!(key,Key::Down),shift);return;}
        if !matches!(key,Key::WordLeft|Key::WordRight) {self.has_preferred_x=false;}
        if matches!(key,Key::Undo|Key::Redo) {
            let result=if matches!(key,Key::Undo) {self.undo.undo(&mut self.buffer,&mut self.cursor)}
                else {self.undo.redo(&mut self.buffer,&mut self.cursor)};
            if let Err(error)=result {self.error=Some(error);}else {self.refresh_text();}
            return;
        }
        if matches!(key,Key::LineStart|Key::LineEnd|Key::Backspace|Key::Delete) {self.clamp();}
        if matches!(key,Key::Backspace|Key::Delete) {
            if !self.delete_selection() {
                if matches!(key,Key::Backspace) && self.cursor>0 {self.delete(self.cursor-1,1);self.cursor-=1;}
                else if matches!(key,Key::Delete) && self.cursor<self.text.len() {self.delete(self.cursor,1);}
            }
            return;
        }
        if shift {self.prepare_selection();}
        else if matches!(key,Key::Left|Key::Right|Key::WordLeft|Key::WordRight) && self.has_selection() {
            self.collapse(matches!(key,Key::Right|Key::WordRight));return;
        }else if matches!(key,Key::LineStart|Key::LineEnd) {self.collapse(false);}
        self.cursor=match key {
            Key::Left=>self.cursor.saturating_sub(1),Key::Right=>(self.cursor+1).min(self.text.len()),
            Key::WordLeft=>self.word_left(),Key::WordRight=>self.word_right(),
            Key::TextStart=>0,Key::TextEnd=>self.text.len(),
            Key::LineStart=>{
                let mut index=self.cursor;
                if self.single_line {index=0;}else {while index>0 && self.text[index-1]!=10 {index-=1;}}index
            },
            Key::LineEnd=>{
                let mut index=self.cursor;
                if self.single_line {index=self.text.len();}else {
                    // Shipped non-shift LINEEND compares literal 'n' (110).
                    // Original key bytecode confirms CFR's unusual comparison.
                    let stop=if shift {10}else {110};
                    while index<self.text.len() && self.text[index]!=stop {index+=1;}
                }index
            },
            Key::Backspace|Key::Delete|Key::Undo|Key::Redo|Key::Up|Key::Down=>unreachable!(),
        };
        if shift {self.select_end=self.cursor;}
        else if matches!(key,Key::TextStart|Key::TextEnd) {self.select_start=0;self.select_end=0;}
    }
    pub fn insert(&mut self,units:&[u16]) {
        for &unit in units {
            if self.error.is_some() {break;}
            if unit==0 || unit==10 && self.single_line {continue;}
            self.delete_selection();
            // Source bufCapacityA=256, insertChars counts one per char unit.
            match self.buffer.insert_chars(self.cursor,&[unit]) {
                Ok(true)=>{self.has_preferred_x=false;self.undo.record_insert(self.cursor,1);self.cursor+=1;self.refresh_text();}
                Ok(false)=>{},Err(error)=>self.error=Some(error),
            }
            if self.error.is_none() {self.cursor_anim=-0.3;self.cursor_follow=true;}
        }
    }
}
#[cfg(test)]
mod source_font_projection_tests {
    use super::*;
    #[test]
    fn source_scaled_click_drag_and_vertical_navigation_preserve_character_indices() {
        let units="Wi\nii".encode_utf16().collect::<Vec<_>>();
        let mut a=TextEditState::default();a.begin_focus(&units,false);
        let mut b=TextEditState::default();b.begin_focus(&units,false);b.set_font_height(22.5);
        for (x,y) in [(0.0,0.0),(10.0,0.0),(100.0,18.0)] {
            a.click(x,y);b.click(x*1.25,y*1.25);assert_eq!(a.cursor,b.cursor);
            a.drag(x+3.0,y);b.drag((x+3.0)*1.25,y*1.25);assert_eq!(a.select_end,b.select_end);
        }
        a.click(8.0,0.0);b.click(10.0,0.0);
        a.key(Key::Down,false);b.key(Key::Down,false);assert_eq!(a.cursor,b.cursor);
        a.key(Key::Up,true);b.key(Key::Up,true);assert_eq!(a.select_end,b.select_end);
    }
    #[test]
    fn source_pixel_scroll_and_preferred_position_survive_display_projection_resize() {
        let mut s=TextEditState::default();s.scroll_x=10.0;s.scroll_y=36.0;s.preferred_x=24.0;
        s.set_font_height(22.5);assert_eq!((s.scroll_x,s.scroll_y,s.preferred_x),(12.5,45.0,30.0));
        s.set_font_height(11.25);assert_eq!((s.scroll_x,s.scroll_y,s.preferred_x),(6.25,22.5,15.0));
        s.set_font_height(11.25);assert_eq!(s.scroll_y,22.5);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state(text:&str,single:bool)->TextEditState {let mut s=TextEditState::default();s.synchronize(&text.encode_utf16().collect::<Vec<_>>(),single);s}
    #[test] fn clipboard_copy_cut_and_grouped_paste_follow_source_records() {
        let mut s=state("abcd",true);assert_eq!(s.clipboard_units(),Some(vec![97,98,99,100]));
        s.key(Key::Left,true);s.key(Key::Left,true);
        assert_eq!(s.clipboard_units(),Some(vec![99,100]));assert!(s.cut());assert_eq!(s.text,vec![97,98]);
        assert!(s.paste(&[0xd83d,0xde80]));assert_eq!(s.text,vec![97,98,0xd83d,0xde80]);
        assert_eq!(s.undo.records[s.undo.undo_point-1].delete_length,2);
        let m=state("description",false);assert_eq!(m.clipboard_units(),None);
    }
    #[test] fn failed_source_paste_drops_last_record_even_without_selection() {
        let mut s=state("",true);s.insert(&[65]);assert_eq!(s.undo.undo_point,1);
        assert!(!s.paste(&vec![66;300]));assert_eq!(s.text,vec![65]);assert_eq!(s.undo.undo_point,0);
    }
    #[test] fn original_jvm_vertical_snapshots_preserve_column_and_selection_fields() {
        let mut s=state("",false);s.insert(&[87,105,10,105,108,10,87,87,87,10]);
        let keys=[(Key::Up,false),(Key::Up,false),(Key::Up,false),(Key::Down,false),(Key::Down,false),
            (Key::Down,true),(Key::Up,true),(Key::TextStart,false),(Key::Right,false),(Key::Right,false),
            (Key::Down,false),(Key::Down,false),(Key::Up,false),(Key::Up,false),(Key::Down,true),(Key::Down,false)];
        let expected=[(6,0,0,true,0),(3,0,0,true,0),(0,0,0,true,0),(3,0,0,true,0),(6,0,0,true,0),(10,6,10,true,0),(6,6,6,true,0),(0,0,0,false,0),(1,0,0,false,0),(2,0,0,false,0),(5,0,0,true,1099232706),(7,0,0,true,1099232706),(5,0,0,true,1099232706),(2,0,0,true,1099232706),(5,2,5,true,1099232706),(6,5,5,true,1091174400)];
        for ((key,shift),snapshot) in keys.into_iter().zip(expected) {
            s.key(key,shift);
            assert_eq!((s.cursor,s.select_start,s.select_end,s.has_preferred_x,s.preferred_x.to_bits()),snapshot);
        }
    }
    #[test] fn source_click_and_drag_keep_anchor_across_utf16_lines() {
        let mut s=state("Wi\nil",false);s.click(0.0,0.0);s.drag(1000.0,18.0);
        assert_eq!((s.cursor,s.select_start,s.select_end),(5,0,5));
        s.drag(5.0,0.0);assert_eq!((s.cursor,s.select_start,s.select_end),(0,0,0));
    }
    #[test] fn focus_snapshot_preserves_raw_units_and_changes_only_on_reactivation() {
        let mut s=TextEditState::default();s.begin_focus(&[97,0xd83d],true);
        s.insert(&[98]);s.publish();assert_eq!(s.cancel_units(),vec![97,0xd83d]);
        s.begin_focus(&[88],true);s.insert(&[89]);assert_eq!(s.cancel_units(),vec![88]);
    }
    #[test] fn backing_undo_redo_and_user_buffer_tail_follow_original_publication() {
        let mut s=state("",true);s.insert(&[97,98,99]);assert_eq!(s.publish(),Some(vec![97,98,99]));
        s.key(Key::Undo,false);assert_eq!(s.publish(),Some(vec![97,98]));
        s.key(Key::Redo,false);assert_eq!(s.text,vec![97]);assert_eq!(s.cursor,2);
        assert_eq!(s.publish(),Some(vec![97,98]));
        s.synchronize(&[97,98],true);assert_eq!(s.text,vec![97]);assert_eq!(s.cursor,2);
        s.key(Key::Left,false);assert_eq!(s.cursor,1);
    }
    #[test] fn selection_collapses_without_an_extra_step_and_replaces_in_order() {
        let mut s=state("abcd",true);s.key(Key::Left,true);s.key(Key::Left,true);s.key(Key::Left,false);
        assert_eq!(s.cursor,2);s.key(Key::Right,true);s.insert(&[88]);assert_eq!(s.text,vec![97,98,88,100]);
    }
    #[test] fn utf16_movement_deletes_one_source_char_unit() {
        let mut s=state("a\u{1f600}b",true);s.key(Key::Left,false);s.key(Key::Backspace,false);
        assert_eq!(s.text,vec![97,0xd83d,98]);assert_eq!(s.cursor,2);
    }
    #[test] fn word_boundaries_use_source_separators_not_general_unicode_whitespace() {
        let mut s=state("one,two three",true);s.key(Key::WordLeft,false);assert_eq!(s.cursor,8);
        s.key(Key::WordLeft,false);assert_eq!(s.cursor,4);s.key(Key::WordRight,false);assert_eq!(s.cursor,8);
    }
    #[test] fn shipped_plain_multiline_end_keeps_literal_n_quirk() {
        let mut s=state("ab\nnext",false);s.key(Key::TextStart,false);s.key(Key::LineEnd,false);assert_eq!(s.cursor,3);
        s.key(Key::TextStart,false);s.key(Key::LineEnd,true);assert_eq!(s.cursor,2);
    }
    #[test] fn newline_is_rejected_only_for_single_line_and_capacity_counts_units() {
        let mut s=state("",true);s.insert(&[10]);assert!(s.text.is_empty());
        s.synchronize(&[],false);s.insert(&[10]);assert_eq!(s.text,vec![10]);
        s.insert(&vec![65;300]);assert_eq!(s.text.len(),255);
        assert_eq!(s.buffer.len_w,256);assert_eq!(s.error,Some("ArrayIndexOutOfBoundsException"));
        assert!(s.publish().is_none());
    }
}
