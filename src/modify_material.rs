use bevy::prelude::*;

pub struct ModifyMaterialPlugin;

impl Plugin for ModifyMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (modify_materials, restore_materials))
            .register_type::<OriginalMaterial>();
    }
}

#[derive(Component)]
pub struct ModifyMaterial {
    modifier: Box<dyn Fn(StandardMaterial) -> StandardMaterial + Send + Sync>,
}

impl ModifyMaterial {
    pub fn new(
        modifier: impl Fn(StandardMaterial) -> StandardMaterial + Send + Sync + 'static,
    ) -> Self {
        Self {
            modifier: Box::new(modifier),
        }
    }
}

#[derive(Component, Reflect)]
struct OriginalMaterial(MeshMaterial3d<StandardMaterial>);

fn modify_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    modifies: Query<(Entity, &ModifyMaterial)>,
    children: Query<&Children>,
    material_handles: Query<&MeshMaterial3d<StandardMaterial>, Without<OriginalMaterial>>,
) {
    for (entity, modify) in modifies.iter() {
        for entity in children.iter_descendants(entity).chain([entity]) {
            if let Ok(original_material) = material_handles.get(entity) {
                let new_material =
                    (*modify.modifier)(materials.get(original_material).unwrap().clone());
                commands
                    .entity(entity)
                    .insert(OriginalMaterial(original_material.clone()))
                    .remove::<MeshMaterial3d<StandardMaterial>>()
                    .insert(MeshMaterial3d(materials.add(new_material)));
            }
        }
    }
}

fn restore_materials(
    mut commands: Commands,
    mut removed: RemovedComponents<ModifyMaterial>,
    children: Query<&Children>,
    restores: Query<&OriginalMaterial>,
) {
    for entity in removed.read() {
        for entity in children.iter_descendants(entity).chain([entity]) {
            if let Ok(original_material) = restores.get(entity) {
                commands
                    .entity(entity)
                    .remove::<MeshMaterial3d<StandardMaterial>>()
                    .insert(original_material.0.clone())
                    .remove::<OriginalMaterial>();
            }
        }
    }
}
