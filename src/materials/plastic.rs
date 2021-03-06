//! Plastic Material

#![allow(dead_code)]
use crate::core::geometry::*;
use crate::core::material::*;
use crate::core::microfacet::*;
use crate::core::paramset::*;
use crate::core::pbrt::*;
use crate::core::reflection::*;
use crate::core::spectrum::*;
use crate::core::texture::*;
use crate::textures::*;
use std::sync::Arc;

/// Implements plastic material.
pub struct PlasticMaterial {
    /// Spectral diffuse reflection.
    kd: ArcTexture<Spectrum>,

    /// Spectral specular reflection.
    ks: ArcTexture<Spectrum>,

    /// Roughness.
    roughness: ArcTexture<Float>,

    /// Bump map.
    bump_map: Option<ArcTexture<Float>>,

    /// Remap roughness value to [0, 1] where higher values represent larger
    /// highlights. If this is `false`, use the microfacet distributions `alpha`
    /// parameter.
    remap_roughness: bool,
}

impl PlasticMaterial {
    /// Create a new `PlasticMaterial`.
    ///
    ///
    /// * `kd`              - Spectral diffuse reflection.
    /// * `ks`              - Spectral specular reflection.
    /// * `roughness`       - Roughness.
    /// * `remap_roughness` - Remap roughness value to [0, 1] where higher values
    ///                       represent larger highlights. If this is `false`,
    ///                       use the microfacet distributions `alpha` parameter.
    /// * `bump_map`        - Optional bump map.
    pub fn new(
        kd: ArcTexture<Spectrum>,
        ks: ArcTexture<Spectrum>,
        roughness: ArcTexture<Float>,
        remap_roughness: bool,
        bump_map: Option<ArcTexture<Float>>,
    ) -> Self {
        Self {
            kd: kd.clone(),
            ks: ks.clone(),
            roughness: roughness.clone(),
            remap_roughness,
            bump_map: bump_map.clone(),
        }
    }
}

impl Material for PlasticMaterial {
    /// Initializes representations of the light-scattering properties of the
    /// material at the intersection point on the surface.
    ///
    /// * `si`                   - The surface interaction at the intersection.
    /// * `mode`                 - Transport mode (ignored).
    /// * `allow_multiple_lobes` - Indicates whether the material should use
    ///                            BxDFs that aggregate multiple types of
    ///                            scattering into a single BxDF when such BxDFs
    ///                            are available (ignored).
    fn compute_scattering_functions(
        &self,
        si: &mut SurfaceInteraction,
        _mode: TransportMode,
        _allow_multiple_lobes: bool,
    ) {
        // Perform bump mapping with `bump_map`, if present.
        if let Some(bump_map) = self.bump_map.clone() {
            Material::bump(self, bump_map, si);
        }

        let mut bsdf = BSDF::new(&si.clone(), None);

        // Initialize diffuse component of plastic material.
        let kd = self.kd.evaluate(si).clamp_default();
        if !kd.is_black() {
            bsdf.add(Arc::new(LambertianReflection::new(kd)));
        }

        // Initialize specular component of plastic material.
        let ks = self.ks.evaluate(si).clamp_default();
        if !ks.is_black() {
            let fresnel = Arc::new(FresnelDielectric::new(1.5, 1.0));

            // Create microfacet distribution for plastic material.
            let mut rough = self.roughness.evaluate(si);
            if self.remap_roughness {
                rough = TrowbridgeReitzDistribution::roughness_to_alpha(rough);
            }
            let distrib = Arc::new(TrowbridgeReitzDistribution::new(rough, rough, true));
            let spec = MicrofacetReflection::new(ks, distrib, fresnel);
            bsdf.add(Arc::new(spec));
        }

        si.bsdf = Some(Arc::new(bsdf));
    }
}

impl From<&TextureParams> for PlasticMaterial {
    /// Create a plastic material from given parameter set.
    ///
    /// * `tp` - Texture parameter set.
    fn from(tp: &TextureParams) -> Self {
        let kd = tp.get_spectrum_texture_or_else(
            "Kd",
            Arc::new(ConstantTexture::new(Spectrum::new(0.25))),
        );
        let ks = tp.get_spectrum_texture_or_else(
            "Ks",
            Arc::new(ConstantTexture::new(Spectrum::new(0.25))),
        );
        let roughness =
            tp.get_float_texture_or_else("roughness", Arc::new(ConstantTexture::new(0.1)));
        let bump_map = tp.get_float_texture("bumpmap");
        let remap_roughness = tp.find_bool("remaproughness", true);
        Self::new(kd, ks, roughness, remap_roughness, bump_map)
    }
}
