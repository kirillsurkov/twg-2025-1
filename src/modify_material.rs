use std::marker::PhantomData;

use bevy::prelude::*;

pub struct ModifyMaterialPlugin<MFrom: Material, MTo: Material> {
    _pd: PhantomData<(MFrom, MTo)>,
}

impl<MFrom: Material, MTo: Material> Default for ModifyMaterialPlugin<MFrom, MTo> {
    fn default() -> Self {
        Self {
            _pd: PhantomData::default(),
        }
    }
}

impl<MFrom: Material, MTo: Material> Plugin for ModifyMaterialPlugin<MFrom, MTo> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                restore_original_materials::<MFrom, MTo>,
                prepare_materials::<MFrom, MTo>,
                apply_new_materials::<MFrom, MTo>,
            )
                .chain(),
        )
        .register_type::<OriginalMaterial<MFrom>>();
    }
}

#[derive(Component)]
pub struct ModifyMaterial<MFrom: Material> {
    insert_new_material: Box<dyn Fn(&mut EntityCommands, MFrom) + Send + Sync>,
}

impl<MFrom: Material> ModifyMaterial<MFrom> {
    pub fn new<MTo: Material>(modifier: impl Fn(MFrom) -> MTo + Send + Sync + 'static) -> Self {
        Self {
            insert_new_material: Box::new(move |commands, original_material| {
                commands.insert(NewMaterial(modifier(original_material)));
            }),
        }
    }
}

#[derive(Component, Reflect)]
pub struct OriginalMaterial<MFrom: Material>(pub MeshMaterial3d<MFrom>);

#[derive(Component, Reflect)]
struct NewMaterial<MTo: Material>(MTo);

#[derive(Component)]
struct Modified;

fn prepare_materials<MFrom: Material, MTo: Material>(
    mut commands: Commands,
    orig_materials: Res<Assets<MFrom>>,
    material_handles: Query<&MeshMaterial3d<MFrom>, Without<OriginalMaterial<MFrom>>>,
    modifies: Query<(Entity, &ModifyMaterial<MFrom>), Without<Modified>>,
    children: Query<&Children>,
) {
    for (entity, modify) in modifies.iter() {
        for child in Iterator::chain(children.iter_descendants(entity), [entity]) {
            if let Ok(original_material_handle) = material_handles.get(child) {
                commands.entity(entity).insert(Modified);
                (*modify.insert_new_material)(
                    commands
                        .entity(child)
                        .insert(OriginalMaterial(original_material_handle.clone()))
                        .remove::<MeshMaterial3d<MFrom>>(),
                    orig_materials
                        .get(original_material_handle)
                        .unwrap()
                        .clone(),
                );
            }
        }
    }
}

fn apply_new_materials<MFrom: Material, MTo: Material>(
    mut commands: Commands,
    mut new_materials: ResMut<Assets<MTo>>,
    to_apply: Query<
        (Entity, &NewMaterial<MTo>),
        (With<OriginalMaterial<MFrom>>, Without<MeshMaterial3d<MTo>>),
    >,
) {
    for (entity, NewMaterial(new_material)) in to_apply.iter() {
        commands
            .entity(entity)
            .insert(MeshMaterial3d(new_materials.add(new_material.clone())))
            .remove::<NewMaterial<MTo>>();
    }
}

fn restore_original_materials<MFrom: Material, MTo: Material>(
    mut commands: Commands,
    mut removed: RemovedComponents<ModifyMaterial<MFrom>>,
    children: Query<&Children>,
    restores: Query<&OriginalMaterial<MFrom>>,
) {
    for entity in removed.read() {
        if let Some(mut entity) = commands.get_entity(entity) {
            entity.remove::<Modified>();
        }
        for entity in Iterator::chain(children.iter_descendants(entity), [entity]) {
            let Some(mut entity) = commands.get_entity(entity) else {
                continue;
            };
            if let Ok(original_material) = restores.get(entity.id()) {
                entity
                    .remove::<MeshMaterial3d<MTo>>()
                    .insert(original_material.0.clone())
                    .remove::<OriginalMaterial<MFrom>>();
            }
        }
    }
}
