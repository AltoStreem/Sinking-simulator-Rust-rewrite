//! Bundled TextEditState.UndoRecord and UndoState storage translation.
//! Source undo/redo application retains physical UTF-16 buffer and logical lengths.
//! Live keyboard shortcut wiring awaits migration to this backing-buffer model.
const RECORD_COUNT: usize = 99;
const CHAR_COUNT: usize = 999;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct UndoRecord {
    pub where_: usize,
    pub insert_length: usize,
    pub delete_length: usize,
    pub char_storage: isize,
}
#[derive(Debug)]
pub(crate) struct UndoState {
    pub records: [UndoRecord; RECORD_COUNT],
    pub characters: [u16; CHAR_COUNT],
    pub undo_point: usize,
    pub redo_point: usize,
    pub undo_char_point: usize,
    pub redo_char_point: usize,
}
impl Default for UndoState {
    fn default() -> Self {
        // Source State.initialize calls clear after constructing UndoState.
        Self { records: [UndoRecord::default(); RECORD_COUNT], characters: [0; CHAR_COUNT],
            undo_point: 0, redo_point: RECORD_COUNT, undo_char_point: 0, redo_char_point: CHAR_COUNT }
    }
}
impl UndoState {
    pub fn clear(&mut self) {
        self.undo_point=0; self.undo_char_point=0;
        self.flush_redo();
    }
    pub fn flush_redo(&mut self) {
        self.redo_point=RECORD_COUNT; self.redo_char_point=CHAR_COUNT;
    }
    pub fn discard_undo(&mut self) {
        if self.undo_point==0 {return;}
        if self.records[0].char_storage>=0 {
            let n=self.records[0].insert_length;
            self.undo_char_point-=n;
            self.characters.copy_within(n..n+self.undo_char_point,0);
            for record in &mut self.records[..self.undo_point] {
                if record.char_storage>=0 {record.char_storage-=n as isize;}
            }
        }
        self.undo_point-=1;
        self.records.copy_within(1..1+self.undo_point,0);
    }
    pub fn discard_redo(&mut self) {
        let k=RECORD_COUNT-1;
        if self.redo_point>k {return;}
        if self.records[k].char_storage>=0 {
            let n=self.records[k].insert_length;
            self.redo_char_point+=n;
            let count=CHAR_COUNT-self.redo_char_point;
            self.characters.copy_within(self.redo_char_point-n..self.redo_char_point-n+count,self.redo_char_point);
            for record in &mut self.records[self.redo_point..k] {
                if record.char_storage>=0 {record.char_storage+=n as isize;}
            }
        }
        // Preserve the shipped source copy range, including the record just
        // below redo_point. It differs from common stb_textedit versions.
        self.records.copy_within(self.redo_point-1..RECORD_COUNT-1,self.redo_point);
        self.redo_point+=1;
    }
    pub fn create_record(&mut self,num_chars:usize)->Option<usize> {
        self.flush_redo();
        if self.undo_point==RECORD_COUNT {self.discard_undo();}
        if num_chars>CHAR_COUNT {
            self.undo_point=0;self.undo_char_point=0;return None;
        }
        while self.undo_char_point+num_chars>CHAR_COUNT {self.discard_undo();}
        let index=self.undo_point;self.undo_point+=1;Some(index)
    }
    pub fn create_undo(&mut self,pos:usize,insert_len:usize,delete_len:usize)->Option<usize> {
        let index=self.create_record(insert_len)?;
        self.records[index]=UndoRecord {where_:pos,insert_length:insert_len,
            delete_length:delete_len,char_storage:-1};
        // Null return for zero character storage still creates an undo record.
        if insert_len==0 {return None;}
        let storage=self.undo_char_point;
        self.records[index].char_storage=storage as isize;
        self.undo_char_point+=insert_len;Some(storage)
    }
    pub fn record_insert(&mut self,where_:usize,length:usize) {
        self.create_undo(where_,0,length);
    }
    pub fn record_delete(&mut self,where_:usize,units:&[u16]) {
        if let Some(storage)=self.create_undo(where_,units.len(),0) {
            self.characters[storage..storage+units.len()].copy_from_slice(units);
        }
    }
}
/// Source char[] backing and independent curLenW/curLenA fields.
/// Operations return the source array-bound exception boundary explicitly.
#[derive(Debug)]
pub(crate) struct EditBuffer {
    pub units: Vec<u16>, pub len_w: isize, pub len_a: isize, pub capacity_a: usize,
}
impl Default for EditBuffer {
    fn default()->Self {Self {units:vec![0;256],len_w:0,len_a:0,capacity_a:256}}
}
impl EditBuffer {
    fn char_at(&self,index:usize)->Result<u16,&'static str> {
        self.units.get(index).copied().ok_or("ArrayIndexOutOfBoundsException")
    }
    pub fn delete_chars(&mut self,pos:usize,n:usize)->Result<(),&'static str> {
        if n==0 {return Ok(());}
        self.len_a-=n as isize;self.len_w-=n as isize;
        let mut dst=pos;
        for source in pos+n..self.units.len() {
            let value=self.units[source];
            *self.units.get_mut(dst).ok_or("ArrayIndexOutOfBoundsException")?=value;
            dst+=1;
        }
        *self.units.get_mut(dst).ok_or("ArrayIndexOutOfBoundsException")?=0;
        Ok(())
    }
    pub fn insert_chars(&mut self,pos:usize,units:&[u16])->Result<bool,&'static str> {
        let text_len=usize::try_from(self.len_w).map_err(|_|"ArrayIndexOutOfBoundsException")?;
        let n=units.len();
        // Source assertions are disabled by the supplied JVM invocation.
        // No implicit cursor clamp is inserted here.
        if n as isize+self.len_a>self.capacity_a as isize || n+text_len>self.units.len() {return Ok(false);}
        if pos!=text_len {
            for i in 0..text_len.saturating_sub(pos) {
                let value=self.units[text_len-1-i];
                *self.units.get_mut(text_len-1+n-i).ok_or("ArrayIndexOutOfBoundsException")?=value;
            }
        }
        for (i,&value) in units.iter().enumerate() {
            *self.units.get_mut(pos+i).ok_or("ArrayIndexOutOfBoundsException")?=value;
        }
        self.len_w+=n as isize;self.len_a+=n as isize;
        let terminator=usize::try_from(self.len_w).map_err(|_|"ArrayIndexOutOfBoundsException")?;
        *self.units.get_mut(terminator).ok_or("ArrayIndexOutOfBoundsException")?=0;
        Ok(true)
    }
}
impl UndoState {
    pub fn undo(&mut self,text:&mut EditBuffer,cursor:&mut usize)->Result<(),&'static str> {
        if self.undo_point==0 {return Ok(());}
        let u=self.records[self.undo_point-1];
        let mut r_index=self.redo_point-1;
        self.records[r_index]=u; // Source put copies, rather than swapping lengths.
        self.records[r_index].char_storage=-1;
        if u.delete_length!=0 {
            if self.undo_char_point+u.delete_length>=CHAR_COUNT {
                self.records[r_index].insert_length=0;
            } else {
                while self.undo_char_point+u.delete_length>self.redo_char_point {
                    self.discard_redo();
                    if self.redo_point==RECORD_COUNT {return Ok(());}
                }
                r_index=self.redo_point-1;
                self.records[r_index].char_storage=(self.redo_char_point-u.delete_length) as isize;
                self.redo_char_point-=u.delete_length;
                for i in 0..u.delete_length {
                    self.characters[self.redo_char_point+i]=text.char_at(u.where_+i)?;
                }
            }
            text.delete_chars(u.where_,u.delete_length)?;
        }
        if u.insert_length!=0 {
            let storage=usize::try_from(u.char_storage).map_err(|_|"ArrayIndexOutOfBoundsException")?;
            let units=self.characters.get(storage..storage+u.insert_length).ok_or("ArrayIndexOutOfBoundsException")?;
            text.insert_chars(u.where_,units)?;
            self.undo_char_point-=u.insert_length;
        }
        *cursor=u.where_+u.insert_length;
        self.undo_point-=1;self.redo_point-=1;Ok(())
    }
    pub fn redo(&mut self,text:&mut EditBuffer,cursor:&mut usize)->Result<(),&'static str> {
        if self.redo_point==RECORD_COUNT {return Ok(());}
        let r=self.records[self.redo_point];
        let u_index=self.undo_point;
        self.records[u_index]=r;
        self.records[u_index].char_storage=-1;
        if r.delete_length!=0 {
            let count=self.records[u_index].insert_length;
            if self.undo_char_point+count>self.redo_char_point {
                self.records[u_index].insert_length=0;
                self.records[u_index].delete_length=0;
            } else {
                self.records[u_index].char_storage=self.undo_char_point as isize;
                let start=self.undo_char_point;self.undo_char_point+=count;
                for i in 0..count {self.characters[start+i]=text.char_at(r.where_+i)?;}
            }
            text.delete_chars(r.where_,r.delete_length)?;
        }
        if r.insert_length!=0 {
            let storage=usize::try_from(r.char_storage).map_err(|_|"ArrayIndexOutOfBoundsException")?;
            let units=self.characters.get(storage..storage+r.insert_length).ok_or("ArrayIndexOutOfBoundsException")?;
            text.insert_chars(r.where_,units)?;
            self.redo_char_point+=r.insert_length;
        }
        *cursor=r.where_+r.insert_length;
        self.undo_point+=1;self.redo_point+=1;Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn original_jvm_insert_undo_redo_retains_shipped_length_copy_behavior() {
        let mut history=UndoState::default();
        let mut text=EditBuffer {units:vec![0;256],len_w:0,len_a:0,capacity_a:256};
        let mut cursor=0;
        for value in [97,98,99] {
            assert!(text.insert_chars(cursor,&[value]).unwrap());
            history.record_insert(cursor,1);cursor+=1;
        }
        assert_eq!((text.len_w,text.len_a,cursor),(3,3,3));
        assert_eq!(&text.units[..4],&[97,98,99,0]);
        history.undo(&mut text,&mut cursor).unwrap();
        assert_eq!((text.len_w,text.len_a,cursor),(2,2,2));
        assert_eq!(&text.units[..4],&[97,98,0,0]);
        history.redo(&mut text,&mut cursor).unwrap();
        assert_eq!((text.len_w,text.len_a,cursor),(1,1,2));
        assert_eq!(&text.units[..4],&[97,98,0,0]);
    }
    #[test] fn source_capacity_terminator_fault_occurs_after_lengths_and_units_change() {
        let mut text=EditBuffer {units:vec![65;256],len_w:255,len_a:255,capacity_a:256};
        assert_eq!(text.insert_chars(255,&[66]),Err("ArrayIndexOutOfBoundsException"));
        assert_eq!((text.len_w,text.len_a,text.units[255]),(256,256,66));
    }
    #[test] fn zero_storage_records_survive_null_return_and_evict_oldest_at_99() {
        let mut s=UndoState::default();
        for i in 0..100 {assert_eq!(s.create_undo(i,0,1),None);}
        assert_eq!(s.undo_point,99);assert_eq!(s.records[0].where_,1);
        assert_eq!(s.records[98].where_,99);assert_eq!(s.undo_char_point,0);
        assert_eq!(s.records[98].char_storage,-1);
    }
    #[test] fn character_capacity_eviction_compacts_units_and_record_offsets() {
        let mut s=UndoState::default();
        s.record_delete(0,&vec![0xd83d;600]);
        s.record_delete(1,&vec![0xde80;300]);
        s.record_delete(2,&vec![65;200]);
        assert_eq!((s.undo_point,s.undo_char_point),(2,500));
        assert_eq!(s.records[0].where_,1);assert_eq!(s.records[0].char_storage,0);
        assert_eq!(s.records[1].char_storage,300);
        assert_eq!(&s.characters[..300],&vec![0xde80;300]);
        assert_eq!(&s.characters[300..500],&vec![65;200]);
    }
    #[test] fn oversized_record_clears_history_and_new_edits_flush_redo() {
        let mut s=UndoState::default();s.record_insert(1,1);
        s.redo_point=98;s.redo_char_point=997;
        assert_eq!(s.create_undo(0,1000,0),None);
        assert_eq!((s.undo_point,s.undo_char_point,s.redo_point,s.redo_char_point),(0,0,99,999));
    }
    #[test] fn redo_discard_uses_source_overlap_copy_and_storage_offsets() {
        let mut s=UndoState::default();s.redo_point=97;s.redo_char_point=994;
        s.records[96]=UndoRecord {where_:96,char_storage:-1,..Default::default()};
        s.records[97]=UndoRecord {where_:97,insert_length:2,char_storage:994,..Default::default()};
        s.records[98]=UndoRecord {where_:98,insert_length:3,char_storage:996,..Default::default()};
        s.characters[994..996].copy_from_slice(&[11,12]);
        s.discard_redo();
        assert_eq!((s.redo_point,s.redo_char_point),(98,997));
        assert_eq!(s.records[97].where_,96);
        assert_eq!((s.records[98].where_,s.records[98].char_storage),(97,997));
        assert_eq!(&s.characters[997..999],&[11,12]);
    }
}
