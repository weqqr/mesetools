struct VertexInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) texcoord: vec2f,
};

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) texcoord: vec2f,
};

struct Uniforms {
    forward: vec3f,
    fov: f32,
    position: vec3f,
    aspect_ratio: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> grid: array<u32>;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4(model.position, 1.0);
    out.texcoord = model.texcoord;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var ray: Ray;
    ray.origin = uniforms.position;
    ray.dir = get_ray_dir(uniforms.aspect_ratio, in.texcoord);
    ray.inv_dir = 1.0 / ray.dir;

    let box_dist = s_box(ray, vec3f(8, 8, 8), vec3f(8, 8, 8));
    if box_dist > 0.0 {
        ray.origin += ray.dir * (box_dist - 0.1);
    }

    var distance: f32;
    var normal: vec3f;
    var voxel: u32;

    let intersects = block_dda(ray, &distance, &normal, &voxel);

    if intersects {
        let hit_point = ray.origin + distance * ray.dir;
        let sun_dir = normalize(vec3(0.5, 0.7, 1.0));
        let light = saturate(max(dot(normal, sun_dir), 0.2));
        return vec4(light, 0.0, 0.0, 1.0);
    }

    return vec4(0.0, 0.0, 0.0, 1.0);
}

struct Ray {
    origin: vec3f,
    dir: vec3f,
    inv_dir: vec3f,
};

fn get_ray_dir(aspect_ratio: f32, texcoord: vec2f) -> vec3f {
    let up = vec3(0.0, 1.0, 0.0);
    let horizontal = cross(uniforms.forward, up);
    let vertical = cross(horizontal, uniforms.forward);

    let tan_half_fov = tan(uniforms.fov / 2.0);

    let x = (texcoord.x - 1.0) * horizontal * 2.0 * tan_half_fov * aspect_ratio;
    let y = (texcoord.y - 1.0) * vertical * 2.0 * tan_half_fov;

    return normalize(uniforms.forward + x + y);
}

const BLOCK_SIZE: u32 = 16;
const BLOCK_DDA_MAX_STEPS: u32 = 48;
const BLOCK_VOLUME = BLOCK_SIZE * BLOCK_SIZE * BLOCK_SIZE;

const SUPERBLOCK_SIZE: u32 = 8;
const SUPERBLOCK_DDA_MAX_STEPS: u32 = 24;
const SUPERBLOCK_VOLUME: u32 = SUPERBLOCK_SIZE * SUPERBLOCK_SIZE * SUPERBLOCK_SIZE;

fn superblock_dda(ray: Ray, distance: ptr<function, f32>, normal: ptr<function, vec3f>, voxel: ptr<function, u32>) -> bool {
    var r = ray;
    var intersects = false;

    var dda = dda_init(r);

    for (var i = 0u; i < SUPERBLOCK_DDA_MAX_STEPS; i += 1u) {
        dda_step(&dda);
        *voxel = fetch_voxel(dda.voxel_pos);

        let id = ((*voxel >> 24) & 0xFF) | ((*voxel >> 16) & 0xFF);
        if id != 0u {
            intersects = true;
            break;
        }

        if any(dda.voxel_pos > vec3i(i32(BLOCK_SIZE))) || any(dda.voxel_pos < vec3i(-1)) {
            break;
        }
    }
    dda_end(dda, r, distance, normal);
    return intersects;
}

fn block_dda(ray: Ray, distance: ptr<function, f32>, normal: ptr<function, vec3f>, voxel: ptr<function, u32>) -> bool {
    var r = ray;
    var intersects = false;

    var dda = dda_init(r);

    for (var i = 0u; i < BLOCK_DDA_MAX_STEPS; i += 1u) {
        dda_step(&dda);
        *voxel = fetch_voxel(dda.voxel_pos);

        let id = ((*voxel >> 24) & 0xFF) | ((*voxel >> 16) & 0xFF);
        if id != 0u {
            intersects = true;
            break;
        }

        if any(dda.voxel_pos > vec3i(i32(BLOCK_SIZE))) || any(dda.voxel_pos < vec3i(-1)) {
            break;
        }
    }
    dda_end(dda, r, distance, normal);
    return intersects;
}

struct DDAState {
    voxel_pos: vec3i,
    d_dist: vec3f,
    ray_step: vec3i,
    dist: vec3f,
    mask: vec3<bool>,
};

fn dda_init(ray: Ray) -> DDAState {
    var dda_state: DDAState;

    dda_state.voxel_pos = vec3i(floor(ray.origin));
    dda_state.d_dist = abs(vec3(length(ray.dir)) * ray.inv_dir);
    let s = sign(ray.dir);
    dda_state.ray_step = vec3i(s);
    dda_state.dist = (s * (vec3f(dda_state.voxel_pos) - ray.origin) + (s * 0.5) + 0.5) * dda_state.d_dist;

    return dda_state;
}

fn dda_step(dda: ptr<function, DDAState>) {
    let lt = (*dda).dist.xxy < (*dda).dist.yzz;
    if lt.x && lt.y {
        (*dda).dist.x += (*dda).d_dist.x;
        (*dda).voxel_pos.x += (*dda).ray_step.x;
        (*dda).mask = vec3<bool>(true, false, false);
    } else if !lt.x && lt.z {
        (*dda).dist.y += (*dda).d_dist.y;
        (*dda).voxel_pos.y += (*dda).ray_step.y;
        (*dda).mask = vec3<bool>(false, true, false);
    } else {
        (*dda).dist.z += (*dda).d_dist.z;
        (*dda).voxel_pos.z += (*dda).ray_step.z;
        (*dda).mask = vec3<bool>(false, false, true);
    }
}

fn dda_end(dda: DDAState, ray: Ray, distance: ptr<function, f32>, normal: ptr<function, vec3f>) {
    *normal = vec3f(dda.mask) * -sign(ray.dir);
    let mini = (vec3f(dda.voxel_pos) - ray.origin + 0.5 - 0.5 * vec3f(dda.ray_step)) * ray.inv_dir;
    *distance = max(mini.x, max(mini.y, mini.z));
}

fn fetch_voxel(pos: vec3i) -> u32 {
    let in_bounds = all(pos < vec3i(BLOCK_SIZE)) && all(pos >= vec3i(0));
    return select(0u, grid[u32(pos.x) + u32(pos.y) * BLOCK_SIZE + u32(pos.z) * BLOCK_SIZE * BLOCK_SIZE], in_bounds);
}

// http://iquilezles.org/www/articles/boxfunctions/boxfunctions.htm
fn s_box(ray: Ray, center: vec3f, radius: vec3f) -> f32 {
    let ro = ray.origin - center;
    let m = 1.0/ray.dir;
    let n = m*ro;
    let k = abs(m)*radius;

    let t1 = -n - k;
    let t2 = -n + k;

    let tN = max(max(t1.x, t1.y), t1.z);
    let tF = min(min(t2.x, t2.y), t2.z);
    if tN > tF || tF < 0.0 {
        return -1.0;
    }

    return tN;
}
