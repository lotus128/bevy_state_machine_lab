use bevy::app::App;
use bevy::app::PluginGroup;
use bevy::app::Startup;
use bevy::app::Update;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::Commands;
use bevy::ecs::system::In;
use bevy::log::LogPlugin;
use bevy::DefaultPlugins;
use bevy_state_machine_lab::ActiveStateComponent;
use bevy_state_machine_lab::AfterTransitionResource;
use bevy_state_machine_lab::BeforeTransitionResource;
use bevy_state_machine_lab::TransitionEventSystemIn;
use std::any::TypeId;

#[derive(Component, Default)]
pub struct PlayerMachineTypeComponent;

#[derive(Component, Default)]
pub struct PlayerInitialStateComponent;

#[derive(Component, Default)]
pub struct PlayerIdleStateComponent;

#[derive(Component, Default)]
pub struct PlayerMoveStateComponent;

pub fn always_entity_transition_condition_system(In(_machine_entity): In<Entity>) -> bool {
    return true;
}

pub fn log_after_entity_transition_event_system(
    In(event_system_in): In<TransitionEventSystemIn<TypeId>>,
) {
    bevy::log::debug!("after_entity_transition_event_system");
    bevy::log::debug!("    event_system_in={:?}", event_system_in);
}

pub fn log_before_entity_transition_event_system(
    In(event_system_in): In<TransitionEventSystemIn<TypeId>>,
) {
    bevy::log::debug!("before_entity_transition_event_system");
    bevy::log::debug!("    event_system_in={:?}", event_system_in);
}

pub fn remove_state_after_transition_event_system<TStateComponent: Component>(
    In(event_system_in): In<TransitionEventSystemIn<TypeId>>,
    mut commands: Commands,
) {
    let mut ec = commands.get_entity(event_system_in.machine_entity).unwrap();
    ec.remove::<TStateComponent>();
}

pub fn insert_state_after_transition_event_system<TStateComponent: Component + Default>(
    In(event_system_in): In<TransitionEventSystemIn>,
    mut commands: Commands,
) {
    let mut ec = commands.get_entity(event_system_in.machine_entity).unwrap();
    ec.insert(TStateComponent::default());
}

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        PlayerMachineTypeComponent,
        PlayerInitialStateComponent,
        ActiveStateComponent::<PlayerMachineTypeComponent>::from_type_id(TypeId::of::<
            PlayerInitialStateComponent,
        >()),
    ));
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(LogPlugin {
        level: bevy::log::Level::DEBUG,
        ..bevy::utils::default()
    }));

    let always_transition_condition_system_id =
        app.register_system(always_entity_transition_condition_system);

    let log_before_transition_event_system_id =
        app.register_system(log_before_entity_transition_event_system);

    let log_after_transition_event_system_id =
        app.register_system(log_after_entity_transition_event_system);

    let remove_initial_state_after_transition_event_system_id = app
        .register_system(remove_state_after_transition_event_system::<PlayerInitialStateComponent>);

    let insert_idle_state_after_transition_event_system_id =
        app.register_system(insert_state_after_transition_event_system::<PlayerIdleStateComponent>);

    app.insert_resource(BeforeTransitionResource::<PlayerMachineTypeComponent> {
        _marker: std::marker::PhantomData::<PlayerMachineTypeComponent>,
        systems: vec![log_before_transition_event_system_id],
    });

    app.insert_resource(AfterTransitionResource::<PlayerMachineTypeComponent> {
        _marker: std::marker::PhantomData::<PlayerMachineTypeComponent>,
        systems: vec![
            log_after_transition_event_system_id,
            remove_initial_state_after_transition_event_system_id,
            insert_idle_state_after_transition_event_system_id,
        ],
    });

    app.add_systems(Startup, spawn);

    app.add_systems(
        Update,
        bevy_state_machine_lab::create_transition_system::<
            PlayerMachineTypeComponent,
            TypeId,
            With<PlayerInitialStateComponent>,
        >(
            always_transition_condition_system_id,
            TypeId::of::<PlayerInitialStateComponent>(),
        ),
    );

    app.run();
}
