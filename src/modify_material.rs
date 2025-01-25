use std::marker::PhantomData;

use bevy::prelude::*;

#[derive(Default)]
pub struct ModifyMaterialPlugin<M> {
    _pd: PhantomData<M>,
}

impl<M: Material> Plugin for ModifyMaterialPlugin<M> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Last,
            (
                (prepare_materials::<M>, apply_new_materials::<M>).chain(),
                restore_original_materials::<M>,
            ),
        )
        .register_type::<OriginalMaterial>();
    }
}

#[derive(Component)]
pub struct ModifyMaterial {
    insert_new_material: Box<dyn Fn(&mut EntityCommands, StandardMaterial) + Send + Sync>,
}

impl ModifyMaterial {
    pub fn new<M: Material>(
        modifier: impl Fn(StandardMaterial) -> M + Send + Sync + 'static,
    ) -> Self {
        Self {
            insert_new_material: Box::new(move |commands, original_material| {
                commands.insert(NewMaterial(modifier(original_material)));
            }),
        }
    }
}

#[derive(Component, Reflect)]
struct OriginalMaterial(MeshMaterial3d<StandardMaterial>);

#[derive(Component, Reflect)]
struct NewMaterial<M>(M);

fn prepare_materials<M: Material + Sized>(
    mut commands: Commands,
    orig_materials: Res<Assets<StandardMaterial>>,
    material_handles: Query<&MeshMaterial3d<StandardMaterial>, Without<OriginalMaterial>>,
    modifies: Query<(Entity, &ModifyMaterial)>,
    children: Query<&Children>,
) {
    for (entity, modify) in modifies.iter() {
        for entity in Iterator::chain(children.iter_descendants(entity), [entity]) {
            if let Ok(original_material_handle) = material_handles.get(entity) {
                (*modify.insert_new_material)(
                    commands
                        .entity(entity)
                        .insert(OriginalMaterial(original_material_handle.clone()))
                        .remove::<MeshMaterial3d<StandardMaterial>>(),
                    orig_materials
                        .get(original_material_handle)
                        .unwrap()
                        .clone(),
                );
            }
        }
    }
}

fn apply_new_materials<M: Material>(
    mut commands: Commands,
    mut new_materials: ResMut<Assets<M>>,
    to_apply: Query<
        (Entity, &NewMaterial<M>),
        (With<OriginalMaterial>, Without<MeshMaterial3d<M>>),
    >,
) {
    for (entity, NewMaterial(new_material)) in to_apply.iter() {
        commands
            .entity(entity)
            .insert(MeshMaterial3d(new_materials.add(new_material.clone())))
            .remove::<NewMaterial<M>>();
    }
}

fn restore_original_materials<M: Material>(
    mut commands: Commands,
    mut removed: RemovedComponents<ModifyMaterial>,
    children: Query<&Children>,
    restores: Query<&OriginalMaterial>,
) {
    for entity in removed.read() {
        for entity in Iterator::chain(children.iter_descendants(entity), [entity]) {
            if let Ok(original_material) = restores.get(entity) {
                commands
                    .entity(entity)
                    .remove::<MeshMaterial3d<M>>()
                    .insert(original_material.0.clone())
                    .remove::<OriginalMaterial>();
            }
        }
    }
}
