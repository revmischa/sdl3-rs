use crate::sys;
use std::convert::TryFrom;
use std::mem::transmute;
use surface::SurfaceRef;
use crate::get_error;

pub struct Palette {
    raw: *mut sys::pixels::SDL_Palette,
}

impl Palette {
    #[inline]
    /// Creates a new, uninitialized palette
    #[doc(alias = "SDL_CreatePalette")]
    pub fn new(mut capacity: usize) -> Result<Self, String> {
        use crate::common::*;

        let ncolors = {
            // This is kind of a hack. We have to cast twice because
            // ncolors is a c_int, and validate_int only takes a u32.
            // FIXME: Modify validate_int to make this unnecessary
            let u32_max = u32::max_value() as usize;
            if capacity > u32_max {
                capacity = u32_max;
            }

            match validate_int(capacity as u32, "capacity") {
                Ok(len) => len,
                Err(e) => return Err(format!("{}", e)),
            }
        };

        let raw = unsafe { sys::pixels::SDL_CreatePalette(ncolors) };

        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Palette { raw })
        }
    }

    /// Creates a palette from the provided colors
    #[doc(alias = "SDL_SetPaletteColors")]
    pub fn with_colors(colors: &[Color]) -> Result<Self, String> {
        let pal = Self::new(colors.len())?;

        // Already validated, so don't check again
        let ncolors = colors.len() as ::libc::c_int;

        let result = unsafe {
            let mut raw_colors: Vec<sys::pixels::SDL_Color> =
                colors.iter().map(|color| color.raw()).collect();

            let pal_ptr = (&mut raw_colors[0]) as *mut sys::pixels::SDL_Color;

            sys::pixels::SDL_SetPaletteColors(pal.raw, pal_ptr, 0, ncolors)
        };

        if !result  {
            Err(get_error())
        } else {
            Ok(pal)
        }
    }

    pub fn len(&self) -> usize {
        unsafe { (*self.raw).ncolors as usize }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Drop for Palette {
    #[doc(alias = "SDL_DestroyPalette")]
    fn drop(&mut self) {
        unsafe {
            sys::pixels::SDL_DestroyPalette(self.raw);
        }
    }
}

impl_raw_accessors!((Palette, *mut sys::pixels::SDL_Palette));

#[test]
fn create_palette() {
    let colors: Vec<_> = (0..0xff).map(|u| Color::RGB(u, 0, 0xff - u)).collect();

    let palette = Palette::with_colors(&colors).unwrap();

    assert!(palette.len() == 255);
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    #[inline]
    #[allow(non_snake_case)]
    pub const fn RGB(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 0xff }
    }

    #[inline]
    #[allow(non_snake_case)]
    pub const fn RGBA(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }

    #[doc(alias = "SDL_MapRGBA")]
    pub fn to_u32(self, format: &PixelFormat) -> u32 {
        unsafe { sys::pixels::SDL_MapRGBA(format.pixel_format_details(),format.surface_palette(), self.r, self.g, self.b, self.a) }
    }

    #[doc(alias = "SDL_GetRGBA")]
    pub fn from_u32(format: &PixelFormat, pixel: u32) -> Color {
        let (mut r, mut g, mut b, mut a) = (0, 0, 0, 0);

        unsafe { sys::pixels::SDL_GetRGBA(pixel, format.pixel_format_details(),format.surface_palette(), &mut r, &mut g, &mut b, &mut a) };
        Color::RGBA(r, g, b, a)
    }

    pub fn invert(self) -> Color {
        Color::RGBA(255 - self.r, 255 - self.g, 255 - self.b, 255 - self.a)
    }

    #[inline]
    pub const fn rgb(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    #[inline]
    pub const fn rgba(self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }

    // Implemented manually and kept private, because reasons
    #[inline]
    const fn raw(self) -> sys::pixels::SDL_Color {
        sys::pixels::SDL_Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }

    pub const WHITE: Color = Color::RGBA(255, 255, 255, 255);
    pub const BLACK: Color = Color::RGBA(0, 0, 0, 255);
    pub const GRAY: Color = Color::RGBA(128, 128, 128, 255);
    pub const GREY: Color = Color::GRAY;
    pub const RED: Color = Color::RGBA(255, 0, 0, 255);
    pub const GREEN: Color = Color::RGBA(0, 255, 0, 255);
    pub const BLUE: Color = Color::RGBA(0, 0, 255, 255);
    pub const MAGENTA: Color = Color::RGBA(255, 0, 255, 255);
    pub const YELLOW: Color = Color::RGBA(255, 255, 0, 255);
    pub const CYAN: Color = Color::RGBA(0, 255, 255, 255);
}

impl Into<sys::pixels::SDL_Color> for Color {
    fn into(self) -> sys::pixels::SDL_Color {
        self.raw()
    }
}

impl From<sys::pixels::SDL_Color> for Color {
    fn from(raw: sys::pixels::SDL_Color) -> Color {
        Color::RGBA(raw.r, raw.g, raw.b, raw.a)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Color {
        Color::RGB(r, g, b)
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from((r, g, b, a): (u8, u8, u8, u8)) -> Color {
        Color::RGBA(r, g, b, a)
    }
}

pub struct PixelMasks {
    /// Bits per pixel; usually 15, 16, or 32
    pub bpp: u8,
    /// The red mask
    pub rmask: u32,
    /// The green mask
    pub gmask: u32,
    /// The blue mask
    pub bmask: u32,
    /// The alpha mask
    pub amask: u32,
}

pub struct PixelFormat {
    raw: *mut sys::pixels::SDL_PixelFormat,
    surface: *SurfaceRef,
}

impl_raw_accessors!((PixelFormat, *mut sys::pixels::SDL_PixelFormat));
impl_raw_constructor!((PixelFormat, PixelFormat(raw: *mut sys::pixels::SDL_PixelFormat, surface: *SurfaceRef)));

impl PixelFormat {
    pub unsafe fn pixel_format_details(&self) -> *sys::pixels::SDL_PixelFormatDetails {
        sys::pixels::SDL_GetPixelFormatDetails(*self.raw)
    }

    pub unsafe fn surface_palette(&self) -> *mut sys::pixels::SDL_Palette {
        sys::surface::SDL_GetSurfacePalette(self.surface.raw())
    }
}

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum PixelFormatEnum {
    Unknown = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_UNKNOWN as i32,
    Index1LSB = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_INDEX1LSB as i32,
    Index1MSB = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_INDEX1MSB as i32,
    Index4LSB = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_INDEX4LSB as i32,
    Index4MSB = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_INDEX4MSB as i32,
    Index8 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_INDEX8 as i32,
    RGB332 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGB332 as i32,
    RGB444 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGB444 as i32,
    RGB555 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGB555 as i32,
    BGR555 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_BGR555 as i32,
    ARGB4444 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_ARGB4444 as i32,
    RGBA4444 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGBA4444 as i32,
    ABGR4444 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_ABGR4444 as i32,
    BGRA4444 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_BGRA4444 as i32,
    ARGB1555 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_ARGB1555 as i32,
    RGBA5551 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGBA5551 as i32,
    ABGR1555 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_ABGR1555 as i32,
    BGRA5551 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_BGRA5551 as i32,
    RGB565 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGB565 as i32,
    BGR565 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_BGR565 as i32,
    RGB24 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGB24 as i32,
    BGR24 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_BGR24 as i32,
    XRGB8888 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_XRGB8888 as i32,
    RGBX8888 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGBX8888 as i32,
    XBGR8888 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_XBGR8888 as i32,
    BGRX8888 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_BGRX8888 as i32,
    ARGB8888 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_ARGB8888 as i32,
    RGBA8888 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGBA8888 as i32,
    ABGR8888 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_ABGR8888 as i32,
    BGRA8888 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_BGRA8888 as i32,
    ARGB2101010 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_ARGB2101010 as i32,
    YV12 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_YV12 as i32,
    IYUV = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_IYUV as i32,
    YUY2 = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_YUY2 as i32,
    UYVY = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_UYVY as i32,
    YVYU = sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_YVYU as i32,
}

// Endianness-agnostic aliases for 32-bit formats
#[cfg(target_endian = "big")]
impl PixelFormatEnum {
    pub const RGBA32: PixelFormatEnum = PixelFormatEnum::RGBA8888;
    pub const ARGB32: PixelFormatEnum = PixelFormatEnum::ARGB8888;
    pub const BGRA32: PixelFormatEnum = PixelFormatEnum::BGRA8888;
    pub const ABGR32: PixelFormatEnum = PixelFormatEnum::ABGR8888;
}

#[cfg(target_endian = "little")]
impl PixelFormatEnum {
    pub const RGBA32: PixelFormatEnum = PixelFormatEnum::ABGR8888;
    pub const ARGB32: PixelFormatEnum = PixelFormatEnum::BGRA8888;
    pub const BGRA32: PixelFormatEnum = PixelFormatEnum::ARGB8888;
    pub const ABGR32: PixelFormatEnum = PixelFormatEnum::RGBA8888;
}

impl PixelFormatEnum {
    #[doc(alias = "SDL_GetPixelFormatForMasks")]
    pub fn from_masks(masks: PixelMasks) -> PixelFormatEnum {
        unsafe {
            let format = sys::pixels::SDL_GetPixelFormatForMasks(
                masks.bpp as i32,
                masks.rmask,
                masks.gmask,
                masks.bmask,
                masks.amask,
            );
            PixelFormatEnum::try_from(format).unwrap()
        }
    }

    #[doc(alias = "SDL_GetMasksForPixelFormat")]
    pub fn into_masks(self) -> Result<PixelMasks, String> {
        let format: u32 = self as u32;
        let mut bpp = 0;
        let mut rmask = 0;
        let mut gmask = 0;
        let mut bmask = 0;
        let mut amask = 0;
        let result = unsafe {
            sys::pixels::SDL_GetMasksForPixelFormat(
                format.into(), &mut bpp, &mut rmask, &mut gmask, &mut bmask, &mut amask,
            )
        };
        if !result  {
            Err(get_error())
        } else {
            Ok(PixelMasks {
                bpp: bpp as u8,
                rmask,
                gmask,
                bmask,
                amask,
            })
        }
    }

    /// Calculates the total byte size of an image buffer, given its pitch
    /// and height.
    pub fn byte_size_from_pitch_and_height(self, pitch: usize, height: usize) -> usize {
        match self {
            PixelFormatEnum::YV12 | PixelFormatEnum::IYUV => {
                // YUV is 4:2:0.
                // `pitch` is the width of the Y component, and
                // `height` is the height of the Y component.
                // U and V have half the width and height of Y.
                pitch * height + 2 * (pitch / 2 * height / 2)
            }
            _ => pitch * height,
        }
    }

    #[allow(clippy::match_same_arms)]
    pub fn byte_size_of_pixels(self, num_of_pixels: usize) -> usize {
        match self {
            PixelFormatEnum::RGB332 => num_of_pixels,
            PixelFormatEnum::RGB444
            | PixelFormatEnum::RGB555
            | PixelFormatEnum::BGR555
            | PixelFormatEnum::ARGB4444
            | PixelFormatEnum::RGBA4444
            | PixelFormatEnum::ABGR4444
            | PixelFormatEnum::BGRA4444
            | PixelFormatEnum::ARGB1555
            | PixelFormatEnum::RGBA5551
            | PixelFormatEnum::ABGR1555
            | PixelFormatEnum::BGRA5551
            | PixelFormatEnum::RGB565
            | PixelFormatEnum::BGR565 => num_of_pixels * 2,
            PixelFormatEnum::RGB24 | PixelFormatEnum::BGR24 => num_of_pixels * 3,
            PixelFormatEnum::XRGB8888
            | PixelFormatEnum::RGBX8888
            | PixelFormatEnum::XBGR8888
            | PixelFormatEnum::BGRX8888
            | PixelFormatEnum::ARGB8888
            | PixelFormatEnum::RGBA8888
            | PixelFormatEnum::ABGR8888
            | PixelFormatEnum::BGRA8888
            | PixelFormatEnum::ARGB2101010 => num_of_pixels * 4,
            // YUV formats
            // FIXME: rounding error here?
            PixelFormatEnum::YV12 | PixelFormatEnum::IYUV => num_of_pixels / 2 * 3,
            PixelFormatEnum::YUY2 | PixelFormatEnum::UYVY | PixelFormatEnum::YVYU => {
                num_of_pixels * 2
            }
            // Unsupported formats
            PixelFormatEnum::Index8 => num_of_pixels,
            PixelFormatEnum::Unknown
            | PixelFormatEnum::Index1LSB
            | PixelFormatEnum::Index1MSB
            | PixelFormatEnum::Index4LSB
            | PixelFormatEnum::Index4MSB => panic!("not supported format: {:?}", self),
        }
    }

    #[allow(clippy::match_same_arms)]
    pub fn byte_size_per_pixel(self) -> usize {
        match self {
            PixelFormatEnum::RGB332 => 1,
            PixelFormatEnum::RGB444
            | PixelFormatEnum::RGB555
            | PixelFormatEnum::BGR555
            | PixelFormatEnum::ARGB4444
            | PixelFormatEnum::RGBA4444
            | PixelFormatEnum::ABGR4444
            | PixelFormatEnum::BGRA4444
            | PixelFormatEnum::ARGB1555
            | PixelFormatEnum::RGBA5551
            | PixelFormatEnum::ABGR1555
            | PixelFormatEnum::BGRA5551
            | PixelFormatEnum::RGB565
            | PixelFormatEnum::BGR565 => 2,
            PixelFormatEnum::RGB24 | PixelFormatEnum::BGR24 => 3,
            PixelFormatEnum::XRGB8888
            | PixelFormatEnum::RGBX8888
            | PixelFormatEnum::XBGR8888
            | PixelFormatEnum::BGRX8888
            | PixelFormatEnum::ARGB8888
            | PixelFormatEnum::RGBA8888
            | PixelFormatEnum::ABGR8888
            | PixelFormatEnum::BGRA8888
            | PixelFormatEnum::ARGB2101010 => 4,
            // YUV formats
            PixelFormatEnum::YV12 | PixelFormatEnum::IYUV => 1,
            PixelFormatEnum::YUY2 | PixelFormatEnum::UYVY | PixelFormatEnum::YVYU => 2,
            // Unsupported formats
            PixelFormatEnum::Index8 => 1,
            PixelFormatEnum::Unknown
            | PixelFormatEnum::Index1LSB
            | PixelFormatEnum::Index1MSB
            | PixelFormatEnum::Index4LSB
            | PixelFormatEnum::Index4MSB => panic!("not supported format: {:?}", self),
        }
    }

    pub fn supports_alpha(self) -> bool {
        use crate::pixels::PixelFormatEnum::*;
        match self {
            ARGB4444 | ARGB1555 | ARGB8888 | ARGB2101010 | ABGR4444 | ABGR1555 | ABGR8888
            | BGRA4444 | BGRA5551 | BGRA8888 | RGBA4444 | RGBA5551 | RGBA8888 => true,
            _ => false,
        }
    }
}

impl From<PixelFormat> for PixelFormatEnum {
    fn from(pf: PixelFormat) -> PixelFormatEnum {
        unsafe {
            let sdl_pf = *pf.raw;
            match PixelFormatEnum::try_from(sdl_pf.format()) {
                Ok(pfe) => pfe,
                Err(()) => panic!("Unknown pixel format: {:?}", sdl_pf.format().into()),
            }
        }
    }
}

impl TryFrom<u32> for PixelFormatEnum {
    type Error = ();

    fn try_from(n: u32) -> Result<Self, Self::Error> {
        use self::PixelFormatEnum::*;

        Ok(match unsafe { transmute(n) } {
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_UNKNOWN => Unknown,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_INDEX1LSB => Index1LSB,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_INDEX1MSB => Index1MSB,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_INDEX4LSB => Index4LSB,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_INDEX4MSB => Index4MSB,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_INDEX8 => Index8,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGB332 => RGB332,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGB444 => RGB444,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGB555 => RGB555,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_BGR555 => BGR555,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_ARGB4444 => ARGB4444,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGBA4444 => RGBA4444,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_ABGR4444 => ABGR4444,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_BGRA4444 => BGRA4444,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_ARGB1555 => ARGB1555,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGBA5551 => RGBA5551,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_ABGR1555 => ABGR1555,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_BGRA5551 => BGRA5551,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGB565 => RGB565,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_BGR565 => BGR565,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGB24 => RGB24,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_BGR24 => BGR24,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_XRGB8888 => XRGB8888,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGBX8888 => RGBX8888,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_XBGR8888 => XBGR8888,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_BGRX8888 => BGRX8888,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_ARGB8888 => ARGB8888,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_RGBA8888 => RGBA8888,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_ABGR8888 => ABGR8888,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_BGRA8888 => BGRA8888,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_ARGB2101010 => ARGB2101010,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_YV12 => YV12,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_IYUV => IYUV,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_YUY2 => YUY2,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_UYVY => UYVY,
            sys::pixels::SDL_PixelFormat::SDL_PIXELFORMAT_YVYU => YVYU,
            _ => return Err(()),
        })
    }
}

impl TryFrom<PixelFormatEnum> for PixelFormat {
    type Error = String;

    #[doc(alias = "SDL_GetPixelFormatDetails")]
    fn try_from(pfe: PixelFormatEnum) -> Result<Self, Self::Error> {
        unsafe {
            let pf_ptr = sys::pixels::SDL_GetPixelFormatDetails(pfe.into());
            let surface = SurfaceRef::null();
            if pf_ptr.is_null() {
                Err(get_error())
            } else {
                // not sure what surface should be here
                Ok(PixelFormat::from_ll(pf_ptr.format(), surface))
            }
        }
    }
}

// Just test a round-trip conversion from PixelFormat to
// PixelFormatEnum and back.
#[test]
fn test_pixel_format_enum() {
    let pixel_formats = vec![
        PixelFormatEnum::RGB332,
        PixelFormatEnum::RGB444,
        PixelFormatEnum::RGB555,
        PixelFormatEnum::BGR555,
        PixelFormatEnum::ARGB4444,
        PixelFormatEnum::RGBA4444,
        PixelFormatEnum::ABGR4444,
        PixelFormatEnum::BGRA4444,
        PixelFormatEnum::ARGB1555,
        PixelFormatEnum::RGBA5551,
        PixelFormatEnum::ABGR1555,
        PixelFormatEnum::BGRA5551,
        PixelFormatEnum::RGB565,
        PixelFormatEnum::BGR565,
        PixelFormatEnum::RGB24,
        PixelFormatEnum::BGR24,
        PixelFormatEnum::XRGB8888,
        PixelFormatEnum::RGBX8888,
        PixelFormatEnum::XBGR8888,
        PixelFormatEnum::BGRX8888,
        PixelFormatEnum::ARGB8888,
        PixelFormatEnum::RGBA8888,
        PixelFormatEnum::ABGR8888,
        PixelFormatEnum::BGRA8888,
        PixelFormatEnum::ARGB2101010,
        PixelFormatEnum::Index8,
        PixelFormatEnum::Unknown,
        //  These formats don't seem to survive the round-trip on all platforms;
        //PixelFormatEnum::YV12,
        //PixelFormatEnum::IYUV,
        //PixelFormatEnum::YUY2,
        //PixelFormatEnum::UYVY,
        //PixelFormatEnum::YVYU,
	//PixelFormatEnum::Index1LSB,
        //PixelFormatEnum::Index1MSB,
	//PixelFormatEnum::Index4LSB,
        //PixelFormatEnum::Index4MSB
    ];

    let _sdl_context = crate::sdl::init().unwrap();
    for format in pixel_formats {
        // If we don't support making a surface of a specific format,
        // that's fine, just keep going the best we can.
        if let Ok(surf) = super::surface::Surface::new(1, 1, format) {
            let surf_format = surf.pixel_format();
            assert_eq!(PixelFormatEnum::from(surf_format), format);
        }
    }
}
