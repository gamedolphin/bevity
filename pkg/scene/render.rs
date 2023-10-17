use bevity_primitives::{FileReference, UnityColor};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UnityRenderSettings {
    #[serde(alias = "m_Fog")]
    pub fog: i32,
    #[serde(alias = "m_FogColor")]
    pub fog_color: UnityColor,

    #[serde(alias = "m_AmbientSkyColor")]
    pub ambient_sky_color: UnityColor,
    #[serde(alias = "m_AmbientIntensity")]
    pub ambient_intensity: f32,

    #[serde(alias = "m_SkyboxMaterial")]
    pub skybox_material: FileReference,

    #[serde(alias = "m_IndirectSpecularColor")]
    pub indirect_specular_color: UnityColor,
}
