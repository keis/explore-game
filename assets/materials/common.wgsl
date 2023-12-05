#import noisy_bevy::fbm_simplex_2d

fn modulo(a: f32, n: f32) -> f32 {
    return a - n * floor(a / n);
}

fn pixel_noise(world_uv: vec2<f32>, color_a: vec4<f32>, color_b: vec4<f32>, color_c: vec4<f32>) -> vec4<f32> {
    let octaves = 2;
    let lacunarity = 4.0;
    let gain = 0.5;

    let floor_world_uv = floor(world_uv * 16.0) / 16.0;
    let noise =  fbm_simplex_2d(floor_world_uv, octaves, lacunarity, gain) + 1.0;

    let frac = 1.0 / 3.0;
    if noise > frac * 5.0 {
        return color_a;
    } else if noise > frac * 4.0 {
        return color_b;
    } else if noise > frac * 3.0 {
        return color_c;
    } else if noise > frac * 2.0 {
        return  mix(color_a, color_b, 0.5);
    } else if noise > frac * 1.0 {
        return mix(color_a, color_c, 0.5);
    } else {
        return mix(color_b, color_c, 0.5);
    }
}
