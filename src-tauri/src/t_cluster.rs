use crate::t_sqlite::{Face, Person};

/// DBSCAN parameters
// const EPSILON: f32 = 0.42; // Now read from config
const MIN_SAMPLES: usize = 2; // Minimum number of faces to form a cluster

/// Calculate cosine distance between two embeddings
/// Distance = 1.0 - Cosine Similarity
fn cosine_distance(a: &[u8], b: &[u8]) -> f32 {
    let emb1: Vec<f32> = a
        .chunks(4)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();

    let emb2: Vec<f32> = b
        .chunks(4)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();

    let dot_product: f32 = emb1.iter().zip(emb2.iter()).map(|(x, y)| x * y).sum();
    let norm1: f32 = emb1.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm2: f32 = emb2.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm1 == 0.0 || norm2 == 0.0 {
        return 1.0;
    }

    let similarity = dot_product / (norm1 * norm2);
    // Clamp similarity to [-1.0, 1.0] to handle floating point errors
    let similarity = similarity.clamp(-1.0, 1.0);
    1.0 - similarity
}

/// Run DBSCAN clustering on ALL faces (resets existing assignments)
pub fn cluster_faces(epsilon: f32) -> Result<usize, String> {
    // 1. Reset all existing assignments and delete persons
    Face::reset_all_assignments()?;

    // 2. Get ALL faces for re-clustering
    let faces = Face::get_all()?;
    if faces.is_empty() {
        return Ok(0);
    }

    let n = faces.len();
    let mut labels = vec![0; n]; // 0: undefined, -1: noise, >0: cluster id
    let mut cluster_id = 0;

    // 2. DBSCAN Algorithm
    for i in 0..n {
        if labels[i] != 0 {
            continue;
        }

        let neighbors = region_query(&faces, i, epsilon);

        if neighbors.len() < MIN_SAMPLES {
            labels[i] = -1; // Noise
        } else {
            cluster_id += 1;
            expand_cluster(&faces, &mut labels, i, &neighbors, cluster_id, epsilon);
        }
    }

    // 3. Create persons for clusters and assign faces
    let mut total_assigned = 0;
    for cid in 1..=cluster_id {
        // Collect faces in this cluster
        let cluster_indices: Vec<usize> = labels
            .iter()
            .enumerate()
            .filter(|&(_, &l)| l == cid)
            .map(|(i, _)| i)
            .collect();

        if cluster_indices.is_empty() {
            continue;
        }

        // Create a new person
        // Check if there's any existing person we might match? (Future improvement)
        let person_name = format!("Person {}", cid); // Temporary naming
        let person_id = Person::create(Some(&person_name))?;

        // Assign faces to person
        for &idx in &cluster_indices {
            let face_id = faces[idx].id;
            Face::assign_to_person(face_id, person_id)?;
            total_assigned += 1;
        }
    }

    Ok(total_assigned)
}

fn region_query(faces: &[Face], point_idx: usize, epsilon: f32) -> Vec<usize> {
    let mut neighbors = Vec::new();
    let point_emb = faces[point_idx].embedding.as_ref().unwrap();

    for i in 0..faces.len() {
        if let Some(emb) = &faces[i].embedding {
            let dist = cosine_distance(point_emb, emb);

            if dist <= epsilon {
                neighbors.push(i);
            }
        }
    }

    neighbors
}

fn expand_cluster(
    faces: &[Face],
    labels: &mut Vec<i32>,
    point_idx: usize,
    neighbor_pts: &[usize],
    cluster_id: i32,
    epsilon: f32,
) {
    labels[point_idx] = cluster_id;

    let mut search_queue = neighbor_pts.to_vec();
    let mut i = 0;

    while i < search_queue.len() {
        let p_idx = search_queue[i];

        if labels[p_idx] == -1 {
            labels[p_idx] = cluster_id; // Change noise to border point
        } else if labels[p_idx] == 0 {
            labels[p_idx] = cluster_id;

            let p_neighbors = region_query(faces, p_idx, epsilon);
            if p_neighbors.len() >= MIN_SAMPLES {
                for &pn in &p_neighbors {
                    if !search_queue.contains(&pn) {
                        search_queue.push(pn);
                    }
                }
            }
        }

        i += 1;
    }
}
