use bevy::prelude::*;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // .add_plugins(WorldInspectorPlugin::new())  // Zeigt FPS + alle Entities!
        .add_systems(Update, (verlet_integration,
            check_ground_collision, 
            keyboard_input,
            spawn_particle,
        ))
        .run();
}

#[derive(Component)]
struct Particle {
    pos: Vec3,
    old_pos: Vec3,
    acceleration: Vec3,
    radius: f32
}

impl Particle {
    fn new(pos: Vec3, radius: f32) -> Self {
        Self {
            pos: pos,
            old_pos: pos, 
            acceleration: Vec3::ZERO,
            radius: radius,
        }
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, 8.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
    ));

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // Particle Spawn
    
}
fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        spawn_particle(commands, meshes, materials);
    }
}
fn spawn_particle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rand = rand::thread_rng();
    let start_pos = Vec3::new(
        rand.gen_range(-1.0..1.0),  // Float mit .0
        5.0,
        rand.gen_range(-1.0..1.0),
    );
    commands.spawn(( // Alle gespawnten Sachen sind ein Entity, sie haben die gleiche ID
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            ..default()
        })),
        Transform::from_translation(start_pos),
        Particle::new(start_pos, 0.5),
    ));
}
fn verlet_integration(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Particle)>,
) {
    let gravity = vec3(0.0, -3.0, 0.0);
    let dt = time.delta_secs();
    let dt_squared = dt * dt;

    for (mut transform, mut particle) in query.iter_mut() {
        particle.acceleration = gravity;

        let velocity = particle.pos - particle.old_pos;
        let new_pos = particle.pos + velocity + particle.acceleration * dt_squared;

        // Alte Position speichern
        particle.old_pos = particle.pos;
        particle.pos = new_pos;
        
        // Transform aktualisieren
        transform.translation = particle.pos;
    }
}

fn check_ground_collision(
    mut query: Query<&mut Particle>,
) {
    for mut particle in query.iter_mut() {
        let ground_level = particle.radius;
        
        if particle.pos.y <= ground_level {
            // Position korrigieren
            particle.pos.y = ground_level;
            
            // Bounce-Effekt durch Anpassung der old_pos (70% Dämpfung)
            let velocity_y = particle.pos.y - particle.old_pos.y;
            particle.old_pos.y = particle.pos.y + velocity_y * 0.8;
            
            
        }
    }
}