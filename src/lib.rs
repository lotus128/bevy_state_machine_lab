use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryFilter;
use bevy::ecs::system::In;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::system::Resource;
use bevy::ecs::system::SystemId;
use bevy::ecs::system::SystemState;
use bevy::ecs::world::World;
use std::any::TypeId;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct TransitionEventSystemIn<TStateId = TypeId> {
    pub machine_entity: Entity,
    pub prev_state_id: TStateId,
    pub next_state_id: TStateId,
}

#[derive(Resource)]
pub struct BeforeTransitionResource<TMachineType: Send, TStateId: 'static = TypeId> {
    pub _marker: PhantomData<TMachineType>,
    pub systems: Vec<SystemId<In<TransitionEventSystemIn<TStateId>>>>,
}

#[derive(Resource)]
pub struct AfterTransitionResource<TMachineType: Send, TStateId: 'static = TypeId> {
    pub _marker: PhantomData<TMachineType>,
    pub systems: Vec<SystemId<In<TransitionEventSystemIn<TStateId>>>>,
}

#[derive(Component)]
pub struct ActiveStateComponent<TMachineTypeMarker: Send + Sync, TStateId: Send = TypeId> {
    _marker: PhantomData<TMachineTypeMarker>,
    pub active_state_type_id: TStateId,
}

impl<TMachineTypeMarker: Send + Sync, TStateId: Send + Sync>
    ActiveStateComponent<TMachineTypeMarker, TStateId>
{
    pub fn from_type_id(
        active_state_type_id: TStateId,
    ) -> ActiveStateComponent<TMachineTypeMarker, TStateId> {
        return ActiveStateComponent::<TMachineTypeMarker, TStateId> {
            _marker: PhantomData::<TMachineTypeMarker>,
            active_state_type_id,
        };
    }
}

pub fn create_transition_system<
    TMachineType: Send + Sync,
    TStateId: Clone + Send + Sync,
    TQueryFilter: QueryFilter,
>(
    condition_system_id: SystemId<In<Entity>, bool>,
    next_state_type_id: TStateId,
) -> impl FnMut(
    &mut World,
    &mut SystemState<(
        Res<BeforeTransitionResource<TMachineType, TStateId>>,
        Res<AfterTransitionResource<TMachineType, TStateId>>,
        Query<(Entity, &mut ActiveStateComponent<TMachineType, TStateId>), TQueryFilter>,
    )>,
) {
    let transition_system = move |world: &mut World,
                                  params: &mut SystemState<(
        Res<BeforeTransitionResource<TMachineType, TStateId>>,
        Res<AfterTransitionResource<TMachineType, TStateId>>,
        Query<(Entity, &mut ActiveStateComponent<TMachineType, TStateId>), TQueryFilter>,
    )>| {
        let entities_to_update: Vec<(Entity, TStateId)> = {
            let mut entities_to_update = Vec::new();

            let (_, _, mut target_q) = params.get_mut(world);

            for (entity, active_state_c) in target_q.iter_mut() {
                let current_state_type_id = active_state_c.active_state_type_id.clone();
                entities_to_update.push((entity, current_state_type_id));
            }

            entities_to_update
        };

        for (machine_entity, active_state_type_id) in entities_to_update {
            let condition_output = world
                .run_system_with_input(condition_system_id, machine_entity)
                .unwrap();

            if !condition_output {
                continue;
            }

            {
                let (before_transition_handler_r, _, _) = params.get_mut(world);
                let before_transition_event_handler_system_ids =
                    before_transition_handler_r.systems.clone();
                for before_transition_handler_system_id in
                    before_transition_event_handler_system_ids.iter()
                {
                    let input = TransitionEventSystemIn {
                        machine_entity: machine_entity,
                        prev_state_id: active_state_type_id.clone(),
                        next_state_id: next_state_type_id.clone(),
                    };

                    world
                        .run_system_with_input(*before_transition_handler_system_id, input)
                        .unwrap();
                }
            }

            let (_, _, mut q) = params.get_mut(world);
            let qi = q.get_mut(machine_entity).unwrap();
            let (_, mut active_state_type_id_c) = qi;

            active_state_type_id_c.active_state_type_id = next_state_type_id.clone();

            {
                let (_, after_transition_handler_r, _) = params.get_mut(world);
                let after_transition_event_handler_system_ids =
                    after_transition_handler_r.systems.clone();
                for after_transition_handler_system_id in
                    after_transition_event_handler_system_ids.iter()
                {
                    let input = TransitionEventSystemIn {
                        machine_entity: machine_entity,
                        prev_state_id: active_state_type_id.clone(),
                        next_state_id: next_state_type_id.clone(),
                    };

                    world
                        .run_system_with_input(*after_transition_handler_system_id, input)
                        .unwrap();
                }
            }
        }
    };

    return transition_system;
}
