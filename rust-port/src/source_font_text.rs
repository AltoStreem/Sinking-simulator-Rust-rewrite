//! Original imgui/font/Font text measurement and ASCII word wrapping.
//! Source FontAtlas construction and glyph rasterization are separate work.
use bevy::prelude::Vec2;

#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) enum Error {
    NotImplemented(&'static str),
    StringBounds(i32,usize,bool),
    ArrayBounds(i32,usize),
}
impl Error {
    pub(crate) fn class(&self)->&'static str {
        match self {Self::NotImplemented(_)=>"kotlin.NotImplementedError",Self::StringBounds(..)=>"java.lang.StringIndexOutOfBoundsException",Self::ArrayBounds(..)=>"java.lang.ArrayIndexOutOfBoundsException"}
    }
    pub(crate) fn message(&self)->String {
        match self {
            Self::NotImplemented("")=>"An operation is not implemented.".into(),
            Self::NotImplemented(message)=>format!("An operation is not implemented: {message}"),
            Self::StringBounds(index,length,true)=>format!("index {index},length {length}"),
            Self::StringBounds(index,_,false)=>format!("String index out of range: {index}"),
            Self::ArrayBounds(index,length)=>format!("Index {index} out of bounds for length {length}"),
        }
    }
}
#[derive(Clone,Copy,Debug,PartialEq)]
pub(crate) struct Measured {pub size:Vec2,pub remaining:i32}
/// Font.calcTextSizeA reads indexAdvanceX directly, falling back beyond that table.
pub(crate) trait Advances {fn advance(&self,unit:u16)->f32;}
pub(crate) struct F18;
impl Advances for F18 {fn advance(&self,unit:u16)->f32 {crate::source_font_advances::advance(unit)}}
pub(crate) struct Font<A> {pub font_size:f32,pub advances:A}
fn blank(unit:u16)->bool {unit==32 || unit==9}
fn string_bounds(text:&[u16],index:i32)->Error {
    // The bundled Java 13 default compact-string implementation uses separate
    // Latin1/UTF16 checks. Original JVM snapshots retain their different messages.
    Error::StringBounds(index,text.len(),text.iter().any(|u|*u>255))
}
impl<A:Advances> Font<A> {
    pub(crate) fn wrap(&self,scale:f32,text:&[u16],ptr:i32,text_end:i32,wrap_width:f32)->Result<i32,Error> {
        let mut line_width=0.0f32;let mut word_width=0.0f32;let mut blank_width=0.0f32;
        let width=wrap_width/scale;let mut word_end=ptr;let mut prev_word_end=-1;let mut inside_word=true;let mut s=ptr;
        while s<text_end {
            let unit=*text.get(s as usize).ok_or(Error::ArrayBounds(s,text.len()))?;
            if unit>=128 {return Err(Error::NotImplemented(""));}
            let next_s=s.wrapping_add(1);
            if unit==0 {break;}
            if unit<32 {
                if unit==10 {line_width=0.0;word_width=0.0;blank_width=0.0;inside_word=true;s=next_s;continue;}
                if unit==13 {s=next_s;continue;}
            }
            let char_width=self.advances.advance(unit);
            if blank(unit) {
                if inside_word {line_width+=blank_width;blank_width=0.0;word_end=s;}
                blank_width+=char_width;inside_word=false;
            } else {
                word_width+=char_width;
                if inside_word {word_end=next_s;}else {prev_word_end=word_end;line_width+=word_width+blank_width;word_width=0.0;blank_width=0.0;}
                inside_word=!matches!(unit,46|44|59|33|63|34);
            }
            if line_width+word_width>width {
                if word_width<width {s=if prev_word_end != -1 {prev_word_end}else {word_end};}
                break;
            }
            s=next_s;
        }
        Ok(s)
    }
    pub(crate) fn measure(&self,size:f32,max_width:f32,wrap_width:f32,text:&[u16],text_end:i32)->Result<Measured,Error> {
        let text_end=if text_end == -1 {text.len() as i32}else {text_end};
        let scale=size/self.font_size;let mut result=Vec2::ZERO;let mut line_width=0.0f32;
        let word_wrap_enabled=wrap_width>0.0;let mut word_wrap_eol=-1;let mut s=0i32;
        'outer: while s<text_end {
            if word_wrap_enabled {
                if word_wrap_eol == -1 {
                    word_wrap_eol=self.wrap(scale,text,s,text_end,wrap_width-line_width)?;
                    if word_wrap_eol==s {word_wrap_eol=word_wrap_eol.wrapping_add(1);}
                }
                if s>=word_wrap_eol {
                    if result.x<line_width {result.x=line_width;}
                    result.y+=size;line_width=0.0;word_wrap_eol=-1;
                    while s<text_end {
                        let unit=*text.get(s as usize).ok_or_else(||string_bounds(text,s))?;
                        if blank(unit) {s=s.wrapping_add(1);continue;}
                        if unit==10 {s=s.wrapping_add(1);}
                        continue 'outer;
                    }
                    continue;
                }
            }
            let prev_s=s;let unit=*text.get(s as usize).ok_or_else(||string_bounds(text,s))?;
            if unit<0xd800 {s=s.wrapping_add(1);}else {return Err(Error::NotImplemented("Probabily surrogate character"));}
            if unit<32 {
                if unit==10 {
                    // glm.max uses the source comparison order rather than f32::max's NaN rule.
                    result.x=if result.x>line_width {result.x}else {line_width};
                    result.y+=size;line_width=0.0;continue;
                }
                if unit==13 {continue;}
            }
            let char_width=self.advances.advance(unit)*scale;
            if line_width+char_width>=max_width {s=prev_s;break;}
            line_width+=char_width;
        }
        if result.x<line_width {result.x=line_width;}
        if line_width>0.0 || result.y==0.0 {result.y+=size;}
        Ok(Measured {size:result,remaining:s})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_jvm_measurement_and_wrap_snapshots_match_float_bits_remaining_indices_and_errors() {
        let fixture:serde_json::Value=serde_json::from_str(include_str!("../tools/source-font-text-capture.json")).unwrap();
        let font=Font {font_size:18.0,advances:F18};
        let value=|case:&serde_json::Value,key:&str|f32::from_bits(case[key].as_u64().unwrap() as u32);
        for (index,case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
            let units:Vec<_>=case["units"].as_array().unwrap().iter().map(|u|u.as_u64().unwrap() as u16).collect();
            let expected=&case["result"];
            let result=if case["kind"]=="measure" {
                font.measure(value(case,"sizeBits"),value(case,"maxWidthBits"),value(case,"wrapWidthBits"),&units,case["end"].as_i64().unwrap() as i32).map(|m|vec![m.size.x.to_bits() as i64,m.size.y.to_bits() as i64,m.remaining as i64])
            } else {
                font.wrap(value(case,"scaleBits"),&units,case["ptr"].as_i64().unwrap() as i32,case["end"].as_i64().unwrap() as i32,value(case,"wrapWidthBits"))
                    .map(|value|vec![value as i64])
            };
            match result {
                Ok(actual)=>assert_eq!(actual,expected["values"].as_array().unwrap().iter().map(|v|v.as_i64().unwrap()).collect::<Vec<_>>(),"case {index}: {case}"),
                Err(error)=>{assert_eq!(error.class(),expected["class"].as_str().unwrap(),"case {index}");assert_eq!(error.message(),expected["message"].as_str().unwrap(),"case {index}");},
            }
        }
    }
    #[test]
    fn independent_character_advances_do_not_kern_or_make_ligatures() {
        let font=Font {font_size:18.0,advances:F18};
        for text in ["AV","To","floors","office","ffi","fi","fl"] {
            let units:Vec<_>=text.encode_utf16().collect();let expected=units.iter().fold(0.0f32,|width,u|width+font.advances.advance(*u));
            assert_eq!(font.measure(18.0,f32::MAX,0.0,&units,-1).unwrap().size.x.to_bits(),expected.to_bits());
        }
    }
}
