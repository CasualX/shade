#version 300 es

/*
MIT License

Copyright (c) 2025 Matt Sephton @gingerbeardman

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

precision highp float;

out vec4 o_fragColor;

in vec2 v_uv;

uniform sampler2D u_texture;
uniform float u_scanline_intensity;
uniform float u_scanline_count;
uniform float u_brightness;
uniform float u_contrast;
uniform float u_saturation;
uniform float u_bloom_intensity;
uniform float u_bloom_threshold;
uniform float u_rgb_shift;
uniform float u_adaptive_intensity;
uniform float u_vignette_strength;
uniform float u_curvature;
uniform float u_flicker_strength;
uniform float u_time;

const float PI = 3.14159265;
const vec3 LUMA = vec3(0.299, 0.587, 0.114);
const float BLOOM_THRESHOLD_FACTOR = 0.5;
const float BLOOM_FACTOR_MULT = 1.5;
const float RGB_SHIFT_SCALE = 0.005;
const float RGB_SHIFT_INTENSITY = 0.08;
const float Y_OFFSET_SPEED = 0.03;

vec2 curve_remap_uv(vec2 uv, float curvature) {
	vec2 coords = uv * 2.0 - 1.0;
	float curve_amount = curvature * 0.25;
	float dist = dot(coords, coords);
	coords *= 1.0 + dist * curve_amount;
	return coords * 0.5 + 0.5;
}

vec4 sample_bloom(sampler2D tex, vec2 uv, float radius, vec4 center_sample) {
	vec2 offset = vec2(radius);
	vec4 center = center_sample * 0.4;
	vec4 cross = (
		texture(tex, uv + vec2(offset.x, 0.0)) +
		texture(tex, uv - vec2(offset.x, 0.0)) +
		texture(tex, uv + vec2(0.0, offset.y)) +
		texture(tex, uv - vec2(0.0, offset.y))
	) * 0.15;
	return center + cross;
}

float vignette_approx(vec2 uv, float strength) {
	vec2 vig_coord = uv * 2.0 - 1.0;
	float dist = max(abs(vig_coord.x), abs(vig_coord.y));
	return 1.0 - dist * dist * strength;
}

void main() {
	vec2 uv = v_uv;

	if (u_curvature > 0.001) {
		uv = curve_remap_uv(uv, u_curvature);
		if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
			o_fragColor = vec4(0.0);
			return;
		}
	}

	vec4 pixel = texture(u_texture, uv);

	if (u_bloom_intensity > 0.001) {
		float pixel_luma = dot(pixel.rgb, LUMA);
		float bloom_threshold_half = u_bloom_threshold * BLOOM_THRESHOLD_FACTOR;
		if (pixel_luma > bloom_threshold_half) {
			vec4 bloom_sample = sample_bloom(u_texture, uv, 0.005, pixel);
			bloom_sample.rgb *= u_brightness;
			float bloom_luma = dot(bloom_sample.rgb, LUMA);
			float bloom_factor = u_bloom_intensity * max(0.0, (bloom_luma - u_bloom_threshold) * BLOOM_FACTOR_MULT);
			pixel.rgb += bloom_sample.rgb * bloom_factor;
		}
	}

	if (u_rgb_shift > 0.005) {
		float shift = u_rgb_shift * RGB_SHIFT_SCALE;
		pixel.r += texture(u_texture, vec2(uv.x + shift, uv.y)).r * RGB_SHIFT_INTENSITY;
		pixel.b += texture(u_texture, vec2(uv.x - shift, uv.y)).b * RGB_SHIFT_INTENSITY;
	}

	pixel.rgb *= u_brightness;

	float luminance = dot(pixel.rgb, LUMA);
	pixel.rgb = (pixel.rgb - 0.5) * u_contrast + 0.5;
	pixel.rgb = mix(vec3(luminance), pixel.rgb, u_saturation);

	float lighting_mask = 1.0;

	if (u_scanline_intensity > 0.001) {
		float y_offset = u_time * Y_OFFSET_SPEED;
		float scanline_y = (uv.y + y_offset) * u_scanline_count;
		float scanline_pattern = abs(sin(scanline_y * PI));

		float adaptive_factor = 1.0;
		if (u_adaptive_intensity > 0.001) {
			float y_pattern = sin(uv.y * 30.0) * 0.5 + 0.5;
			adaptive_factor = 1.0 - y_pattern * u_adaptive_intensity * 0.2;
		}

		lighting_mask *= 1.0 - scanline_pattern * u_scanline_intensity * adaptive_factor;
	}

	if (u_flicker_strength > 0.001) {
		lighting_mask *= 1.0 + sin(u_time * 110.0) * u_flicker_strength;
	}

	if (u_vignette_strength > 0.001) {
		lighting_mask *= max(vignette_approx(uv, u_vignette_strength), 0.0);
	}

	pixel.rgb *= lighting_mask;
	o_fragColor = pixel;
}
