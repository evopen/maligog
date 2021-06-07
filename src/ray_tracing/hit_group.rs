use crate::ShaderStage;
use ash::vk;
#[derive(Clone)]
pub struct TrianglesHitGroup {
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
#[derive(Clone)]
pub struct ProceduralHitGroup {}

pub trait HitGroup: dyn_clone::DynClone + 'static {
    fn shader_stage_create_infos(&self) -> Vec<vk::PipelineShaderStageCreateInfo>;
    fn shader_group_type(&self) -> vk::RayTracingShaderGroupTypeKHR;
    fn has_closest_hit_shader(&self) -> bool;
    fn has_any_hit_shader(&self) -> bool;
    fn has_intersection_hit_shader(&self) -> bool;
}

impl HitGroup for TrianglesHitGroup {
    fn shader_stage_create_infos(&self) -> Vec<vk::PipelineShaderStageCreateInfo> {
        let mut infos = Vec::new();
        infos.push(self.closest_hit_shader.shader_stage_create_info());
        if let Some(any_hit_shader) = self.any_hit_shader.as_ref() {
            infos.push(any_hit_shader.shader_stage_create_info());
        }
        infos
    }

    fn shader_group_type(&self) -> vk::RayTracingShaderGroupTypeKHR {
        vk::RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP
    }

    fn has_closest_hit_shader(&self) -> bool {
        true
    }

    fn has_any_hit_shader(&self) -> bool {
        self.any_hit_shader.is_some()
    }

    fn has_intersection_hit_shader(&self) -> bool {
        false
    }
}
impl HitGroup for ProceduralHitGroup {
    fn shader_stage_create_infos(&self) -> Vec<vk::PipelineShaderStageCreateInfo> {
        todo!()
    }

    fn shader_group_type(&self) -> vk::RayTracingShaderGroupTypeKHR {
        vk::RayTracingShaderGroupTypeKHR::PROCEDURAL_HIT_GROUP
    }

    fn has_closest_hit_shader(&self) -> bool {
        todo!()
    }

    fn has_any_hit_shader(&self) -> bool {
        todo!()
    }

    fn has_intersection_hit_shader(&self) -> bool {
        todo!()
    }
}
