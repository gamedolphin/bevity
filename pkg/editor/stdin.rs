use std::sync::{
    mpsc::{self, Receiver, TryRecvError},
    Arc,
};

use bevity_scene::{MonoBehaviour, ObjectLocalFileId, UnityChangeObject, UnityTransformDirty};
use bevy::{prelude::*, utils::HashMap};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ChangeObject {
    pub object_id: i64,
    pub serialized: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IdResponse {
    pub object_id: i64,
    pub actual_id: i64,
}

#[derive(Resource)]
pub struct UnityStdin {
    pub receiver: Arc<Mutex<Receiver<String>>>,
}

#[derive(Resource, Default)]
pub struct UnityEntityMap {
    pub object_map: HashMap<i64, Entity>,
}

pub(crate) fn setup_stdin<
    T: serde::de::DeserializeOwned + Send + Sync + 'static + MonoBehaviour,
>(
    app: &mut App,
) {
    let receiver = spawn_stdin_channel();

    app.insert_resource(UnityStdin {
        receiver: Arc::new(Mutex::new(receiver)),
    })
    .insert_resource(UnityEntityMap::default())
    .add_systems(PreUpdate, send_unknown_gameobjects)
    .add_systems(PreUpdate, listen_stdin::<T>);
}

#[derive(Component)]
pub(crate) struct UnityTrackingRequested;

#[derive(Component)]
pub(crate) struct UnityTracked {
    object_id: i64,
}

#[allow(clippy::type_complexity)]
fn send_unknown_gameobjects(
    query: Query<
        (Entity, &ObjectLocalFileId),
        (Without<UnityTrackingRequested>, Without<UnityTracked>),
    >,
    mut commands: Commands,
) {
    for (entity, _) in &query {
        commands.entity(entity).insert(UnityTrackingRequested);
    }
}

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    std::thread::spawn(move || loop {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer).unwrap();
    });
    rx
}

fn listen_stdin<T: serde::de::DeserializeOwned + MonoBehaviour>(world: &mut World) {
    world.resource_scope(|world2, unity_stdin: Mut<UnityStdin>| {
        world2.resource_scope(|world3, mut unity_map: Mut<UnityEntityMap>| {
            let receiver = unity_stdin.receiver.lock();
            match receiver.try_recv() {
                Ok(key) => handle_stdin::<T>(key, &mut unity_map, world3),
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {}
            }
        });
    });
}

fn handle_stdin<T: serde::de::DeserializeOwned + MonoBehaviour>(
    instruction: String,
    unity_map: &mut UnityEntityMap,
    world: &mut World,
) {
    let splits: Vec<&str> = instruction.split('|').collect();
    let Some(first) = splits.first() else {
        // not an editor change, random text, throw this
        return;
    };

    if !first.contains("EDITOR_CHANGE") {
        tracing::warn!("does not containe EDITOR_CHANGE, received: {}", instruction);
        return;
    }

    let Some(kind) = splits.get(1) else {
        tracing::warn!("missing message type, received: {}", instruction);
        return;
    };

    let Ok(kind) = kind.parse::<i32>() else {
        tracing::warn!("received unsupported message kind: {}", kind);
        return;
    };

    let Some(instruction) = splits.get(2) else {
        tracing::warn!("missing actual instruction, received: {}", instruction);
        return;
    };

    match kind {
        0 => handle_incoming_update::<T>(instruction, unity_map, world),
        1 => {}
        2 => handle_incoming_ids(instruction, unity_map),
        _ => {
            tracing::warn!("unsupported kind value: {}", kind);
        }
    }
}

fn handle_incoming_ids(instruction: &str, unity_map: &mut UnityEntityMap) {
    let ids = match serde_json::from_str::<Vec<IdResponse>>(instruction) {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!("failed to parse id response: {}", e);
            return;
        }
    };
}

fn handle_incoming_update<T: serde::de::DeserializeOwned + MonoBehaviour>(
    instruction: &str,
    unity_map: &UnityEntityMap,
    world: &mut World,
) {
    let instructions = match serde_json::from_str::<Vec<ChangeObject>>(instruction) {
        Ok(instructions) => instructions,
        Err(e) => {
            tracing::error!("failed to parse change list: {}", e);
            return;
        }
    };

    instructions.iter().for_each(|f| {
        let Some(entity) = unity_map.object_map.get(&f.object_id) else {
            tracing::error!("got an unknown object_id: {}", f.object_id);
            return;
        };

        let mut e = world.entity_mut(*entity);
        // println!("received instruction: {} for {:?}", f.serialized, entity);
        let Ok(obj) = serde_json::from_str::<UnityChangeObject<T>>(&f.serialized) else {
            tracing::error!("failed to parse change object: {}", f.serialized);
            return;
        };
        match obj {
            UnityChangeObject::Transform(t) => {
                e.insert(TransformBundle {
                    local: (&t).into(),
                    ..default()
                });
                e.insert(UnityTransformDirty);
            }
            UnityChangeObject::MonoBehaviour(v) => v.update_component(&mut e),
            _ => {}
        };
    });
}
