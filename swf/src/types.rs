//! The data structures used in an Adobe SWF file.
//!
//! These structures are documented in the Adobe SWF File Foramt Specification
//! version 19 (henceforth SWF19):
//! https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf
use enumset::{EnumSet, EnumSetType};
use std::collections::HashSet;
use std::io::Read;

mod matrix;

pub use matrix::Matrix;

/// A complete header and tags in the SWF file.
/// This is returned by the `swf::read_swf` convenience method.
#[derive(Debug, PartialEq)]
pub struct Swf {
    pub header: Header,
    pub tags: Vec<Tag>,
}

/// Returned by `read::read_swf_header`. Includes the decompress
/// stream as well as the uncompressed data length.
pub struct SwfStream<'a> {
    pub header: Header,
    pub uncompressed_length: usize,
    pub decompressed_reader: Box<dyn Read + 'a>,
}

/// The header of an SWF file.
///
/// Notably contains the compression format used by the rest of the SWF data.
///
/// [SWF19 p.27](https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf#page=27)
#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    pub version: u8,
    pub compression: Compression,
    pub stage_size: Rectangle,
    pub frame_rate: f32,
    pub num_frames: u16,
}

/// The compression foramt used internally by the SWF file.
///
/// The vast majority of SWFs will use zlib compression.
/// [SWF19 p.27](https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf#page=27)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Compression {
    None,
    Zlib,
    Lzma,
}

/// Most coordinates in an SWF file are represented in "twips".
/// A twip is 1/20th of a pixel.
///
/// `Twips` is a type-safe wrapper type documenting where Twips are used
/// in the SWF format.
///
/// Use `Twips::from_pixels` and `Twips::to_pixels` to convert to and from
/// pixel values.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, PartialOrd, Ord)]
pub struct Twips(i32);

impl Twips {
    /// There are 20 twips in a pixel.
    pub const TWIPS_PER_PIXEL: f64 = 20.0;

    /// Creates a new `Twips` object. Note that the `twips` value is in twips,
    /// not pixels. Use `from_pixels` to convert from pixel units.
    pub fn new<T: Into<i32>>(twips: T) -> Self {
        Self(twips.into())
    }

    /// Creates a new `Twips` object set to the value of 0.
    pub fn zero() -> Self {
        Self(0)
    }

    /// Returns the number of twips.
    pub fn get(self) -> i32 {
        self.0
    }

    /// Converts the number of pixels into twips.
    ///
    /// This may be a lossy conversion; any precision less than a twip (1/20 pixels) is truncated.
    pub fn from_pixels(pixels: f64) -> Self {
        Self((pixels * Self::TWIPS_PER_PIXEL) as i32)
    }

    /// Converts this twips value into pixel units.
    ///
    /// This is a lossless operation.
    pub fn to_pixels(self) -> f64 {
        f64::from(self.0) / Self::TWIPS_PER_PIXEL
    }
}

impl std::ops::Add for Twips {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::AddAssign for Twips {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0
    }
}

impl std::ops::Sub for Twips {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl std::ops::SubAssign for Twips {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0
    }
}

impl std::ops::Mul<i32> for Twips {
    type Output = Self;
    fn mul(self, other: i32) -> Self {
        Self(self.0 * other)
    }
}

impl std::ops::MulAssign<i32> for Twips {
    fn mul_assign(&mut self, other: i32) {
        self.0 *= other
    }
}

impl std::ops::Div<i32> for Twips {
    type Output = Self;
    fn div(self, other: i32) -> Self {
        Self(self.0 / other)
    }
}

impl std::ops::DivAssign<i32> for Twips {
    fn div_assign(&mut self, other: i32) {
        self.0 /= other
    }
}

impl std::fmt::Display for Twips {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_pixels())
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Rectangle {
    pub x_min: Twips,
    pub x_max: Twips,
    pub y_min: Twips,
    pub y_max: Twips,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn from_rgb(rgb: u32, alpha: u8) -> Self {
        Self {
            r: ((rgb & 0xFF_0000) >> 16) as u8,
            g: ((rgb & 0x00_FF00) >> 8) as u8,
            b: (rgb & 0x00_00FF) as u8,
            a: alpha,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ColorTransform {
    pub r_multiply: f32,
    pub g_multiply: f32,
    pub b_multiply: f32,
    pub a_multiply: f32,
    pub r_add: i16,
    pub g_add: i16,
    pub b_add: i16,
    pub a_add: i16,
}

impl ColorTransform {
    pub fn new() -> ColorTransform {
        ColorTransform {
            r_multiply: 1f32,
            g_multiply: 1f32,
            b_multiply: 1f32,
            a_multiply: 1f32,
            r_add: 0,
            g_add: 0,
            b_add: 0,
            a_add: 0,
        }
    }
}

impl Default for ColorTransform {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    Unknown,
    Latin,
    Japanese,
    Korean,
    SimplifiedChinese,
    TraditionalChinese,
}

#[derive(Debug, PartialEq)]
pub struct FileAttributes {
    pub use_direct_blit: bool,
    pub use_gpu: bool,
    pub has_metadata: bool,
    pub is_action_script_3: bool,
    pub use_network_sandbox: bool,
}

#[derive(Debug, PartialEq)]
pub struct FrameLabel {
    pub label: String,
    pub is_anchor: bool,
}

#[derive(Debug, PartialEq)]
pub struct DefineSceneAndFrameLabelData {
    pub scenes: Vec<FrameLabelData>,
    pub frame_labels: Vec<FrameLabelData>,
}

#[derive(Debug, PartialEq)]
pub struct FrameLabelData {
    pub frame_num: u32,
    pub label: String,
}

pub type Depth = u16;
pub type CharacterId = u16;

#[derive(Debug, PartialEq)]
pub struct PlaceObject {
    pub version: u8,
    pub action: PlaceObjectAction,
    pub depth: Depth,
    pub matrix: Option<Matrix>,
    pub color_transform: Option<ColorTransform>,
    pub ratio: Option<u16>,
    pub name: Option<String>,
    pub clip_depth: Option<Depth>,
    pub class_name: Option<String>,
    pub filters: Vec<Filter>,
    pub background_color: Option<Color>,
    pub blend_mode: BlendMode,
    pub clip_actions: Vec<ClipAction>,
    pub is_image: bool,
    pub is_bitmap_cached: bool,
    pub is_visible: bool,
    pub amf_data: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PlaceObjectAction {
    Place(CharacterId),
    Modify,
    Replace(CharacterId),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Filter {
    DropShadowFilter(Box<DropShadowFilter>),
    BlurFilter(Box<BlurFilter>),
    GlowFilter(Box<GlowFilter>),
    BevelFilter(Box<BevelFilter>),
    GradientGlowFilter(Box<GradientGlowFilter>),
    ConvolutionFilter(Box<ConvolutionFilter>),
    ColorMatrixFilter(Box<ColorMatrixFilter>),
    GradientBevelFilter(Box<GradientBevelFilter>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct DropShadowFilter {
    pub color: Color,
    pub blur_x: f64,
    pub blur_y: f64,
    pub angle: f64,
    pub distance: f64,
    pub strength: f32,
    pub is_inner: bool,
    pub is_knockout: bool,
    pub num_passes: u8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlurFilter {
    pub blur_x: f64,
    pub blur_y: f64,
    pub num_passes: u8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GlowFilter {
    pub color: Color,
    pub blur_x: f64,
    pub blur_y: f64,
    pub strength: f32,
    pub is_inner: bool,
    pub is_knockout: bool,
    pub num_passes: u8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BevelFilter {
    pub shadow_color: Color,
    pub highlight_color: Color,
    pub blur_x: f64,
    pub blur_y: f64,
    pub angle: f64,
    pub distance: f64,
    pub strength: f32,
    pub is_inner: bool,
    pub is_knockout: bool,
    pub is_on_top: bool,
    pub num_passes: u8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GradientGlowFilter {
    pub colors: Vec<GradientRecord>,
    pub blur_x: f64,
    pub blur_y: f64,
    pub angle: f64,
    pub distance: f64,
    pub strength: f32,
    pub is_inner: bool,
    pub is_knockout: bool,
    pub is_on_top: bool,
    pub num_passes: u8,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ConvolutionFilter {
    pub num_matrix_rows: u8,
    pub num_matrix_cols: u8,
    pub matrix: Vec<f64>,
    pub divisor: f64,
    pub bias: f64,
    pub default_color: Color,
    pub is_clamped: bool,
    pub is_preserve_alpha: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ColorMatrixFilter {
    pub matrix: [f64; 20],
}

#[derive(Debug, PartialEq, Clone)]
pub struct GradientBevelFilter {
    pub colors: Vec<GradientRecord>,
    pub blur_x: f64,
    pub blur_y: f64,
    pub angle: f64,
    pub distance: f64,
    pub strength: f32,
    pub is_inner: bool,
    pub is_knockout: bool,
    pub is_on_top: bool,
    pub num_passes: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlendMode {
    Normal,
    Layer,
    Multiply,
    Screen,
    Lighten,
    Darken,
    Difference,
    Add,
    Subtract,
    Invert,
    Alpha,
    Erase,
    Overlay,
    HardLight,
}

/// An clip action (a.k.a. clip event) placed on a movieclip instance.
/// Created in the Flash IDE using `onClipEvent` or `on` blocks.
///
/// [SWF19 pp.37-38 ClipActionRecord](https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf#page=37)
#[derive(Debug, Clone, PartialEq)]
pub struct ClipAction {
    pub events: EnumSet<ClipEventFlag>,
    pub key_code: Option<KeyCode>,
    pub action_data: Vec<u8>,
}

/// An event that can be attached to a movieclip instance using
/// an `onClipEvent` or `on` block.
///
/// [SWF19 pp.48-50 ClipEvent](https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf#page=38)
#[derive(Debug, EnumSetType)]
pub enum ClipEventFlag {
    Construct,
    Data,
    DragOut,
    DragOver,
    EnterFrame,
    Initialize,
    KeyUp,
    KeyDown,
    KeyPress,
    Load,
    MouseUp,
    MouseDown,
    MouseMove,
    Press,
    RollOut,
    RollOver,
    Release,
    ReleaseOutside,
    Unload,
}

/// A key code used in `ButtonAction` and `ClipAction` key press events.
pub type KeyCode = u8;

/// Represents a tag in an SWF file.
///
/// The SWF format is made up of a stream of tags. Each tag either
/// defines a character (graphic, sound, movieclip), or places/modifies
/// an instance of these characters on the display list.
///
// [SWF19 p.29](https://www.adobe.com/content/dam/acom/en/devnet/pdf/swf-file-format-spec.pdf#page=29)
#[derive(Debug, PartialEq)]
pub enum Tag {
    ExportAssets(ExportAssets),
    ScriptLimits {
        max_recursion_depth: u16,
        timeout_in_seconds: u16,
    },
    ShowFrame,

    Protect(Option<String>),
    CsmTextSettings(CsmTextSettings),
    DebugId(DebugId),
    DefineBinaryData {
        id: CharacterId,
        data: Vec<u8>,
    },
    DefineBits {
        id: CharacterId,
        jpeg_data: Vec<u8>,
    },
    DefineBitsJpeg2 {
        id: CharacterId,
        jpeg_data: Vec<u8>,
    },
    DefineBitsJpeg3(DefineBitsJpeg3),
    DefineBitsLossless(DefineBitsLossless),
    DefineButton(Box<Button>),
    DefineButton2(Box<Button>),
    DefineButtonColorTransform(ButtonColorTransform),
    DefineButtonSound(Box<ButtonSounds>),
    DefineEditText(Box<EditText>),
    DefineFont(Box<FontV1>),
    DefineFont2(Box<Font>),
    DefineFont4(Font4),
    DefineFontAlignZones {
        id: CharacterId,
        thickness: FontThickness,
        zones: Vec<FontAlignZone>,
    },
    DefineFontInfo(Box<FontInfo>),
    DefineFontName {
        id: CharacterId,
        name: String,
        copyright_info: String,
    },
    DefineMorphShape(Box<DefineMorphShape>),
    DefineScalingGrid {
        id: CharacterId,
        splitter_rect: Rectangle,
    },
    DefineShape(Shape),
    DefineSound(Box<Sound>),
    DefineSprite(Sprite),
    DefineText(Box<Text>),
    DefineVideoStream(DefineVideoStream),
    DoAbc(DoAbc),
    DoAction(DoAction),
    DoInitAction {
        id: CharacterId,
        action_data: Vec<u8>,
    },
    EnableDebugger(String),
    EnableTelemetry {
        password_hash: Vec<u8>,
    },
    End,
    Metadata(String),
    ImportAssets {
        url: String,
        imports: Vec<ExportedAsset>,
    },
    JpegTables(JpegTables),
    SetBackgroundColor(SetBackgroundColor),
    SetTabIndex {
        depth: Depth,
        tab_index: u16,
    },
    SoundStreamBlock(SoundStreamBlock),
    SoundStreamHead(Box<SoundStreamHead>),
    SoundStreamHead2(Box<SoundStreamHead>),
    StartSound(StartSound),
    StartSound2 {
        class_name: String,
        sound_info: Box<SoundInfo>,
    },
    SymbolClass(Vec<SymbolClassLink>),
    PlaceObject(Box<PlaceObject>),
    RemoveObject(RemoveObject),
    VideoFrame(VideoFrame),
    FileAttributes(FileAttributes),

    FrameLabel(FrameLabel),
    DefineSceneAndFrameLabelData(DefineSceneAndFrameLabelData),

    ProductInfo(ProductInfo),

    Unknown {
        tag_code: u16,
        data: Vec<u8>,
    },
}

pub type ExportAssets = Vec<ExportedAsset>;

#[derive(Debug, PartialEq, Clone)]
pub struct ExportedAsset {
    pub id: CharacterId,
    pub name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RemoveObject {
    pub depth: Depth,
    pub character_id: Option<CharacterId>,
}

pub type SetBackgroundColor = Color;

#[derive(Debug, PartialEq, Clone)]
pub struct SymbolClassLink {
    pub id: CharacterId,
    pub class_name: String,
}

#[derive(Debug, PartialEq)]
pub struct Shape {
    pub version: u8,
    pub id: CharacterId,
    pub shape_bounds: Rectangle,
    pub edge_bounds: Rectangle,
    pub has_fill_winding_rule: bool,
    pub has_non_scaling_strokes: bool,
    pub has_scaling_strokes: bool,
    pub styles: ShapeStyles,
    pub shape: Vec<ShapeRecord>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Sound {
    pub id: CharacterId,
    pub format: SoundFormat,
    pub num_samples: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SoundInfo {
    pub event: SoundEvent,
    pub in_sample: Option<u32>,
    pub out_sample: Option<u32>,
    pub num_loops: u16,
    pub envelope: Option<SoundEnvelope>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SoundEvent {
    Event,
    Start,
    Stop,
}

pub type SoundEnvelope = Vec<SoundEnvelopePoint>;

#[derive(Debug, PartialEq, Clone)]
pub struct SoundEnvelopePoint {
    pub sample: u32,
    pub left_volume: f32,
    pub right_volume: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StartSound {
    pub id: CharacterId,
    pub sound_info: Box<SoundInfo>,
}

#[derive(Debug, PartialEq)]
pub struct Sprite {
    pub id: CharacterId,
    pub num_frames: u16,
    pub tags: Vec<Tag>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShapeStyles {
    pub fill_styles: Vec<FillStyle>,
    pub line_styles: Vec<LineStyle>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ShapeRecord {
    StyleChange(StyleChangeData),
    StraightEdge {
        delta_x: Twips,
        delta_y: Twips,
    },
    CurvedEdge {
        control_delta_x: Twips,
        control_delta_y: Twips,
        anchor_delta_x: Twips,
        anchor_delta_y: Twips,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct StyleChangeData {
    pub move_to: Option<(Twips, Twips)>,
    pub fill_style_0: Option<u32>,
    pub fill_style_1: Option<u32>,
    pub line_style: Option<u32>,
    pub new_styles: Option<ShapeStyles>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FillStyle {
    Color(Color),
    LinearGradient(Gradient),
    RadialGradient(Gradient),
    FocalGradient {
        gradient: Gradient,
        focal_point: f32,
    },
    Bitmap {
        id: CharacterId,
        matrix: Matrix,
        is_smoothed: bool,
        is_repeating: bool,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Gradient {
    pub matrix: Matrix,
    pub spread: GradientSpread,
    pub interpolation: GradientInterpolation,
    pub records: Vec<GradientRecord>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GradientSpread {
    Pad,
    Reflect,
    Repeat,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GradientInterpolation {
    RGB,
    LinearRGB,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GradientRecord {
    pub ratio: u8,
    pub color: Color,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LineStyle {
    pub width: Twips,
    pub color: Color,
    pub start_cap: LineCapStyle,
    pub end_cap: LineCapStyle,
    pub join_style: LineJoinStyle,
    pub fill_style: Option<FillStyle>,
    pub allow_scale_x: bool,
    pub allow_scale_y: bool,
    pub is_pixel_hinted: bool,
    pub allow_close: bool,
}

impl LineStyle {
    pub fn new_v1(width: Twips, color: Color) -> LineStyle {
        LineStyle {
            width,
            color,
            start_cap: LineCapStyle::Round,
            end_cap: LineCapStyle::Round,
            join_style: LineJoinStyle::Round,
            fill_style: None,
            allow_scale_x: false,
            allow_scale_y: false,
            is_pixel_hinted: false,
            allow_close: true,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LineCapStyle {
    Round,
    None,
    Square,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LineJoinStyle {
    Round,
    Bevel,
    Miter(f32),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AudioCompression {
    UncompressedUnknownEndian,
    Adpcm,
    Mp3,
    Uncompressed,
    Nellymoser16Khz,
    Nellymoser8Khz,
    Nellymoser,
    Speex,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SoundFormat {
    pub compression: AudioCompression,
    pub sample_rate: u16,
    pub is_stereo: bool,
    pub is_16_bit: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SoundStreamHead {
    pub stream_format: SoundFormat,
    pub playback_format: SoundFormat,
    pub num_samples_per_block: u16,
    pub latency_seek: i16,
}

pub type SoundStreamBlock = Vec<u8>;

#[derive(Debug, PartialEq, Clone)]
pub struct Button {
    pub id: CharacterId,
    pub is_track_as_menu: bool,
    pub records: Vec<ButtonRecord>,
    pub actions: Vec<ButtonAction>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ButtonRecord {
    pub states: HashSet<ButtonState>,
    pub id: CharacterId,
    pub depth: Depth,
    pub matrix: Matrix,
    pub color_transform: ColorTransform,
    pub filters: Vec<Filter>,
    pub blend_mode: BlendMode,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ButtonState {
    Up,
    Over,
    Down,
    HitTest,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ButtonColorTransform {
    pub id: CharacterId,
    pub color_transforms: Vec<ColorTransform>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ButtonSounds {
    pub id: CharacterId,
    pub over_to_up_sound: Option<ButtonSound>,
    pub up_to_over_sound: Option<ButtonSound>,
    pub over_to_down_sound: Option<ButtonSound>,
    pub down_to_over_sound: Option<ButtonSound>,
}

pub type ButtonSound = (CharacterId, SoundInfo);

#[derive(Debug, PartialEq, Clone)]
pub struct ButtonAction {
    pub conditions: HashSet<ButtonActionCondition>,
    pub key_code: Option<u8>,
    pub action_data: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ButtonActionCondition {
    IdleToOverDown,
    OutDownToIdle,
    OutDownToOverDown,
    OverDownToOutDown,
    OverDownToOverUp,
    OverUpToOverDown,
    OverUpToIdle,
    IdleToOverUp,
    OverDownToIdle,
    KeyPress,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DefineMorphShape {
    pub version: u8,
    pub id: CharacterId,
    pub has_non_scaling_strokes: bool,
    pub has_scaling_strokes: bool,
    pub start: MorphShape,
    pub end: MorphShape,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MorphShape {
    pub shape_bounds: Rectangle,
    pub edge_bounds: Rectangle,
    pub fill_styles: Vec<FillStyle>,
    pub line_styles: Vec<LineStyle>,
    pub shape: Vec<ShapeRecord>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontV1 {
    pub id: CharacterId,
    pub glyphs: Vec<Vec<ShapeRecord>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Font {
    pub version: u8,
    pub id: CharacterId,
    pub name: String,
    pub language: Language,
    pub layout: Option<FontLayout>,
    pub glyphs: Vec<Glyph>,
    pub is_small_text: bool,
    pub is_shift_jis: bool, // TODO(Herschel): Use enum for Shift-JIS/ANSI/UCS-2
    pub is_ansi: bool,
    pub is_bold: bool,
    pub is_italic: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Font4 {
    pub id: CharacterId,
    pub is_italic: bool,
    pub is_bold: bool,
    pub name: String,
    pub data: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub shape_records: Vec<ShapeRecord>,
    pub code: u16,
    pub advance: Option<i16>,
    pub bounds: Option<Rectangle>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontLayout {
    pub ascent: u16,
    pub descent: u16,
    pub leading: i16,
    pub kerning: Vec<KerningRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KerningRecord {
    pub left_code: u16,
    pub right_code: u16,
    pub adjustment: Twips,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontInfo {
    pub id: CharacterId,
    pub version: u8,
    pub name: String,
    pub is_small_text: bool,
    pub is_shift_jis: bool,
    pub is_ansi: bool,
    pub is_bold: bool,
    pub is_italic: bool,
    pub language: Language,
    pub code_table: Vec<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Text {
    pub id: CharacterId,
    pub bounds: Rectangle,
    pub matrix: Matrix,
    pub records: Vec<TextRecord>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextRecord {
    pub font_id: Option<CharacterId>,
    pub color: Option<Color>,
    pub x_offset: Option<Twips>,
    pub y_offset: Option<Twips>,
    pub height: Option<Twips>,
    pub glyphs: Vec<GlyphEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlyphEntry {
    pub index: u32,
    pub advance: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EditText {
    pub id: CharacterId,
    pub bounds: Rectangle,
    pub font_id: Option<CharacterId>, // TODO(Herschel): Combine with height
    pub font_class_name: Option<String>,
    pub height: Option<Twips>,
    pub color: Option<Color>,
    pub max_length: Option<u16>,
    pub layout: Option<TextLayout>,
    pub variable_name: String,
    pub initial_text: Option<String>,
    pub is_word_wrap: bool,
    pub is_multiline: bool,
    pub is_password: bool,
    pub is_read_only: bool,
    pub is_auto_size: bool,
    pub is_selectable: bool,
    pub has_border: bool,
    pub was_static: bool,
    pub is_html: bool,
    pub is_device_font: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextLayout {
    pub align: TextAlign,
    pub left_margin: Twips,
    pub right_margin: Twips,
    pub indent: Twips,
    pub leading: Twips,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontAlignZone {
    // TODO(Herschel): Read these as f16s.
    pub left: i16,
    pub width: i16,
    pub bottom: i16,
    pub height: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontThickness {
    Thin,
    Medium,
    Thick,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CsmTextSettings {
    pub id: CharacterId,
    pub use_advanced_rendering: bool,
    pub grid_fit: TextGridFit,
    pub thickness: f32, // TODO(Herschel): 0.0 is default. Should be Option?
    pub sharpness: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextGridFit {
    None,
    Pixel,
    SubPixel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DefineBitsLossless {
    pub version: u8,
    pub id: CharacterId,
    pub format: BitmapFormat,
    pub width: u16,
    pub height: u16,
    pub num_colors: u8,
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitmapFormat {
    ColorMap8,
    Rgb15,
    Rgb32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DefineVideoStream {
    pub id: CharacterId,
    pub num_frames: u16,
    pub width: u16,
    pub height: u16,
    pub is_smoothed: bool,
    pub deblocking: VideoDeblocking,
    pub codec: VideoCodec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoDeblocking {
    UseVideoPacketValue,
    None,
    Level1,
    Level2,
    Level3,
    Level4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoCodec {
    H263,
    ScreenVideo,
    VP6,
    VP6WithAlpha,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VideoFrame {
    pub stream_id: CharacterId,
    pub frame_num: u16,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DefineBitsJpeg3 {
    pub id: CharacterId,
    pub version: u8,
    pub deblocking: f32,
    pub data: Vec<u8>,
    pub alpha_data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DoAbc {
    pub name: String,
    pub is_lazy_initialize: bool,
    pub data: Vec<u8>,
}

pub type DoAction = Vec<u8>;

pub type JpegTables = Vec<u8>;

/// `ProductInfo` contains information about the software used to generate the SWF.
/// Not documented in the SWF19 reference. Emitted by mxmlc.
/// See http://wahlers.com.br/claus/blog/undocumented-swf-tags-written-by-mxmlc/
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductInfo {
    pub product_id: u32,
    pub edition: u32,
    pub major_version: u8,
    pub minor_version: u8,
    pub build_number: u64,
    pub compilation_date: u64,
}

/// `DebugId` is a UUID written to debug SWFs and used by the Flash Debugger.
pub type DebugId = [u8; 16];
