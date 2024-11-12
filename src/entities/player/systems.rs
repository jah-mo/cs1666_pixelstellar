use core::num;
use std::cmp::{min, max};

use bevy::prelude::*;
use super::{blaster::{self, components::*}, components::*, resources::*};
use crate::{
    common::{
        hitbox::Hitbox,
        gravity::Gravity,
    },
    entities::particle::resources::ParticleMap,
    entities::enemy::components::Enemy,
    entities::particle::components::ParticleElement,
    LEVEL_H,
    LEVEL_W,
};





pub fn initialize(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
){
    let player_sheet_handle = asset_server.load("walking.png");
    //               used to be tilesize. removed TILE_SIZE and now at 100, but change as needed  \/
    let player_layout = TextureAtlasLayout::from_grid(UVec2::splat(100), 4, 1, None, None);
    let player_layout_len = player_layout.textures.len();
    let player_layout_handle = texture_atlases.add(player_layout);
    commands.spawn((
        SpriteBundle {
            texture: player_sheet_handle,
            transform: Transform {
                translation: Vec3::new(0., 100.0, 900.),
                ..default()
            },
            sprite: Sprite {
                // Flip the logo to the left
                flip_x: false,
                ..default()
            },
            ..default()
        },
        TextureAtlas {
            layout: player_layout_handle,
            index: 0,
        },
        AnimationTimer(Timer::from_seconds(ANIM_TIME, TimerMode::Repeating)),
        AnimationFrameCount(player_layout_len),
        Velocity::new(),
        Health::new(100.0),
        Gravity::new(),
        Hitbox::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32, Vec2::new(0., 110.)),
        Player,
    ));

    commands.insert_resource(PlayerRatioWaterParticles{
        number: 0.0,
    });
}

pub fn move_player(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Transform, &mut Velocity, &mut Sprite, &mut Hitbox, &mut Health), With<Player>>,
    hitboxes: Query<&Hitbox, Without<Player>>,
    enemy_hitboxes: Query<&Hitbox, (With<Enemy>, Without<Player>)>,
    mut blaster_transform: Query<&mut Transform, (With<Blaster>, Without<Enemy>, Without<Player>)>,
    map: ResMut<ParticleMap>,
) {
    let (mut pt, mut pv, mut ps, mut hb, mut player_health) = player.single_mut();
    let mut deltav_x = 0.;
    let mut bt = blaster_transform.single_mut();

    if input.pressed(KeyCode::KeyA) {
        if (pt.translation.x >= -(LEVEL_W / 2.) + (SPRITE_WIDTH as f32) / 2.){
            deltav_x -= 1.;
            ps.flip_x = true;
        }
    }

    if input.pressed(KeyCode::KeyD) {
        if pt.translation.x <= LEVEL_W - (LEVEL_W / 2. + (SPRITE_WIDTH as f32) / 2.){
            deltav_x += 1.;
            ps.flip_x = false;
        }
    }
    let deltat = time.delta_seconds();
    let acc_x = ACCEL_RATE_X * deltat;

    if deltav_x != 0. {
        if pv.velocity.y >= 0. {
            pv.velocity.x = (pv.velocity.x + deltav_x * acc_x).clamp(-PLAYER_MAX_SPEED, PLAYER_MAX_SPEED);
        }
        else {
            pv.velocity.x = (pv.velocity.x + deltav_x * acc_x).clamp(-PLAYER_MAX_SPEED * 0.5, PLAYER_MAX_SPEED * 0.5);
        }
    } else if pv.velocity.x.abs() > acc_x {
        pv.velocity.x -= pv.velocity.x.signum() * acc_x;
    } else {
        pv.velocity.x = 0.;
    }

    //Account for player in water
    let ratio_of_water_particles = hb.ratio_of_water_grid_tiles(&map);
    if ratio_of_water_particles > 0.0 {
        pv.velocity.x = pv.velocity.x * (1. - 0.7 * ratio_of_water_particles.powf(0.5));
    }

    let change = pv.velocity * deltat;
    let new_pos = pt.translation + change.extend(0.);
    let new_hb = Hitbox::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32, new_pos.xy());

    
    if new_hb.player_enemy_collision(&enemy_hitboxes){
        player_health.current -=1.;
    }
    if new_pos.x >= -(LEVEL_W / 2.) + (SPRITE_WIDTH as f32) / 2.
        && new_pos.x <= LEVEL_W - (LEVEL_W / 2. + (SPRITE_WIDTH as f32) / 2.)
        && !new_hb.all_player_collisions(&hitboxes)
    {
        pt.translation = new_pos;
        *hb = new_hb;
        bt.translation.x = pt.translation.x + BLASTER_OFFSET_X;
        bt.translation.y = pt.translation.y + BLASTER_OFFSET_Y;
    }
    //info!("{}", pt.translation);

}

pub fn flight(
    time: Res<Time>, 
    input: Res<ButtonInput<KeyCode>>, 
    mut player: Query<(&mut Transform, &mut Velocity, &mut Gravity, &mut Hitbox), With<Player>>, 
    hitboxes: Query<&Hitbox, Without<Player>>,
    mut blaster_transform: Query<&mut Transform, (With<Blaster>, Without<Enemy>, Without<Player>)>,
    map: ResMut<ParticleMap>,
    mut player_ratio_water_particles: ResMut<PlayerRatioWaterParticles>,
    mut commands: Commands,
) {
    let (mut pt, mut pv, mut pg, mut hb) = player.single_mut();
    let mut bt = blaster_transform.single_mut();
    let deltat = time.delta_seconds();
    let acc_y = ACCEL_RATE_Y * deltat;

    if input.pressed(KeyCode::Space) {
        if pt.translation.y <= (LEVEL_H / 2.) - (SPRITE_HEIGHT as f32) / 2. {
            pg.reset_g();
            pv.velocity.y = f32::min(MAX_FLIGHT_SPEED, pv.velocity.y + (1. * acc_y))
        }
        else {
            pg.reset_g();
            pv.velocity.y = 0.0;
        }
    } else {
        pg.update_g(&pv.velocity.y, &deltat);
        pv.velocity.y = pg.get_g();
    }
    //Account for player in water
    let ratio_of_water_particles = hb.ratio_of_water_grid_tiles(&map);
    if ratio_of_water_particles > 0.0 {
        pv.velocity.y = pv.velocity.y * (1. - 0.7 * ratio_of_water_particles.powf(0.5));
    }
    let change = pv.velocity * deltat;
    let new_pos = pt.translation + change.extend(0.);
    let new_hb = Hitbox::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32, new_pos.xy());
    //Bound player to within level height

    if new_pos.y >= -(LEVEL_H / 2.) + (SPRITE_HEIGHT as f32) / 2.
        && new_pos.y <= (LEVEL_H / 2.) - (SPRITE_HEIGHT as f32) / 2.
        && !new_hb.all_player_collisions(&hitboxes)
    {
        pt.translation = new_pos;
        *hb = new_hb;
        bt.translation.x = pt.translation.x + BLASTER_OFFSET_X;
        bt.translation.y = pt.translation.y + BLASTER_OFFSET_Y;
    }  
    
    let new_hb = Hitbox::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32, new_pos.xy());
    // Velocity is zero when player hits the ground
    if pt.translation.y <= -(LEVEL_H / 2.) + (SPRITE_HEIGHT as f32) ||
        new_hb.all_player_collisions(&hitboxes) 
    {
        pv.velocity.y = 0.;
    }
    //assumes the player is a square and pt.translation is the lower-left corner

    //update number of water particles the player is in

    player_ratio_water_particles.number = water_splash(&mut player_ratio_water_particles, &hb, map, &pv, commands);

}

pub fn animate_player(
    time: Res<Time>,
    mut player: Query<
        (
            &Velocity,
            &mut TextureAtlas,
            &mut AnimationTimer,
            &AnimationFrameCount,
        ),
        With<Player>,
    >,
) {
    let (v, mut texture_atlas, mut timer, frame_count) = player.single_mut();
    let x_vel = Vec2::new(v.velocity.x, 0.);
    //info!(x_vel.x);
    if x_vel.cmpne(Vec2::ZERO).any() {
        timer.tick(time.delta());

        if timer.just_finished() {
        texture_atlas.index = (texture_atlas.index + 1) % **frame_count;
         }
    }
}

fn water_splash(
    player_ratio_water_particles: &mut ResMut<PlayerRatioWaterParticles>,
    hb: &Hitbox, 
    mut map: ResMut<ParticleMap>,
    pv: &Velocity,
    mut commands: Commands,
) -> f32 {
    let new_ratio = hb.ratio_of_water_grid_tiles(&map);
    if new_ratio / player_ratio_water_particles.number > SPLASH_THRESHOLD {
        let num_water_particles_occupied = hb.number_of_water_grid_tiles_colliding(&map);
        let num_water_particles_to_splash = ((new_ratio - player_ratio_water_particles.number) * num_water_particles_occupied as f32 * pv.velocity.length() / PLAYER_MAX_SPEED as f32) as i32;

        if num_water_particles_to_splash > 0 {
            // Actually splash the water particles
            let (top_left_x, top_left_y, bottom_right_x, bottom_right_y) = hb.get_grid_tiles_to_check();
            let center_x = (top_left_x + bottom_right_x) / 2;
            let mut row_offset = 0;
            let mut count = 0;
            info!("Number of water particles to splash: {}", num_water_particles_to_splash);

            let mut num_while_ran = 0;
            while (num_while_ran == 0) {
                info!("Coords: {}, {}, {}, {}", top_left_x, top_left_y, bottom_right_x, bottom_right_y);
                let lower_x = min(bottom_right_x, top_left_x);
                let upper_x = max(bottom_right_x, top_left_x);
                let lower_y = min(bottom_right_y, top_left_y);
                let upper_y = max(bottom_right_y, top_left_y);
                for i in lower_x..upper_x{
                    for j in lower_y..upper_y{
                        if map.get_element_at((i, j)) == ParticleElement::Water {
                            info!("Giving velocity");
                            map.delete_at(&mut commands, (i,j));
                            //map.give_velocity(&mut commands, (i, j), Vec2::new(0., 10000.)); 
                            //map.insert_at(commands, pos, list)
                            //map.insert_at::<WaterParticle>(&mut commands, (x, y), ListType::ReplaceOnlyAir)
                            
                            count += 1;
                            if count >= num_water_particles_to_splash {
                                break;
                            }
                        }
                    }
                }
                num_while_ran += 1;
            }
            


            // loop {
            //     let y = top_left_y + row_offset;

            //     // Stop at bottom row
            //     if y > bottom_right_x  {
            //         break;
            //     }

            //     // Iterate from top center outwards (left and right)
            //     for offset in 0..=(bottom_right_x - top_left_x) / 2 {
            //         let left_x = center_x.saturating_sub(offset);
            //         let right_x = center_x + offset;

            //         if left_x >= top_left_x && map.get_element_at((left_x, y)) == ParticleElement::Water {
            //             //give velocity
            //             info!("Giving velocity");
            //             map.give_velocity(&mut commands, (left_x, y), Vec2::new(0., 100.)); 
            //             count += 1;
            //             if count >= num_water_particles_to_splash {
            //                 break;
            //             }
            //         }

            //         if right_x <= bottom_right_x && map.get_element_at((right_x, y)) == ParticleElement::Water {
            //             info!("Giving velocity");
            //             map.give_velocity(&mut commands, (right_x, y), Vec2::new(0., 100.)); 
            //             count += 1;
            //             if count >= num_water_particles_to_splash {
            //                 break;
            //             }
            //         }
            //     }

            //     row_offset += 1;
            // }
        }
    }

    new_ratio
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // Startup events
        app.add_systems(Startup, initialize);

        // Update events
        app.add_systems(Update, move_player);   
        app.add_systems(Update, flight.after(super::systems::move_player));
        app.add_systems(Update, animate_player.after(super::systems::move_player));
        
        // Blaster systems
        // Event
        app.add_event::<super::blaster::components::ChangeBlasterEvent>();

        // Startup events
        app.add_systems(Startup, super::blaster::systems::initialize.after(initialize));
        
        // Update events
        app.add_systems(Update, super::blaster::systems::update_blaster_aim);
        app.add_systems(Update, super::blaster::systems::shoot_blaster);
        app.add_systems(Update, super::blaster::systems::handle_blaster_change_input);
        app.add_systems(Update, super::blaster::systems::change_blaster_on_event);
     //   app.add_system(super::blaster::systems::switch_blaster.system());
      //  app.add_system(super::blaster::systems::handle_blaster_switch.system());
        

    }
} 

