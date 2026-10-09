//! Native ship-editor clipboard service. Called only by explicit UI shortcuts.
//! GLFW UTF-8 conversion, delayed formats and native failure parity remain pending.
use bevy::prelude::*;
#[derive(Resource,Default)]
pub(crate) struct Clipboard {
    #[cfg(test)] pub fixture: Option<Vec<u16>>,
}
impl Clipboard {
    pub fn read(&mut self,owner:isize)->Option<Vec<u16>> {
        #[cfg(test)] if let Some(units)=&self.fixture {return Some(units.clone());}
        native::read(owner)
    }
    pub fn write(&mut self,owner:isize,units:&[u16])->bool {
        #[cfg(test)] if let Some(fixture)=&mut self.fixture {*fixture=units.to_vec();return true;}
        native::write(owner,units)
    }
}
/// Source default inputText filter; default multiline permits LF, not Tab.
pub(crate) fn accepted(unit:u16,multiline:bool)->bool {
    (unit>=32 || unit==10 && multiline) && unit!=127 && !(0xe000..=0xf8ff).contains(&unit)
}
pub(crate) fn filtered(units:&[u16],multiline:bool)->Vec<u16> {
    units.iter().copied().take_while(|&unit|unit!=0).filter(|&unit|accepted(unit,multiline)).collect()
}
pub(crate) fn owner(raw:Option<&bevy::window::RawHandleWrapper>)->isize {
    #[cfg(windows)] if let Some(raw)=raw {
        if let winit::raw_window_handle::RawWindowHandle::Win32(handle)=raw.get_window_handle() {return handle.hwnd.get();}
    }
    0
}
#[cfg(windows)]
mod native {
    use std::ffi::c_void;
    #[link(name="user32")]
    unsafe extern "system" {
        fn OpenClipboard(owner:*mut c_void)->i32; fn CloseClipboard()->i32;
        fn EmptyClipboard()->i32; fn GetClipboardData(format:u32)->*mut c_void;
        fn SetClipboardData(format:u32,memory:*mut c_void)->*mut c_void;
    }
    #[link(name="kernel32")]
    unsafe extern "system" {
        fn GlobalAlloc(flags:u32,bytes:usize)->*mut c_void;fn GlobalFree(memory:*mut c_void)->*mut c_void;
        fn GlobalLock(memory:*mut c_void)->*mut c_void;fn GlobalUnlock(memory:*mut c_void)->i32;
        fn GlobalSize(memory:*mut c_void)->usize;
    }
    struct Open;
    impl Drop for Open {fn drop(&mut self) {unsafe {CloseClipboard();}}}
    pub fn read(owner:isize)->Option<Vec<u16>> {
        unsafe {
            if OpenClipboard(owner as *mut c_void)==0 {return None;}let _open=Open;
            let memory=GetClipboardData(13);if memory.is_null() {return None;}
            let size=GlobalSize(memory)/2;let pointer=GlobalLock(memory) as *const u16;
            if pointer.is_null() {return None;}
            let units=std::slice::from_raw_parts(pointer,size);
            let n=units.iter().position(|&u|u==0).unwrap_or(size);
            let result=units[..n].to_vec();GlobalUnlock(memory);Some(result)
        }
    }
    pub fn write(owner:isize,units:&[u16])->bool {
        // A valid game HWND supplies clipboard ownership after EmptyClipboard.
        if owner==0 {return false;}
        unsafe {
            let count=units.iter().position(|&u|u==0).unwrap_or(units.len());
            let Some(bytes)=count.checked_add(1).and_then(|n|n.checked_mul(2)) else {return false;};
            let memory=GlobalAlloc(2,bytes);if memory.is_null() {return false;}
            let pointer=GlobalLock(memory) as *mut u16;
            if pointer.is_null() {GlobalFree(memory);return false;}
            std::ptr::copy_nonoverlapping(units.as_ptr(),pointer,count);*pointer.add(count)=0;
            GlobalUnlock(memory);
            if OpenClipboard(owner as *mut c_void)==0 {GlobalFree(memory);return false;}
            let _open=Open;
            if EmptyClipboard()==0 || SetClipboardData(13,memory).is_null() {GlobalFree(memory);return false;}
            // Ownership passes to Windows only after successful SetClipboardData.
            true
        }
    }
}
#[cfg(not(windows))]
mod native {pub fn read(_:isize)->Option<Vec<u16>> {None} pub fn write(_:isize,_:&[u16])->bool {false}}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn default_clipboard_filter_preserves_c1_surrogates_and_multiline_lf() {
        let units=[65,9,10,13,127,0x85,0xe000,0xf8ff,0xf900,0xd83d,0xde80,0,66];
        assert_eq!(filtered(&units,false),vec![65,0x85,0xf900,0xd83d,0xde80]);
        assert_eq!(filtered(&units,true),vec![65,10,0x85,0xf900,0xd83d,0xde80]);
    }
    #[test] fn clipboard_fixture_never_touches_native_data() {
        let mut clipboard=Clipboard {fixture:Some(vec![97,0xd83d])};
        assert_eq!(clipboard.read(0),Some(vec![97,0xd83d]));
        assert!(clipboard.write(0,&[98]));assert_eq!(clipboard.read(0),Some(vec![98]));
    }
}
