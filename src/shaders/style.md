# GLSL Style Guide

Avoid name collisions and make shader interfaces obvious.

## 1. Mandatory Prefixes

- `a_` — vertex attributes
- `u_` — uniforms
- `v_` — vertex → fragment varyings
- `o_` — fragment outputs

## 2. Naming Format

prefix_camelCase

Examples:
- `a_pos`
- `v_normal`
- `u_viewProjMatrix`
- `o_fragColor`

## 4. Layout Rules

- Use explicit `layout(location = …)` on fragment outputs for more than one output
- Varyings must match exactly between VS and FS

## 5. Hard Rules

- No unprefixed globals
- No mixed naming styles
- No type info in names (`v3_`, `m4_`)
