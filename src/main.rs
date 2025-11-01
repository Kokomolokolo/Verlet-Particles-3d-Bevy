use bevy::{prelude::*, transform};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, DiagnosticsStore};
use rand::Rng;

use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add a system that prints FPS every second
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(Grid::new(2.0))
        .add_systems(Startup, setup)
        .insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0))
        .add_systems(
            FixedUpdate,
            (
                keyboard_input,
                //spawn_particle,
                physics_substeps,
                sync_particles_to_transforms,
                fps_display_system,
                camera_movment,
            ),
        )
        .run();
}
fn fps_display_system(diagnostics: Res<DiagnosticsStore>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(avg) = fps.average() {
            println!("FPS: {:.2}", avg);
        }
    }
}

#[derive(Component)]
struct Particle {
    pos: Vec3,
    old_pos: Vec3,
    acceleration: Vec3,
    radius: f32,
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
#[derive(Component)]
struct Ground;

const BOX_SIZE: f32 = 20.0;

#[derive(Component)]
struct FpsCamera {
    speed: f32,
    sensitivity: f32,
    yaw: f32,   // Rotation links/rechts
    pitch: f32, // Rotation hoch/runter
}
impl FpsCamera {
    fn new() -> Self {
        Self {
            speed: 5.0,
            sensitivity: 0.002,
            yaw: 0.0,
            pitch: 0.0,
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
        FpsCamera::new(),
    ));

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(BOX_SIZE / 2., 10.0, BOX_SIZE / 2.),
    ));
    // Particle Spawn
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(BOX_SIZE, BOX_SIZE))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            metallic: 0.0,
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(BOX_SIZE / 2.0, 0.0, BOX_SIZE / 2.0),
        Ground,
    ));
}

// WASD Camera movment
fn camera_movment(
    key: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &FpsCamera)>,
) {
    for (mut transform, fps_cam) in query.iter_mut() {
        // ECS basiert, obwohl es nur eine Kamera gibt
        let mut velocity = Vec3::ZERO;

        let forward = transform.forward();
        let right = transform.right();

        if key.pressed(KeyCode::KeyW) {
            velocity += *forward;
        }
        if key.pressed(KeyCode::KeyA) {
            velocity -= *right;
        }
        if key.pressed(KeyCode::KeyD) {
            velocity += *right;
        }
        if key.pressed(KeyCode::KeyS) {
            velocity -= *forward;
        }

        // Space / Shift
        if key.pressed(KeyCode::Space) {
            velocity.y += 1.0;
        }
        if key.pressed(KeyCode::ShiftLeft) {
            velocity.y -= 1.0;
        }

        transform.translation += velocity * fps_cam.speed * time.delta_secs();
    }
}

fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if keyboard.just_pressed(KeyCode::KeyE) {
        spawn_particle(commands, meshes, materials);
    }
}
fn spawn_particle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rand = rand::thread_rng();
    for _ in 0..10 {
        let start_pos = Vec3::new(
            rand.gen_range(2.0..18.0), // Innerhalb der Box (nicht -1 bis 1!)
            15.0,                      // Oben spawnen
            rand.gen_range(2.0..18.0),
        );
        commands.spawn((
            // Alle gespawnten Sachen sind ein Entity, sie haben die gleiche ID
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.2, 0.2),
                ..default()
            })),
            Transform::from_translation(start_pos),
            Particle::new(start_pos, 0.5),
        ));
    }
}

fn physics_substeps(
    mut grid: ResMut<Grid>,
    mut query: Query<(Entity, &mut Particle, &mut Transform)>
) {
    let substeps = 4;
    let dt = 1.0 / 60.0; // dt, s. in main 
    let sub_dt = dt / substeps as f32;

    for _ in 0..substeps {
        verlet_integration(sub_dt, &mut query);
        check_box_collision(&mut query);
        build_spatial_grid(&mut grid, &query);
        resolve_collisons(&grid, &mut query);
    }
}

fn verlet_integration(dt: f32, query: &mut Query<(Entity, &mut Particle, &mut Transform)>) {
    let gravity = vec3(0.0, -3.0, 0.0);
    let dt_squared = dt * dt;

    for (_, mut particle, _transform) in query.iter_mut() {
        particle.acceleration = gravity;

        let velocity = particle.pos - particle.old_pos;
        let new_pos = particle.pos + velocity + particle.acceleration * dt_squared;

        // Alte Position speichern
        particle.old_pos = particle.pos;
        particle.pos = new_pos;

        // Transform wird in eigener Funktion aktuallieisert.
    }
}

fn sync_particles_to_transforms(mut query: Query<(&Particle, &mut Transform)>) {
    for (particle, mut transform) in &mut query {
        transform.translation = particle.pos;
    }
}


fn check_box_collision(query: &mut Query<(Entity, &mut Particle, &mut Transform)>) {
    let damping = 0.8;
    for (_, mut particle, _) in query.iter_mut() {
        if particle.pos.y - particle.radius < 0.0 {
            particle.pos.y = particle.radius;
            particle.old_pos.y = particle.pos.y + (particle.pos.y - particle.old_pos.y) * damping;
        }

        // DECKE (oben) - Y = BOX_SIZE
        if particle.pos.y + particle.radius > BOX_SIZE {
            particle.pos.y = BOX_SIZE - particle.radius;
            particle.old_pos.y = particle.pos.y + (particle.pos.y - particle.old_pos.y) * damping;
        }

        // RECHTE WAND - X = BOX_SIZE
        if particle.pos.x + particle.radius > BOX_SIZE {
            particle.pos.x = BOX_SIZE - particle.radius;
            particle.old_pos.x = particle.pos.x + (particle.pos.x - particle.old_pos.x) * damping;
        }

        // LINKE WAND - X = 0
        if particle.pos.x - particle.radius < 0.0 {
            particle.pos.x = particle.radius;
            particle.old_pos.x = particle.pos.x + (particle.pos.x - particle.old_pos.x) * damping;
        }

        // VORDERE WAND - Z = BOX_SIZE
        if particle.pos.z + particle.radius > BOX_SIZE {
            particle.pos.z = BOX_SIZE - particle.radius;
            particle.old_pos.z = particle.pos.z + (particle.pos.z - particle.old_pos.z) * damping;
        }

        // HINTERE WAND - Z = 0
        if particle.pos.z - particle.radius < 0.0 {
            particle.pos.z = particle.radius;
            particle.old_pos.z = particle.pos.z + (particle.pos.z - particle.old_pos.z) * damping;
        }
    }
}
#[derive(Resource)] // verwendet für globale, einzigartige Dinge
struct Grid {
    cell_size: f32,
    grid: HashMap<(i32, i32, i32), Vec<Entity>>,
}
impl Grid {
    fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
        }
    }
    fn get_cell(&self, pos: Vec3) -> (i32, i32, i32) {
        (
            (pos.x / self.cell_size) as i32,
            (pos.y / self.cell_size) as i32,
            (pos.z / self.cell_size) as i32,
        )
    }
    fn clear(&mut self) {
        self.grid.clear()
    }
    fn insert(&mut self, pos: Vec3, entity: Entity) {
        let cell = self.get_cell(pos);
        self.grid.entry(cell).or_insert_with(Vec::new).push(entity)
    }
    fn get_neighbors(&self, pos: Vec3) -> Vec<Entity> {
        let cell = self.get_cell(pos);

        let mut nearby = Vec::new();

        if let Some(entities) = self.grid.get(&cell) {
            nearby.extend(entities);
        }

        let neighbor_cells = [
            // Gleiche Y-Ebene (8 Nachbarn)
            (cell.0 + 1, cell.1, cell.2),     // rechts
            (cell.0 + 1, cell.1, cell.2 + 1), // rechts vorne
            (cell.0, cell.1, cell.2 + 1),     // vorne
            (cell.0 - 1, cell.1, cell.2 + 1), // links vorne
            (cell.0 - 1, cell.1, cell.2),     // links
            (cell.0 - 1, cell.1, cell.2 - 1), // links hinten
            (cell.0, cell.1, cell.2 - 1),     // hinten
            (cell.0 + 1, cell.1, cell.2 - 1), // rechts hinten
            // Obere Y-Ebene (9 Nachbarn)
            (cell.0, cell.1 + 1, cell.2),         // oben mitte
            (cell.0 + 1, cell.1 + 1, cell.2),     // oben rechts
            (cell.0 + 1, cell.1 + 1, cell.2 + 1), // oben rechts vorne
            (cell.0, cell.1 + 1, cell.2 + 1),     // oben vorne
            (cell.0 - 1, cell.1 + 1, cell.2 + 1), // oben links vorne
            (cell.0 - 1, cell.1 + 1, cell.2),     // oben links
            (cell.0 - 1, cell.1 + 1, cell.2 - 1), // oben links hinten
            (cell.0, cell.1 + 1, cell.2 - 1),     // oben hinten
            (cell.0 + 1, cell.1 + 1, cell.2 - 1), // oben rechts hinten
            // Untere Y-Ebene (9 Nachbarn)
            (cell.0, cell.1 - 1, cell.2),         // unten mitte
            (cell.0 + 1, cell.1 - 1, cell.2),     // unten rechts
            (cell.0 + 1, cell.1 - 1, cell.2 + 1), // unten rechts vorne
            (cell.0, cell.1 - 1, cell.2 + 1),     // unten vorne
            (cell.0 - 1, cell.1 - 1, cell.2 + 1), // unten links vorne
            (cell.0 - 1, cell.1 - 1, cell.2),     // unten links
            (cell.0 - 1, cell.1 - 1, cell.2 - 1), // unten links hinten
            (cell.0, cell.1 - 1, cell.2 - 1),     // unten hinten
            (cell.0 + 1, cell.1 - 1, cell.2 - 1), // unten rechts hinten
        ];
        
        for cell in neighbor_cells {
            match self.grid.get(&cell) {
                Some(entities) => {
                    nearby.extend(entities)
                }
                None => {
                    // Existiert nichts in dieser Zelle, nichts wird gemacht
                }
            }
        }
        nearby
    }
}

fn build_spatial_grid(
    grid: &mut Grid,
    query: &Query<(Entity, &mut Particle, &mut Transform)>
) {
    grid.clear();

    for (entity, particle, _) in query.iter() {
        grid.insert(particle.pos, entity)
    }
}

fn resolve_collisons(
    grid: &Grid,
    query: &mut Query<(Entity, &mut Particle, &mut Transform)>
) {
    // Phasen System, da ich sonst mit einem doppelten for loop 2 borrow check probleme hätte, aufgrund von 2 mut queries.

    // Phase 1: Nur Kollisionen erkennen, 2 immutable referenzen: Beine können gleichzeitig existieren
    let mut collisions = Vec::new(); // Speicher alle einities mit Collisionen und ihrem correction_vektor
    for (entity_a, particle_a, _) in query.iter() {
        let neighbors = grid.get_neighbors(particle_a.pos);

        for &entity_b in &neighbors {
            if entity_a == entity_b { continue; }
            if entity_a >= entity_b { continue; }
            // Kollision sowie correcion
            match query.get(entity_b) {
                Ok((_, particle_b, _)) => {
                    let delta = particle_b.pos - particle_a.pos;
                    let min_dist = particle_a.radius + particle_b.radius;
                    let min_dist_squared = min_dist * min_dist;

                    let dist_squared = delta.length_squared();

                    if dist_squared < min_dist_squared && dist_squared > 0.0001 {
                        let dist = dist_squared.sqrt();

                        let overlap = min_dist - dist;

                        let direction = delta / dist;

                        let correction = direction * overlap * 0.5; // Positionskorrektion, 0.5 da so jeder Partikel gleich Korrigiert wird

                        collisions.push((entity_a, entity_b, correction))
                    }
                }
                Err(e) => {
                    println!("{}", e)
                }
            }
        }
    }

    // Phase 2: Die Korrektion der beiden Entities
    for (entity_a, entity_b, correction) in collisions {
        let damping = 0.8;
        if let Ok((_, mut particle_a, _)) = query.get_mut(entity_a) { // Das gleiche wie match query.get_mut { Ok((_, xxx )) => { ... } ... }
            particle_a.pos -= correction;
            particle_a.old_pos -= correction * damping;
        }
        
        if let Ok((_, mut particle_b, _)) = query.get_mut(entity_b) {
            particle_b.pos += correction;
            particle_b.old_pos += correction * damping;
        }
    }
}



// fn resolve_collisons_deprecated(mut query: Query<(Entity, &mut Particle)>) {
//     let mut combinations = query.iter_combinations_mut(); // Wie ein doppelter for loop: Geht in Bevy optimiert über jedes Paar einmal.
//     while let Some([(_entity_a, mut particle_a), (_entity_b, mut particle_b)]) =
//         combinations.fetch_next()
//     {
//         // Pattern matching, noch nicht wirklich was damit gemacht
//         let delta = particle_b.pos - particle_a.pos;
//         let min_dist = particle_a.radius + particle_b.radius;
//         let min_dist_squared = min_dist * min_dist;

//         let dist_squared = delta.length_squared();

//         if dist_squared < min_dist_squared && dist_squared > 0.0001 {
//             let dist = dist_squared.sqrt();

//             let overlap = min_dist - dist;

//             let direction = delta / dist;

//             let correction = direction * overlap * 0.5; // Positionskorrektion, 0.5 da so jeder Partikel gleich Korrigiert wird

//             // Position correction
//             particle_a.pos -= correction;
//             particle_b.pos += correction;

//             // Damping old_pos, to slow down the velocity
//             particle_a.old_pos -= correction * 0.5;
//             particle_b.old_pos += correction * 0.5;
//         }
//     }
// }
