//! PaletteGenKt.java: materials palette PNG layout and bundled-font rendering.
use image::{Rgb, RgbImage};
use std::path::{Path, PathBuf};

const BLOCK_WIDTH: i32 = 50;
const BLOCK_HEIGHT: i32 = 25;

/// Rust counterpart of PaletteGenKt.genPalette(File, File).
pub(crate) fn gen_palette(materials_file: &Path, destination: &Path) -> Result<(), String> {
    let reader = crate::file_reader::FileReader::game();
    let json = reader
        .read_file(materials_file)
        .map_err(|error| error.to_string())?;
    let materials = crate::materials::Materials::from_json(&json)?;
    let rows = materials.source_palette_rows()?;

    #[cfg(windows)]
    {
        let font_path = resolve_font_path(&reader, Path::new("FiraSans-Regular.ttf"))
            .ok_or_else(|| "FiraSans-Regular.ttf was not found".to_owned())?;
        let mut canvas = GdiCanvas::new(1, 1, &font_path)?;
        let font_offset = (BLOCK_HEIGHT + canvas.ascent()) / 2;
        let max_colors = rows
            .iter()
            .map(|(_, colors)| colors.len())
            .max()
            .unwrap_or(0)
            .max(1) as i32;
        let max_name = rows
            .iter()
            .map(|(material, _)| canvas.text_width(&format!(" {} ", material.name.trim())))
            .chain(std::iter::once(canvas.text_width(" Material ")))
            .max()
            .unwrap_or(0);
        let materials_width = 1 + max_colors * (BLOCK_WIDTH + 1);
        let width = 1 + materials_width + max_name;
        let height = 1 + (rows.len() as i32 + 1) * (BLOCK_HEIGHT + 1);
        canvas.resize(width, height)?;

        // TYPE_INT_RGB starts black; the Java method draws its header before
        // painting the white body, so the header row remains black.
        canvas.text(" Colors ", 0, font_offset);
        canvas.text(" Material ", materials_width, font_offset);
        canvas.fill_rect(
            1,
            BLOCK_HEIGHT + 1,
            width - 2,
            height - BLOCK_HEIGHT - 2,
            [255, 255, 255],
        );

        for (index, (material, colors)) in rows.iter().enumerate() {
            let y = (BLOCK_HEIGHT + 1) * (index as i32 + 1) + 1;
            canvas.draw_rect(
                materials_width - 1,
                y - 1,
                max_name + 1,
                BLOCK_HEIGHT + 1,
                [0, 0, 0],
            );
            canvas.text(
                &format!(" {} ", material.name.trim()),
                materials_width,
                y + font_offset,
            );
            for (color_index, color) in colors.iter().enumerate() {
                let x = 1 + (BLOCK_WIDTH + 1) * (max_colors - 1 - color_index as i32);
                let rgb = [
                    ((color >> 16) & 0xff) as u8,
                    ((color >> 8) & 0xff) as u8,
                    (color & 0xff) as u8,
                ];
                canvas.fill_rect(x, y, BLOCK_WIDTH, BLOCK_HEIGHT, rgb);
                canvas.draw_rect(x - 1, y - 1, BLOCK_WIDTH + 1, BLOCK_HEIGHT + 1, [0, 0, 0]);
            }
        }
        let image = canvas.to_rgb_image()?;
        // PaletteGenKt catches ImageIO.write failures, prints the exception, and returns normally.
        if let Err(error) = image.save(destination) {
            eprintln!("PaletteGenKt ImageIO.write failed: {error}");
        }
        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = (reader, rows, destination);
        Err("PaletteGenKt uses the Windows AWT-compatible text backend; this Rust build has no GDI backend".to_owned())
    }
}

/// PaletteGenKt.main ignores arguments and uses these fixed filenames.
pub(crate) fn source_main() -> Result<(), String> {
    gen_palette(Path::new("config/materials.json"), Path::new("palette.png"))
}

#[cfg(windows)]
fn resolve_font_path(reader: &crate::file_reader::FileReader, path: &Path) -> Option<PathBuf> {
    [
        path.to_path_buf(),
        reader.ss_home.join(path),
        reader.ss_resources.join(path),
        reader.bundled_resources.join(path),
    ]
    .into_iter()
    .find(|candidate| candidate.is_file())
}

#[cfg(windows)]
mod win_gdi {
    use std::ffi::c_void;
    pub type Handle = *mut c_void;
    #[repr(C)]
    pub struct BitmapInfoHeader {
        pub size: u32,
        pub width: i32,
        pub height: i32,
        pub planes: u16,
        pub bit_count: u16,
        pub compression: u32,
        pub size_image: u32,
        pub x_pixels_per_meter: i32,
        pub y_pixels_per_meter: i32,
        pub colors_used: u32,
        pub colors_important: u32,
    }
    #[repr(C)]
    pub struct RgbQuad {
        pub blue: u8,
        pub green: u8,
        pub red: u8,
        pub reserved: u8,
    }
    #[repr(C)]
    pub struct BitmapInfo {
        pub header: BitmapInfoHeader,
        pub colors: [RgbQuad; 1],
    }
    #[repr(C)]
    #[derive(Default)]
    pub struct Size {
        pub cx: i32,
        pub cy: i32,
    }
    #[repr(C)]
    #[derive(Default)]
    pub struct TextMetricW {
        pub height: i32,
        pub ascent: i32,
        pub descent: i32,
        pub internal_leading: i32,
        pub external_leading: i32,
        pub average_char_width: i32,
        pub maximum_char_width: i32,
        pub weight: i32,
        pub overhang: i32,
        pub digitized_aspect_x: i32,
        pub digitized_aspect_y: i32,
        pub first_char: u16,
        pub last_char: u16,
        pub default_char: u16,
        pub break_char: u16,
        pub italic: u8,
        pub underlined: u8,
        pub struck_out: u8,
        pub pitch_and_family: u8,
        pub char_set: u8,
    }
    #[link(name = "gdi32")]
    unsafe extern "system" {
        pub fn AddFontResourceExW(path: *const u16, flags: u32, reserved: *mut c_void) -> i32;
        pub fn RemoveFontResourceExW(path: *const u16, flags: u32, reserved: *mut c_void) -> i32;
        pub fn CreateCompatibleDC(dc: Handle) -> Handle;
        pub fn DeleteDC(dc: Handle) -> i32;
        pub fn CreateDIBSection(
            dc: Handle,
            info: *const BitmapInfo,
            usage: u32,
            bits: *mut *mut c_void,
            section: Handle,
            offset: u32,
        ) -> Handle;
        pub fn SelectObject(dc: Handle, object: Handle) -> Handle;
        pub fn DeleteObject(object: Handle) -> i32;
        pub fn CreateFontW(
            height: i32,
            width: i32,
            escapement: i32,
            orientation: i32,
            weight: i32,
            italic: u32,
            underline: u32,
            strike_out: u32,
            charset: u32,
            output_precision: u32,
            clip_precision: u32,
            quality: u32,
            pitch_and_family: u32,
            face: *const u16,
        ) -> Handle;
        pub fn SetTextColor(dc: Handle, color: u32) -> u32;
        pub fn SetBkMode(dc: Handle, mode: i32) -> i32;
        pub fn GetTextMetricsW(dc: Handle, metrics: *mut TextMetricW) -> i32;
        pub fn GetTextExtentPoint32W(
            dc: Handle,
            text: *const u16,
            count: i32,
            size: *mut Size,
        ) -> i32;
        pub fn TextOutW(dc: Handle, x: i32, y: i32, text: *const u16, count: i32) -> i32;
    }
}

#[cfg(windows)]
struct GdiCanvas {
    dc: win_gdi::Handle,
    bitmap: win_gdi::Handle,
    previous_bitmap: win_gdi::Handle,
    font: win_gdi::Handle,
    previous_font: win_gdi::Handle,
    bits: *mut u8,
    width: i32,
    height: i32,
    font_path: Vec<u16>,
    ascent: i32,
}

#[cfg(windows)]
impl GdiCanvas {
    fn new(width: i32, height: i32, font_path: &Path) -> Result<Self, String> {
        use std::os::windows::ffi::OsStrExt;
        use win_gdi::*;
        const FR_PRIVATE: u32 = 0x10;
        let font_path_wide: Vec<u16> = font_path.as_os_str().encode_wide().chain(Some(0)).collect();
        unsafe {
            if AddFontResourceExW(font_path_wide.as_ptr(), FR_PRIVATE, std::ptr::null_mut()) == 0 {
                return Err("GDI could not load the bundled Fira Sans font".to_owned());
            }
            let dc = CreateCompatibleDC(std::ptr::null_mut());
            if dc.is_null() {
                RemoveFontResourceExW(font_path_wide.as_ptr(), FR_PRIVATE, std::ptr::null_mut());
                return Err("GDI could not create a drawing context".to_owned());
            }
            let face: Vec<u16> = "Fira Sans".encode_utf16().chain(Some(0)).collect();
            let font = CreateFontW(-18, 0, 0, 0, 400, 0, 0, 0, 1, 4, 0, 4, 0, face.as_ptr());
            if font.is_null() {
                DeleteDC(dc);
                RemoveFontResourceExW(font_path_wide.as_ptr(), FR_PRIVATE, std::ptr::null_mut());
                return Err("GDI could not create the source palette font".to_owned());
            }
            let previous_font = SelectObject(dc, font);
            SetTextColor(dc, 0);
            SetBkMode(dc, 1);
            let mut metrics = TextMetricW::default();
            if GetTextMetricsW(dc, &mut metrics) == 0 {
                SelectObject(dc, previous_font);
                DeleteObject(font);
                DeleteDC(dc);
                RemoveFontResourceExW(font_path_wide.as_ptr(), FR_PRIVATE, std::ptr::null_mut());
                return Err("GDI could not read source palette font metrics".to_owned());
            }
            let mut canvas = Self {
                dc,
                bitmap: std::ptr::null_mut(),
                previous_bitmap: std::ptr::null_mut(),
                font,
                previous_font,
                bits: std::ptr::null_mut(),
                width: 0,
                height: 0,
                font_path: font_path_wide,
                ascent: metrics.ascent,
            };
            canvas.resize(width, height)?;
            Ok(canvas)
        }
    }
    fn ascent(&self) -> i32 {
        self.ascent
    }
    fn resize(&mut self, width: i32, height: i32) -> Result<(), String> {
        use win_gdi::*;
        if width <= 0 || height <= 0 {
            return Err("palette dimensions must be positive".to_owned());
        }
        unsafe {
            if !self.bitmap.is_null() {
                SelectObject(self.dc, self.previous_bitmap);
                DeleteObject(self.bitmap);
            }
            let info = BitmapInfo {
                header: BitmapInfoHeader {
                    size: std::mem::size_of::<BitmapInfoHeader>() as u32,
                    width,
                    height: -height,
                    planes: 1,
                    bit_count: 32,
                    compression: 0,
                    size_image: 0,
                    x_pixels_per_meter: 0,
                    y_pixels_per_meter: 0,
                    colors_used: 0,
                    colors_important: 0,
                },
                colors: [RgbQuad {
                    blue: 0,
                    green: 0,
                    red: 0,
                    reserved: 0,
                }],
            };
            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let bitmap = CreateDIBSection(self.dc, &info, 0, &mut bits, std::ptr::null_mut(), 0);
            if bitmap.is_null() || bits.is_null() {
                return Err("GDI could not allocate the palette bitmap".to_owned());
            }
            self.previous_bitmap = SelectObject(self.dc, bitmap);
            self.bitmap = bitmap;
            self.bits = bits.cast();
            self.width = width;
            self.height = height;
            std::ptr::write_bytes(self.bits, 0, width as usize * height as usize * 4);
            Ok(())
        }
    }
    fn text_width(&self, text: &str) -> i32 {
        use win_gdi::*;
        let wide: Vec<u16> = text.encode_utf16().collect();
        let mut size = Size::default();
        unsafe {
            GetTextExtentPoint32W(self.dc, wide.as_ptr(), wide.len() as i32, &mut size);
        }
        size.cx
    }
    fn text(&mut self, text: &str, x: i32, baseline: i32) {
        use win_gdi::*;
        let wide: Vec<u16> = text.encode_utf16().collect();
        unsafe {
            TextOutW(
                self.dc,
                x,
                baseline - self.ascent,
                wide.as_ptr(),
                wide.len() as i32,
            );
        }
    }
    fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32, rgb: [u8; 3]) {
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + width).min(self.width);
        let y1 = (y + height).min(self.height);
        for py in y0..y1 {
            for px in x0..x1 {
                self.set_pixel(px, py, rgb);
            }
        }
    }
    fn draw_rect(&mut self, x: i32, y: i32, width: i32, height: i32, rgb: [u8; 3]) {
        for px in x..=x + width {
            self.set_pixel(px, y, rgb);
            self.set_pixel(px, y + height, rgb);
        }
        for py in y..=y + height {
            self.set_pixel(x, py, rgb);
            self.set_pixel(x + width, py, rgb);
        }
    }
    fn set_pixel(&mut self, x: i32, y: i32, rgb: [u8; 3]) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        let pixel = unsafe {
            self.bits
                .add((y as usize * self.width as usize + x as usize) * 4)
        };
        unsafe {
            *pixel = rgb[2];
            *pixel.add(1) = rgb[1];
            *pixel.add(2) = rgb[0];
            *pixel.add(3) = 0;
        }
    }
    fn to_rgb_image(&self) -> Result<RgbImage, String> {
        if self.bits.is_null() {
            return Err("palette bitmap is missing".to_owned());
        }
        let mut image = RgbImage::new(self.width as u32, self.height as u32);
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = unsafe {
                    self.bits
                        .add((y as usize * self.width as usize + x as usize) * 4)
                };
                let bgr = unsafe { [*pixel, *pixel.add(1), *pixel.add(2)] };
                *image.get_pixel_mut(x as u32, y as u32) = Rgb([bgr[2], bgr[1], bgr[0]]);
            }
        }
        Ok(image)
    }
}

#[cfg(windows)]
impl Drop for GdiCanvas {
    fn drop(&mut self) {
        use win_gdi::*;
        unsafe {
            if !self.dc.is_null() {
                if !self.bitmap.is_null() {
                    SelectObject(self.dc, self.previous_bitmap);
                    DeleteObject(self.bitmap);
                }
                if !self.font.is_null() {
                    SelectObject(self.dc, self.previous_font);
                    DeleteObject(self.font);
                }
                DeleteDC(self.dc);
            }
            RemoveFontResourceExW(self.font_path.as_ptr(), 0x10, std::ptr::null_mut());
        }
    }
}
