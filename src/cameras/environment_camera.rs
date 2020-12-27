//! Environment Camera

#![allow(dead_code)]
use crate::core::camera::*;
use crate::core::film::*;
use crate::core::geometry::*;
use crate::core::medium::*;
use crate::core::pbrt::*;
use std::sync::Arc;

// Environment camera.
#[derive(Clone)]
pub struct EnvironmentCamera {
    /// Common camera parameters.
    pub data: CameraData,
}

impl EnvironmentCamera {
    /// Create a new environment camera.
    ///
    /// * `camera_to_world` - Animated transformation describing the camera's
    ///                       motion in the scene.
    /// * `shutter_open`    - Time when shutter is open.
    /// * `shutter_close`   - Time when shutter is closed.
    /// * `film`            - The film to capture the rendered image.
    /// * `medium`          - Scattering medium the camera lies in.
    pub fn new(
        camera_to_world: AnimatedTransform,
        shutter_open: Float,
        shutter_close: Float,
        film: Arc<Film>,
        medium: ArcMedium,
    ) -> Self {
        Self {
            data: CameraData::new(
                camera_to_world,
                shutter_open,
                shutter_close,
                film.clone(),
                medium.clone(),
            ),
        }
    }
}

impl Camera for EnvironmentCamera {
    /// Returns a ray corresponding to a given sample. It also returns, a floating
    /// point value that affects how much the radiance arriving at the film plane
    /// will contribute to final image.
    ///
    /// * `sample` - The sample.
    fn generate_ray(&self, sample: &CameraSample) -> (Ray, Float) {
        // Compute environment camera ray direction.
        let theta = PI * sample.p_film.y / self.data.film.full_resolution.y as Float;
        let phi = TWO_PI * sample.p_film.x / self.data.film.full_resolution.x as Float;
        let dir = Vector3f::new(sin(theta) * cos(phi), cos(theta), sin(theta) * sin(phi));

        let ray = Ray::new(
            Point3f::new(0.0, 0.0, 0.0),
            dir,
            INFINITY,
            lerp(sample.time, self.data.shutter_open, self.data.shutter_close),
            Some(self.data.medium.clone()),
        );

        (self.data.camera_to_world.transform_ray(&ray), 1.0)
    }

    /// Return the spatial and directional PDFs, as a tuple, for sampling a
    /// particular ray leaving the camera.
    ///
    /// * `ray` - The ray.
    fn pdf_we(&self, _ray: &Ray) -> PDFResult {
        panic!("NOT IMPLEMENTED");
    }
}
