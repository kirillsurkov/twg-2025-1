use std::marker::PhantomData;

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

#[derive(Default)]
pub struct UpdateMaterialTexturesPlugin<Material> {
    _pd: PhantomData<Material>,
}

pub trait MaterialTextures {
    fn textures(&self) -> Vec<&Option<Handle<Image>>>;
}

impl<Material: Asset + MaterialTextures> Plugin for UpdateMaterialTexturesPlugin<Material> {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update::<Material>);
    }
}

impl MaterialTextures for StandardMaterial {
    fn textures(&self) -> Vec<&Option<Handle<Image>>> {
        vec![
            &self.base_color_texture,
            &self.metallic_roughness_texture,
            &self.emissive_texture,
            &self.depth_map,
            &self.normal_map_texture,
            &self.occlusion_texture,
        ]
    }
}

fn update<Material: Asset + MaterialTextures>(
    mut image_to_materials: Local<HashMap<AssetId<Image>, HashSet<AssetId<Material>>>>,
    mut material_to_images: Local<HashMap<AssetId<Material>, HashSet<AssetId<Image>>>>,
    mut material_events: EventReader<AssetEvent<Material>>,
    mut image_events: EventReader<AssetEvent<Image>>,
    mut materials: ResMut<Assets<Material>>,
) {
    for event in material_events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                let Some(material) = materials.get(*id) else {
                    continue;
                };
                let material_images = material_to_images.entry(*id).or_default();
                for image in material
                    .textures()
                    .into_iter()
                    .filter_map(|img| img.as_ref())
                {
                    material_images.insert(image.id());
                }
                for image in material_images.iter() {
                    image_to_materials
                        .entry(*image)
                        .or_default()
                        .insert(id.clone());
                }
            }
            AssetEvent::Removed { id } => {
                for image in material_to_images.remove(id).unwrap_or_default() {
                    image_to_materials.get_mut(&image).unwrap().remove(id);
                }
            }
            _ => {}
        }
    }

    for event in image_events.read() {
        if let AssetEvent::Modified { id } = event {
            if let Some(image_materials) = image_to_materials.get(id) {
                for material in image_materials {
                    materials.get_mut(*material);
                }
            }
        }
    }
}
