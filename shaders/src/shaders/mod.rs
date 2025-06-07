use crate::shader_prelude::*;

mod a_lot_of_spheres;
mod a_question_of_time;
mod apollonian;
mod atmosphere_system_test;
mod bubble_buckey_balls;
mod clouds;
mod filtering_procedurals;
mod flappy_bird;
mod galaxy_of_universes;
mod geodesic_tiling;
mod heart;
mod luminescence;
mod mandelbrot_smooth;
mod miracle_snowflakes;
mod morphing;
mod moving_square;
mod on_off_spikes;
mod phantom_star;
mod playing_marble;
mod protean_clouds;
mod raymarching_primitives;
mod seascape;
mod skyline;
mod soft_shadow_variation;
mod tileable_water_caustic;
mod tokyo;
mod two_tweets;
mod voxel_pac_man;

#[allow(edition_2024_expr_fragment_specifier)]
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
    ($($shader_name:ident),* $(,)?) => {
        #[inline(always)]
        pub fn render_shader(shader_index: u32, shader_input: &ShaderInput, shader_output: &mut ShaderResult) {
            match_index!(shader_index; $(
                $shader_name::shader_fn(shader_input, shader_output),
            )*)
        }

        pub const SHADER_DEFINITIONS: &[ShaderDefinition] = &[
            $(
                $shader_name::SHADER_DEFINITION,
            )*
        ];
    };
}

render_shader_macro!(
    miracle_snowflakes,
    morphing,
    voxel_pac_man,
    luminescence,
    seascape,
    two_tweets,
    heart,
    clouds,
    mandelbrot_smooth,
    protean_clouds,
    tileable_water_caustic,
    apollonian,
    phantom_star,
    playing_marble,
    a_lot_of_spheres,
    a_question_of_time,
    galaxy_of_universes,
    atmosphere_system_test,
    soft_shadow_variation,
    bubble_buckey_balls,
    raymarching_primitives,
    moving_square,
    skyline,
    filtering_procedurals,
    geodesic_tiling,
    flappy_bird,
    tokyo,
    on_off_spikes,
);
