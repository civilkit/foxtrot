/// Takes a STEP file (as an array of bytes), and returns a triangle mesh.
///
/// Vertices are packed into rows of 9 floats, representing
/// - Position
/// - Normal
/// - Color
///
/// Vertices are rows of three indexes into the triangle array
///
use wasm_bindgen::prelude::*;
use log::{Level};

#[wasm_bindgen]
pub fn init_log() {
    console_log::init_with_level(Level::Info).expect("Failed to initialize log");
}

#[wasm_bindgen]
pub fn step_to_triangle_buf(data: String) -> Vec<f32> {
    use step::step_file::StepFile;
    use triangulate::triangulate::triangulate; // lol

    let flat = StepFile::strip_flatten(data.as_bytes());
    let step = StepFile::parse(&flat);
    let (mut mesh, _stats) = triangulate(&step);

    let (mut xmin, mut xmax) = (std::f64::INFINITY, -std::f64::INFINITY);
    let (mut ymin, mut ymax) = (std::f64::INFINITY, -std::f64::INFINITY);
    let (mut zmin, mut zmax) = (std::f64::INFINITY, -std::f64::INFINITY);
    for pos in mesh.verts.iter().map(|p| p.pos) {
        xmin = xmin.min(pos.x);
        xmax = xmax.max(pos.x);
        ymin = ymin.min(pos.y);
        ymax = ymax.max(pos.y);
        zmin = ymin.min(pos.z);
        zmax = ymax.max(pos.z);
    }
    let scale = (xmax - xmin).max(ymax - ymin).max(zmax - zmin);
    let xc = (xmax + xmin) / 2.0;
    let yc = (ymax + ymin) / 2.0;
    let zc = (zmax + zmin) / 2.0;
    for pos in mesh.verts.iter_mut().map(|p| &mut p.pos) {
        pos.x = (pos.x - xc) / scale * 200.0;
        pos.y = (pos.y - yc) / scale * 200.0;
        pos.z = (pos.z - zc) / scale * 200.0;
    }

    mesh.triangles.iter()
        .flat_map(|v| v.verts.iter())
        .map(|p| &mesh.verts[*p as usize])
        .flat_map(|v| v.pos.iter().chain(&v.norm).chain(&v.color))
        .map(|f| *f as f32)
        .collect()
}

/// What the CivilKit preview needs: the mesh in the file's own units (no recentring or rescaling), one triangle per
/// 9 floats (`positions`, `normals`), and how many faces could not be meshed.
#[wasm_bindgen]
pub struct Preview {
    positions: Vec<f32>,
    normals: Vec<f32>,
    shells: u32,
    faces: u32,
    errors: u32,
    panics: u32,
}

#[wasm_bindgen]
impl Preview {
    #[wasm_bindgen(getter)] pub fn positions(&self) -> Vec<f32> { self.positions.clone() }
    #[wasm_bindgen(getter)] pub fn normals(&self) -> Vec<f32> { self.normals.clone() }
    #[wasm_bindgen(getter)] pub fn shells(&self) -> u32 { self.shells }
    #[wasm_bindgen(getter)] pub fn faces(&self) -> u32 { self.faces }
    #[wasm_bindgen(getter)] pub fn errors(&self) -> u32 { self.errors }
    #[wasm_bindgen(getter)] pub fn panics(&self) -> u32 { self.panics }
}

#[wasm_bindgen]
pub fn skip_face(id: usize) { triangulate::triangulate::guard::skip(id); }

/// The face being meshed: after a trap, the one to `skip_face` on the next run.
#[wasm_bindgen]
pub fn current_face() -> usize { triangulate::triangulate::guard::current() }

#[wasm_bindgen]
pub fn step_to_preview(data: &[u8]) -> Preview {
    use step::step_file::StepFile;
    use triangulate::triangulate::triangulate;

    let flat = StepFile::strip_flatten(data);
    let step = StepFile::parse(&flat);
    let (mesh, stats) = triangulate(&step);
    let n = mesh.triangles.len();
    let (mut positions, mut normals) = (Vec::with_capacity(n * 9), Vec::with_capacity(n * 9));
    for t in &mesh.triangles {
        for &i in t.verts.iter() {
            let v = &mesh.verts[i as usize];
            positions.extend(v.pos.iter().map(|&f| f as f32));
            normals.extend(v.norm.iter().map(|&f| f as f32));
        }
    }
    Preview { positions, normals, shells: stats.num_shells as u32, faces: stats.num_faces as u32, errors: stats.num_errors as u32, panics: stats.num_panics as u32 }
}
