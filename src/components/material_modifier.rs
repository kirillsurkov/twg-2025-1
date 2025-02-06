use std::marker::PhantomData;

use bevy::prelude::*;

pub struct MaterialModifierPlugin<MFrom: Material, MTo: Material> {
    _pd: PhantomData<(MFrom, MTo)>,
}

impl<MFrom: Material, MTo: Material> Default for MaterialModifierPlugin<MFrom, MTo> {
    fn default() -> Self {
        Self {
            _pd: PhantomData::default(),
        }
    }
}

impl<MFrom: Material, MTo: Material> Plugin for MaterialModifierPlugin<MFrom, MTo> {
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
pub struct MaterialModifier<MFrom: Material>(Box<dyn Fn(&mut EntityCommands, MFrom) + Send + Sync>);

impl<MFrom: Material> MaterialModifier<MFrom> {
    pub fn new<MTo: Material>(modifier: impl Fn(MFrom) -> MTo + Send + Sync + 'static) -> Self {
        Self(Box::new(move |commands, original_material| {
            commands.insert(NewMaterial(modifier(original_material)));
        }))
    }

    pub fn insert_new_material(&self, entity: &mut EntityCommands, material: MFrom) {
        (*self.0)(entity, material);
    }
}

#[derive(Component, Reflect)]
pub struct OriginalMaterial<MFrom: Material>(pub Handle<MFrom>);

#[derive(Component, Reflect)]
struct NewMaterial<MTo: Material>(MTo);

fn prepare_materials<MFrom: Material, MTo: Material>(
    mut commands: Commands,
    original_materials: Res<Assets<MFrom>>,
    original_material_handles: Query<&MeshMaterial3d<MFrom>, Without<OriginalMaterial<MFrom>>>,
    modifiers: Query<(Entity, &MaterialModifier<MFrom>)>,
    children: Query<&Children>,
) {
    for (entity, modifier) in modifiers.iter() {
        for child in Iterator::chain(children.iter_descendants(entity), [entity]) {
            if let Ok(MeshMaterial3d(handle)) = original_material_handles.get(child) {
                let mut entity = commands.entity(child);
                entity
                    .insert(OriginalMaterial(handle.clone()))
                    .remove::<MeshMaterial3d<MFrom>>();
                modifier.insert_new_material(
                    &mut entity,
                    original_materials.get(handle).unwrap().clone(),
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
    mut removed: RemovedComponents<MaterialModifier<MFrom>>,
    original_materials: Query<&OriginalMaterial<MFrom>>,
    children: Query<&Children>,
) {
    for entity in removed.read() {
        if commands.get_entity(entity).is_none() {
            continue;
        }
        for child in Iterator::chain(children.iter_descendants(entity), [entity]) {
            if let Ok(original_material) = original_materials.get(child) {
                commands
                    .entity(child)
                    .remove::<MeshMaterial3d<MTo>>()
                    .insert(MeshMaterial3d(original_material.0.clone()))
                    .remove::<OriginalMaterial<MFrom>>();
            }
        }
    }
}
