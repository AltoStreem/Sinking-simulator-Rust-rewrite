//! Source TextEditState.layout / locateCoord using original default font metrics.
#[derive(Debug,Default)]
pub(crate) struct Row {pub width:f32,pub height:f32,pub count:usize}
pub(crate) fn row(text:&[u16],start:usize,font_size:f32)->Row {
    let mut width=0.0;let mut count=0;
    for &unit in &text[start..] {
        count+=1;
        if unit==10 {break;}
        if unit!=13 {width+=crate::source_font_advances::advance(unit)*(font_size/18.0);}
    }
    Row {width,height:font_size,count}
}
pub(crate) fn locate(text:&[u16],x:f32,y:f32,font_size:f32)->usize {
    let mut index=0;let mut base_y=0.0;
    while index<text.len() {
        let r=row(text,index,font_size);
        if r.count==0 {return text.len();}
        if index==0 && y<base_y {return 0;}
        if y<base_y+r.height {
            if x<0.0 {return index;}
            if x<r.width {
                let mut previous=0.0;
                for k in 0..r.count {
                    let unit=text[index+k];
                    let width=if unit==10 {-1.0}else {crate::source_font_advances::advance(unit)*(font_size/18.0)};
                    if x<previous+width {
                        return index+k+usize::from(x>=previous+width/2.0);
                    }
                    previous+=width;
                }
            }
            return index+r.count-usize::from(text[index+r.count-1]==10);
        }
        index+=r.count;base_y+=r.height;
    }
    text.len()
}
#[derive(Debug,Default)]
pub(crate) struct Find {pub first:usize,pub previous:usize,pub length:usize,pub x:f32}
pub(crate) fn find(text:&[u16],cursor:usize,single_line:bool)->Find {find_at_font(text,cursor,single_line,18.0)}
pub(crate) fn find_at_font(text:&[u16],cursor:usize,single_line:bool,font_size:f32)->Find {
    if cursor==text.len() {
        if single_line {return Find {length:text.len(),x:row(text,0,font_size).width,..Default::default()};}
        let mut index=0;let mut previous=0;
        while index<text.len() {previous=index;index+=row(text,index,font_size).count;}
        return Find {first:index,previous,..Default::default()};
    }
    let mut first=0;let mut previous=0;
    loop {
        let r=row(text,first,font_size);
        if cursor<first+r.count {
            let mut x=0.0;
            for &unit in &text[first..cursor] {x+=if unit==10 {-1.0}else {crate::source_font_advances::advance(unit)*(font_size/18.0)};}
            return Find {first,previous,length:r.count,x};
        }
        previous=first;first+=r.count;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn original_jvm_fira_locate_cases_match_exact_indices() {
        let text=[87,105,10,105,108];
        let cases=[(0.0,0.0),(5.0,0.0),(1000.0,0.0),(0.0,18.0),(5.0,18.0),(1000.0,18.0),(0.0,36.0),(1000.0,-1.0)];
        assert_eq!(cases.map(|(x,y)|locate(&text,x,y,18.0)),[0,0,2,3,4,5,5,0]);
    }
    #[test] fn variable_width_midpoints_choose_utf16_boundaries_and_fallback_units() {
        let text=[87,105,0xd83d,0xde80];let w=crate::source_font_advances::advance(87);
        assert_eq!(locate(&text,w*0.49,0.0,18.0),0);
        assert_eq!(locate(&text,w*0.5,0.0,18.0),1);
        assert_eq!(locate(&text,-1.0,0.0,18.0),0);
        assert_eq!(locate(&text,1000.0,0.0,18.0),4);
        assert_eq!(crate::source_font_advances::advance(0xd83d),crate::source_font_advances::advance(63));
    }
    #[test] fn newline_and_empty_line_edges_use_source_rows_without_wrapping() {
        let text=[87,105,10,10,105,108];
        assert_eq!(locate(&text,1000.0,0.0,18.0),2);
        assert_eq!(locate(&text,0.0,18.0,18.0),3);
        assert_eq!(locate(&text,0.0,36.0,18.0),4);
        assert_eq!(locate(&text,0.0,54.0,18.0),6);
        assert_eq!(locate(&text,1000.0,-1.0,18.0),0);
        assert_eq!(row(&[97,13,98,10],0,18.0).count,4);
    }
}
