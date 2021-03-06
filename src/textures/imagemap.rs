//! Image Texture

#![allow(dead_code)]
use super::get_texture_mapping;
use crate::core::geometry::*;
use crate::core::mipmap::*;
use crate::core::paramset::*;
use crate::core::pbrt::*;
use crate::core::spectrum::*;
use crate::core::texture::*;
use crate::textures::*;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign};

/// Stores an image texture with MIPMaps using texels of type `Tmemory`.
#[derive(Clone)]
pub struct ImageTexture<Tmemory>
where
    Tmemory: Copy
        + Default
        + Mul<Float, Output = Tmemory>
        + MulAssign<Float>
        + Div<Float, Output = Tmemory>
        + DivAssign<Float>
        + Add<Tmemory, Output = Tmemory>
        + AddAssign
        + Clamp<Float>,
    Spectrum: ConvertIn<Tmemory>,
{
    /// 2D mapping.
    mapping: ArcTextureMapping2D,

    /// The mipmaps.
    mipmap: ArcMIPMap<Tmemory>,
}

macro_rules! new_image_texture {
    ($t: ty) => {
        impl ImageTexture<$t> {
            /// Create a new `ImageTexture<$ty>`.
            ///
            /// * `mapping`          - The 2D mapping.
            /// * `path`             - The path to the image file.
            /// * `filtering_method` - Type of filtering to use for mipmaps.
            /// * `wrap_mode`        - Image wrapping convention.
            /// * `scale`            - Scale for the texel values.
            /// * `gamma`            - Do gamma correction for the texel values.
            /// * `max_anisotropy`   - Used to clamp the ellipse eccentricity (EWA).
            ///                        Set to 0 if EWA is not being used.
            pub fn new(
                mapping: ArcTextureMapping2D,
                path: &str,
                filtering_method: FilteringMethod,
                wrap_mode: ImageWrap,
                scale: Float,
                gamma: bool,
                max_anisotropy: Float,
            ) -> Self {
                let tex_info = TexInfo::new(
                    path,
                    filtering_method,
                    wrap_mode,
                    scale,
                    gamma,
                    max_anisotropy,
                );
                let mipmap = match MIPMapCache::get(tex_info) {
                    Ok(mipmap) => mipmap,
                    Err(err) => panic!("Unable to load MIPMap: {}", err),
                };
                Self { mapping, mipmap }
            }
        }
    };
}
new_image_texture!(RGBSpectrum);
new_image_texture!(Float);

// Implement `Texture<Tresult>` for `ImageTexture<Tmemory>` where `Tresult` is
// the output for texture evaluation to fit with the conventions of PBRT v3.

/// Implement `ImageTexture` stored in MIPMaps as `RGBSpectrum` and evaluate to
/// `Spectrum`.
impl Texture<Spectrum> for ImageTexture<RGBSpectrum> {
    /// Evaluate the texture at surface interaction.
    ///
    /// * `si` - Surface interaction.
    fn evaluate(&self, si: &SurfaceInteraction) -> Spectrum {
        // Get the (s, t) mapping for the intersection.
        let TextureMap2DResult {
            p: st,
            dstdx,
            dstdy,
        } = self.mapping.map(si);

        let mem = self.mipmap.lookup(&st, &dstdx, &dstdy);

        // Convert out to `Spectrum`.
        let rgb = mem.to_rgb();
        Spectrum::from_rgb(&rgb, None)
    }
}

/// Implement `ImageTexture` stored in MIPMaps as `Float` and evaluate to
/// `Float`.
impl Texture<Float> for ImageTexture<Float> {
    /// Evaluate the texture at surface interaction.
    ///
    /// * `si` - Surface interaction.
    fn evaluate(&self, si: &SurfaceInteraction) -> Float {
        // Get the (s, t) mapping for the intersection.
        let TextureMap2DResult {
            p: st,
            dstdx,
            dstdy,
        } = self.mapping.map(si);

        // Convert out to `Float`.
        self.mipmap.lookup(&st, &dstdx, &dstdy)
    }
}

macro_rules! from_params {
    ($t: ty) => {
        impl From<(&TextureParams, &Transform)> for ImageTexture<$t> {
            /// Create a `ImageTexture<$t>` from given parameter set and
            /// transformation from texture space to world space.
            ///
            /// * `p` - Tuple containing texture parameters and texture space
            ///         to world space transform.
            fn from(p: (&TextureParams, &Transform)) -> Self {
                let (tp, tex2world) = p;

                // Initialize 2D texture mapping `map` from `tp`.
                let map = get_texture_mapping(tp, tex2world);

                // Initialize `ImageTexture` parameters.
                let max_anisotropy = tp.find_float("maxanisotropy", 8.0);
                let filtering_method = if tp.find_bool("trilinear", false) {
                    FilteringMethod::Trilinear
                } else {
                    FilteringMethod::Ewa
                };
                let wrap = tp.find_string("wrap", String::from("repeat"));
                let wrap_mode = match &wrap[..] {
                    "black" => ImageWrap::Black,
                    "clamp" => ImageWrap::Clamp,
                    _ => ImageWrap::Repeat,
                };
                let scale = tp.find_float("scale", 1.0);
                let path = tp.find_filename("filename", String::from(""));
                let gamma = tp.find_bool("gamma", path.ends_with(".tga") || path.ends_with(".png"));
                Self::new(
                    map,
                    &path,
                    filtering_method,
                    wrap_mode,
                    scale,
                    gamma,
                    max_anisotropy,
                )
            }
        }
    };
}
from_params!(RGBSpectrum);
from_params!(Float);
