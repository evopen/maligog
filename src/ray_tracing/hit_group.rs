use crate::ShaderStage;
use ash::vk;

struct TrianglesHitGroup {
    closest_hit_shader: ShaderStage,
    any_hit_shader: Option<ShaderStage>,
}

impl TrianglesHitGroup {
    pub fn new(closest_hit_shader: &ShaderStage, any_hit_shader: Option<&ShaderStage>) -> Self {
        assert!(closest_hit_shader.stage == vk::ShaderStageFlags::CLOSEST_HIT_KHR);
        if let Some(any_hit_shader) = any_hit_shader {
            assert!(any_hit_shader.stage == vk::ShaderStageFlags::ANY_HIT_KHR);
        }
        Self {
            closest_hit_shader: closest_hit_shader.clone(),
            any_hit_shader: any_hit_shader.map(|a| a.to_owned()),
        }
    }
}

struct ProceduralHitGroup {}

pub(crate) trait HitGroup {}

impl HitGroup for TrianglesHitGroup {}
impl HitGroup for ProceduralHitGroup {}
