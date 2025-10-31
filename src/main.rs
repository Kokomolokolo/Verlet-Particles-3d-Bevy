use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
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
    fn draw(&self) {

    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // sphere
    // 1. **Define the Mesh (the Sphere) and add it to the Assets**
    let sphere_handle = meshes.add(Sphere::new(0.5)); // Assuming a radius of 0.5 based on your code

    // 2. **Define the Material and add it to the Assets**
    let material_handle = materials.add(Color::srgb_u8(124, 144, 255));

    // 3. **Spawn the Entity**
    commands.spawn((
        // Now we use the handle we created
        Mesh3d(sphere_handle),
        // Use the material handle
        MeshMaterial3d(material_handle), 
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}