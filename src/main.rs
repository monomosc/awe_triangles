mod resources;
use std::collections::BTreeMap;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::Rng;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use resources::{Speed, Paused, is_not_paused, VelocityVector};


const COUNT: usize = 40;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, (create_points, set_speed))
        .init_resource::<Speed>()
        .init_resource::<Paused>()
        .register_type::<CornerPhysics>()
        .add_systems(Update, toggle_pause)
        .add_systems(Update, (calc_new_speed, calc_new_position.run_if(is_not_paused), set_corner_positions))
        .run();
}

#[derive(Debug, Component, Clone, Reflect)]
pub struct CornerPhysics {
    pub speed: Vec2,
    pub pos: Vec2,
    pub velocity_arrow: Option<Entity>,
}
#[derive(Debug, Component, Reflect)]
pub struct CornerPartners {
    pub partner_1: Option<Entity>,
    pub partner_2: Option<Entity>,
}

impl CornerPhysics {
    pub fn new(pos: Vec2) -> Self {
        Self {
            speed: Vec2::new(0., 0.),
            pos: pos,
            velocity_arrow: None,
        }
    }
}
impl CornerPartners {
    pub fn new() -> Self {
        Self {
            partner_1: None,
            partner_2: None,
        }
    }
}

impl Default for CornerPhysics {
    fn default() -> Self {
        Self::new(Vec2 { x: 0.0, y: 0.0 })
    }
}
impl Default for CornerPartners {
    fn default() -> Self {
        Self::new()
    }
}

fn rnd_except_i(i: u32) -> u32 {
    use rand::distributions::Uniform;
    let range = Uniform::new(0, COUNT as u32);
    let mut rng = rand::thread_rng();
    let mut r = rng.sample(range);

    while r == i {
        r = rng.sample(range);
    }
    r
}
fn set_speed(mut speed: ResMut<Speed>) {
    *speed = resources::Speed(0.1);
}
fn create_points(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    let partners = (0..COUNT).into_iter().map(|i| {
        let mut x0 = 0;
        let mut x1 = 0;
        while x0 == x1 {
            x0 = rnd_except_i(i as u32);
            x1 = rnd_except_i(i as u32);
        }
        (x0, x1)
    }).collect::<Vec<_>>();
    let entities = (0..COUNT)
        .into_iter()
        .map(|_| commands.spawn_empty().id())
        .collect::<Vec<_>>();
    info!("Created {} entities", entities.len());
    let entities = entities.into_iter().zip(partners).collect::<Vec<_>>();

    let circle = meshes.add(Mesh::from(shape::Circle::new(5.)));
    let arrow = meshes.add(make_arrow_mesh());
    for (entity, (friend_1, friend_2)) in &entities {
        let color = Color::Rgba {
            red: rand::thread_rng().gen_range(0.0..1.0),
            green: rand::thread_rng().gen_range(0.0..1.0),
            blue: rand::thread_rng().gen_range(0.0..1.0),
            alpha: 1.0
        };

        let initial_pos = Vec2 {
            x: rand::thread_rng().gen_range(-300.0..300.0),
            y: rand::thread_rng().gen_range(-200.0..200.0),
        };
        let circle = commands.entity(*entity).insert((
            CornerPartners {
                partner_1: Some(entities[*friend_1 as usize].0),
                partner_2: Some(entities[*friend_2 as usize].0),
            },
            CornerPhysics::new(initial_pos),
            MaterialMesh2dBundle {
                mesh: circle.clone().into(),
                transform: Transform::from_translation(Vec3::new(
                    initial_pos.x,
                    initial_pos.y,
                    0.,
                )),
                material: materials.add(ColorMaterial::from(color)),
                ..Default::default()
            },
            Name::new("Corner")
        )).id();
        // add the pointer children
        let arrow_child = commands.spawn((MaterialMesh2dBundle {
            mesh: arrow.clone().into(),
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)).with_scale(Vec3::new(5.,5.,5.)),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            ..Default::default()
        },
            Name::new("Arrow")
        )).id();
        commands.entity(circle).push_children(&[arrow_child]);
    }
}

fn set_corner_positions(mut query: Query<(&CornerPhysics, &mut Transform)>) {
    for (corner, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(corner.pos.x, corner.pos.y, 0.);
    }
}

fn calc_new_position(mut query: Query<&mut CornerPhysics>, spd: Res<Speed>) {
    for mut corner in query.iter_mut() {
        let speed = { corner.speed };
        corner.pos += speed;
    }
}
fn toggle_pause(mut paused: ResMut<Paused>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        *paused = Paused(!paused.0);
    }
}

fn calc_new_speed(
    mut query: Query<(&mut CornerPhysics, &CornerPartners, Entity, &Children)>,
    mut vector_arrows: Query<&mut Transform>,
    time: Res<Time>,
    sim_speed: Res<Speed>,
) {
    let positions_with_id = query
        .iter()
        .map(|(corner, _, entity, _)| (entity.clone(), corner.clone()))
        .collect::<BTreeMap<_, _>>();
    

    for (mut corner_physics, corner_partners, _, children) in query.iter_mut() {
        let partner_1 = positions_with_id.get(&corner_partners.partner_1.unwrap()).unwrap();
        let partner_2 = positions_with_id.get(&corner_partners.partner_2.unwrap()).unwrap();
        let point_1 = partner_1.pos;
        let point_2 = partner_2.pos;

        //Calculate the midpoint of point_1 and point_2. This will be a point on the perpendicular bisector.
        let midpoint = Vec2 {
            x: (point_1.x + point_2.x) / 2.0,
            y: (point_1.y + point_2.y) / 2.0,
        };

        //Calculate the direction vector of the line connecting point_1 and point_2. This can be obtained by subtracting the coordinates of point_1 from point_2.
        //this is flipped because that is how you get a perpendicular vector
        let mut perpendicular_bisector_direction_vector = Vec2 {
            x: -(point_2.y - point_1.y),
            y: point_2.x - point_1.x,
        };
        perpendicular_bisector_direction_vector = perpendicular_bisector_direction_vector.normalize();
        

        //distance formula from https://en.wikipedia.org/wiki/Distance_from_a_point_to_a_line -> Vector formulation
        let distance_to_perpendicular_bisector = ((corner_physics.pos - midpoint) - ((corner_physics.pos - midpoint).dot(perpendicular_bisector_direction_vector) * perpendicular_bisector_direction_vector)).length();
        
        let distance_to_1 = (corner_physics.pos - point_1).length();
        let distance_to_2 = (corner_physics.pos - point_2).length();
        let mut direction_to_perpendicular_bisector = if distance_to_1 <= distance_to_2 { point_2 - point_1 } else { point_1 - point_2 };
        direction_to_perpendicular_bisector = direction_to_perpendicular_bisector.normalize();



        corner_physics.speed = direction_to_perpendicular_bisector
            * distance_to_perpendicular_bisector
            * time.delta_seconds()
            * sim_speed.0;

        //update vector entity if it exists
        if let Some(vector_entity) = children.get(0) {
            if let Ok(mut vector_transform) = vector_arrows.get_mut(*vector_entity) {
                
                *vector_transform = Transform::from_scale(Vec3::new(distance_to_perpendicular_bisector*0.1, 1., 1.));

                let mut angle = (corner_physics.speed.y / corner_physics.speed.x).atan();
                if (corner_physics.speed.y < 0.0 && corner_physics.speed.x < 0.0) || (corner_physics.speed.y > 0.0 && corner_physics.speed.x < 0.0) {
                    angle += std::f32::consts::PI;
                }
                *vector_transform = vector_transform.mul_transform(Transform::from_rotation(Quat::from_rotation_z(angle)));
                *vector_transform = vector_transform.mul_transform(Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)));
            }
        }
        
    }
}



/* fn show_vector_for_point(mut commands: Commands, point: &mut CornerPhysics) {
    let velocity_arrow = commands.spawn_empty().insert(
        VelocityVector,

    )

} */

const ARROW_SCALE_FACTOR: f32 = 10.0;
const ARROW_LENGTH: f32 = ARROW_SCALE_FACTOR;
const ARROW_HEIGHT_HALF: f32 = ARROW_SCALE_FACTOR/8.;

fn make_arrow_mesh() -> Mesh {
    let mut arrow = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    let v_pos: Vec<[f32; 3]> = vec!(
        [0.0,-ARROW_HEIGHT_HALF,0.0], 
        [ARROW_LENGTH,-ARROW_HEIGHT_HALF,0.0],
        [ARROW_LENGTH,-ARROW_HEIGHT_HALF * 2.,0.0],
        [ARROW_LENGTH + ARROW_HEIGHT_HALF * 2.,0.0, 0.0],
        [ARROW_LENGTH,ARROW_HEIGHT_HALF * 2.,0.0], 
        [ARROW_LENGTH,ARROW_HEIGHT_HALF,0.0],
        [0.0,ARROW_HEIGHT_HALF,0.0]);
    let indices = vec!(0,1,5, 5,6,0, 2,3,4);

    arrow.insert_attribute(Mesh::ATTRIBUTE_POSITION,  v_pos);
    arrow.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

    arrow

}
