use crate::shader_prelude::*;

pub mod a_lot_of_spheres;
pub mod a_question_of_time;
pub mod apollonian;
pub mod atmosphere_system_test;
pub mod bubble_buckey_balls;
pub mod clouds;
pub mod filtering_procedurals;
pub mod flappy_bird;
pub mod galaxy_of_universes;
pub mod geodesic_tiling;
pub mod heart;
pub mod loading_repeating_circles;
pub mod luminescence;
pub mod mandelbrot_smooth;
pub mod miracle_snowflakes;
pub mod morphing;
pub mod moving_square;
pub mod on_off_spikes;
pub mod phantom_star;
pub mod playing_marble;
pub mod protean_clouds;
pub mod raymarching_primitives;
pub mod seascape;
pub mod skyline;
pub mod soft_shadow_variation;
pub mod tileable_water_caustic;
pub mod tokyo;
pub mod two_tweets;
pub mod voxel_pac_man;

macro_rules! match_index {
    ($e:expr; $($result:expr),* $(,)?) => ({
        let mut i = 0..;
        match $e { e => {
            $(if e == i.next().unwrap() { $result } else)*
            { unreachable!() }
        }}
    })
}

macro_rules! render_shader_macro {
    ($num_shaders:expr, $($shader_name:ident),* $(,)?) => {
        #[inline(always)]
        pub fn render_shader(shader_index: u32, shader_input: &ShaderInput, shader_output: &mut ShaderResult) {
            match_index!(shader_index; $(
                $shader_name::shader_fn(shader_input, shader_output),
            )*)
        }

        pub const SHADER_DEFINITIONS: [ShaderDefinition; $num_shaders] = [
            $(
                $shader_name::SHADER_DEFINITION,
            )*
        ];
    };
}

render_shader_macro!(1, loading_repeating_circles,);

/*
render_shader_macro!(
  29,
  loading_repeating_circles,
  two_tweets,
  heart,
  clouds,
  mandelbrot_smooth,
  protean_clouds,
  tileable_water_caustic,
  apollonian,
  phantom_star,
  seascape,
  playing_marble,
  a_lot_of_spheres,
  a_question_of_time,
  galaxy_of_universes,
  atmosphere_system_test,
  soft_shadow_variation,
  miracle_snowflakes,
  morphing,
  bubble_buckey_balls,
  raymarching_primitives,
  moving_square,
  skyline,
  filtering_procedurals,
  geodesic_tiling,
  flappy_bird,
  tokyo,
  on_off_spikes,
  luminescence,
  voxel_pac_man,
);
*/
