use bevity_primitives::{UnityTransformDirty, UnityTransformMeta};
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use serde::Serialize;

use crate::stdin::{ChangeObject, UnityChangeObject};

pub(crate) fn setup_stdout<T: serde::Serialize + Send + Sync + 'static>(app: &mut App) {
    app.insert_resource(UnityChangeMap::default())
        .add_systems(
            PreUpdate,
            (send_changes.before(clear_changes), clear_changes),
        )
        .add_systems(PostUpdate, track_changes::<T>);
}

#[derive(Resource, Default)]
pub struct UnityChangeMap {
    pub changes: HashMap<u64, String>,
    pub dirty: HashSet<u64>,
}

fn clear_changes(mut change_map: ResMut<UnityChangeMap>) {
    change_map.changes.clear();
    change_map.dirty.clear();
}

fn track_changes<T: serde::Serialize>(
    query: Query<
        (
            Entity,
            &UnityTransformMeta,
            &Transform,
            Option<&UnityTransformDirty>,
        ),
        Changed<Transform>,
    >,
    mut change_map: ResMut<UnityChangeMap>,
    mut commands: Commands,
) {
    for (entity, meta, transform, dirty) in &query {
        if let Ok(serialized) =
            serde_json::to_string(&UnityChangeObject::<T>::Transform(transform.into()))
        {
            change_map.changes.insert(meta.object_id, serialized);
            if dirty.is_some() {
                change_map.dirty.insert(meta.object_id);
                commands.entity(entity).remove::<UnityTransformDirty>();
            }
        };
    }
}

#[derive(Serialize)]
struct ChangeAck {
    pub object_id: u64,
}

fn send_changes(change_map: ResMut<UnityChangeMap>) {
    let changes = change_map
        .changes
        .iter()
        .map(|(k, v)| ChangeObject {
            object_id: *k,
            serialized: v.to_string(),
        })
        .collect::<Vec<ChangeObject>>();

    let dirty = change_map
        .dirty
        .iter()
        .map(|object_id| ChangeAck {
            object_id: *object_id,
        })
        .collect::<Vec<ChangeAck>>();

    if !dirty.is_empty() {
        let Ok(serialized) = serde_json::to_string(&dirty) else {
            tracing::error!("failed to serialize ack array");

            return;
        };

        println!("EDITOR_CHANGE|1|{}", serialized);
    }

    if !changes.is_empty() {
        let Ok(serialized) = serde_json::to_string(&changes) else {
            tracing::error!("failed to serialize changes array");

            return;
        };

        println!("EDITOR_CHANGE|0|{}", serialized);
    }
}
