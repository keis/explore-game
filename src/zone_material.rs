use crate::fog::Fog;
use bevy::{
    pbr::RenderMaterials,
    prelude::*,
    reflect::TypeUuid,
    render::{render_resource::*, renderer::RenderQueue, Extract, RenderApp, RenderStage},
};

#[derive(Default)]
pub struct ZoneMaterialPlugin;

impl Plugin for ZoneMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<ZoneMaterial>::default());

        app.sub_app_mut(RenderApp)
            .add_system_to_stage(RenderStage::Extract, extract_zone)
            .add_system_to_stage(RenderStage::Prepare, prepare_zone_material);
    }
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "05f50382-7218-4860-8c4c-06dbd66694db"]
pub struct ZoneMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub terrain_texture: Option<Handle<Image>>,
    #[texture(2)]
    #[sampler(3)]
    pub cloud_texture: Option<Handle<Image>>,
    #[uniform(4)]
    pub visible: u32,
    #[uniform(4)]
    pub explored: u32,
}

impl Material for ZoneMaterial {
    fn fragment_shader() -> ShaderRef {
        "zone_material.wgsl".into()
    }
}

#[derive(Clone, ShaderType)]
struct ZoneMaterialUniformData {
    visible: u32,
    explored: u32,
}

fn extract_zone(
    mut commands: Commands,
    zone_query: Extract<Query<(Entity, &Fog, &Handle<ZoneMaterial>)>>,
) {
    for (entity, fog, handle) in zone_query.iter() {
        commands
            .get_or_spawn(entity)
            .insert(*fog)
            .insert(handle.clone());
    }
}

fn prepare_zone_material(
    materials: Res<RenderMaterials<ZoneMaterial>>,
    zone_query: Query<(&Fog, &Handle<ZoneMaterial>)>,
    render_queue: Res<RenderQueue>,
) {
    for (fog, handle) in &zone_query {
        if let Some(material) = materials.get(handle) {
            for binding in material.bindings.iter() {
                if let OwnedBindingResource::Buffer(cur_buffer) = binding {
                    let mut buffer = encase::UniformBuffer::new(Vec::new());
                    buffer
                        .write(&ZoneMaterialUniformData {
                            visible: fog.visible as u32,
                            explored: fog.explored as u32,
                        })
                        .unwrap();
                    render_queue.write_buffer(cur_buffer, 0, buffer.as_ref());
                }
            }
        }
    }
}
