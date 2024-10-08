use bevy::prelude::*;

use crate::core::engine::hitbox::Hitbox;
use crate::LEVEL_H;
use crate::LEVEL_W;
use crate::WIN_W;
use crate::WIN_H;

use crate::core::engine::gravity::Gravity;
use crate::core::gameplay::player::Player;


const TILE_SIZE: u32 = 100;

const ENEMY_SPEED: f32 = 100.;
const ACCEL_RATE_X: f32 = 5000.;
const ACCEL_RATE_Y: f32 = 10800.;

const ANIM_TIME: f32 = 0.2;

const SPRITE_HEIGHT: u32 = 50;
const SPRITE_WIDTH: u32 = 30;

#[derive(Component)]
pub struct Enemy;

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
pub struct AnimationFrameCount(usize);

#[derive(Component)]
pub struct Velocity {
    velocity: Vec2,
}

impl Velocity {
    fn new() -> Self {
        Self {
            velocity: Vec2::splat(0.),
        }
    }
}

impl From<Vec2> for Velocity {
    fn from(velocity: Vec2) -> Self {
        Self { velocity }
    }
}

#[derive(Component)]
pub struct Health {
    hp: i32,
}

impl Health {
    fn new() -> Self {
        Self {
            hp: 100,
        }
    }
}

pub fn initialize(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
){
    let enemy_sheet_handle = asset_server.load("enemy_walking.png");
    let enemy_layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), 4, 1, None, None);
    let enemy_layout_len = enemy_layout.textures.len();
    let enemy_layout_handle = texture_atlases.add(enemy_layout);
    commands.spawn((
        SpriteBundle {
            texture: enemy_sheet_handle,
            transform: Transform {
                // Julianne 10/8: For now, enemy is being spawned at WIN_W. This will need to be changed eventually.
                translation: Vec3::new(WIN_W, -(WIN_H / 2.) + ((TILE_SIZE as f32) * 1.5), 900.),
                ..default()
            },
            sprite: Sprite {
                flip_x: false,
                ..default()
            },
            ..default()
        },
        TextureAtlas {
            layout: enemy_layout_handle,
            index: 0,
        },
        AnimationTimer(Timer::from_seconds(ANIM_TIME, TimerMode::Repeating)),
        AnimationFrameCount(enemy_layout_len),
        Velocity::new(),
        Health::new(),
        Gravity::new(),
        Hitbox::new(40 as f32, 40 as f32, Vec2::new(0., -210.)),
        Enemy,
    ));
}

pub fn enemy_gravity(
    time: Res<Time>, 
    mut enemy: Query<(&mut Transform, &mut Velocity, &mut Gravity, &mut Hitbox), With<Enemy>>, 
    hitboxes: Query<(&Hitbox), Without<Enemy>>
) {
    /*Julianne 10/8: This function is the same as player flight, but only makes the downward force on the enemy (no flight)*/
    let (mut pt, mut pv, mut pg, mut hb) = enemy.single_mut();

    let deltat = time.delta_seconds();

    //update gravity here
    pg.update_G(&pv.velocity.y, &deltat);
    pv.velocity.y = pg.get_G();
    

    let change = pv.velocity * deltat;
    let new_pos = pt.translation + change.extend(0.);
    let new_hb = Hitbox::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32, new_pos.xy());
    //Bound enemy to within level height

    if new_pos.y >= -(WIN_H / 2.) + (TILE_SIZE as f32) / 2.
        && new_pos.y <= WIN_H - (TILE_SIZE as f32) / 2.
        && !new_hb.all_enemy_collisions(&hitboxes)
    {
        pt.translation = new_pos;
        *hb = new_hb;
    }  
    
    let new_hb = Hitbox::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32, new_pos.xy());
    // Velocity is zero when enemy hits the ground
    if pt.translation.y <= -(LEVEL_H / 2.) + (TILE_SIZE as f32) ||
        new_hb.all_enemy_collisions(&hitboxes) 
    {
        pv.velocity.y = 0.;
    }
    //assumes the enemy is a square and pt.translation is the lower-left corner
}


pub fn animate_enemy(
    time: Res<Time>,
    mut enemy: Query<
        (
            &Velocity,
            &mut TextureAtlas,
            &mut AnimationTimer,
            &AnimationFrameCount,
        ),
        With<Enemy>,
    >,
) {
    let (v, mut texture_atlas, mut timer, frame_count) = enemy.single_mut();
    let x_vel = Vec2::new(v.velocity.x, 0.);
    //info!(x_vel.x);
    if x_vel.cmpne(Vec2::ZERO).any() {
        timer.tick(time.delta());

        if timer.just_finished() {
        texture_atlas.index = (texture_atlas.index + 1) % **frame_count;
         }
    }
}

/*Julianne 10/8: This finds if the player is on the left or right side
 and simply makes enemy walk towards the player, changing x translation only
 This also check if enemy is within camera frame. If they are not, they will not move*/
pub fn track_player(
    time: Res<Time>,
    mut enemy: Query<(&mut Transform, &mut Velocity, &mut Sprite, &mut Hitbox, &mut AnimationTimer), (With<Enemy>, Without<Player>)>,
    mut player: Query<(&mut Transform), (With<Player>, Without<Enemy>)>,
    hitboxes: Query<(&Hitbox), Without<Enemy>>, 
    mut camera: Query<&mut Transform, (Without<Player>, Without<Enemy>, With<Camera>)>
){
    //get enemy, player and camera
    let (mut et, mut ev, mut es, mut ehb, mut timer) = enemy.single_mut();
    let pt = player.single_mut();
    let cam_t = camera.single_mut();

    let mut deltav_x = 0.;

    // Is enemy within the camera frame?
    if et.translation.x > cam_t.translation.x + (WIN_W/2.){
        return}
    else{
        timer.tick(time.delta());
    }
    

    //face player and walk towards player
    if pt.translation.x >= et.translation.x {
        deltav_x += 1.;
        es.flip_x=false;
    }
    else{
        deltav_x -= 1.;
        es.flip_x = true;
    }

    let deltat = time.delta_seconds();
    let acc_x = ACCEL_RATE_X * deltat;

    if deltav_x != 0. {
        if ev.velocity.y >= 0. {
            ev.velocity.x = (ev.velocity.x + deltav_x * acc_x).clamp(-ENEMY_SPEED, ENEMY_SPEED);
        }
        else {
            ev.velocity.x = (ev.velocity.x + deltav_x * acc_x).clamp(-ENEMY_SPEED * 0.3, ENEMY_SPEED * 0.3);
        }
    } else if ev.velocity.x.abs() > acc_x {
        ev.velocity.x -= ev.velocity.x.signum() * acc_x;
    } else {
        ev.velocity.x = 0.;
    }

    let change = ev.velocity * deltat;
    let new_pos = et.translation + change.extend(0.);
    let new_hb = Hitbox::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32, new_pos.xy());

    if new_pos.x >= -(WIN_W / 2.) + (TILE_SIZE as f32) / 2.
        && new_pos.x <= LEVEL_W - (WIN_W / 2. + (TILE_SIZE as f32) / 2.)
        && !new_hb.all_enemy_collisions(&hitboxes)
    {
        et.translation = new_pos;
        *ehb = new_hb;
    }
}