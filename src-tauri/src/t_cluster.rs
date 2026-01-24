use crate::t_common;
use crate::t_sqlite::{Face, Person};
use rand::seq::SliceRandom;
use std::collections::HashMap;

/// Clustering progress information
#[derive(Clone, serde::Serialize)]
pub struct ClusterProgress {
    pub phase: String, // "graph", "iterate", "converged", "assign", "thumbnail"
    pub current: usize,
    pub total: usize,
}

/// Calculate cosine distance between two PRE-PARSED embeddings
/// Distance = 1.0 - Cosine Similarity
/// NOTE: Assumes input vectors are already normalized!
fn cosine_distance(emb1: &[f32], emb2: &[f32]) -> f32 {
    // Dot product of normalized vectors = cosine similarity
    let dot_product: f32 = emb1.iter().zip(emb2.iter()).map(|(x, y)| x * y).sum();

    // Clamp similarity to [-1.0, 1.0] to handle floating point errors
    let similarity = dot_product.clamp(-1.0, 1.0);
    1.0 - similarity
}

/// Helper: Parse raw byte embedding to normalized f32 vector
fn parse_embedding(bytes: &[u8]) -> Vec<f32> {
    let emb_vec: Vec<f32> = bytes
        .chunks(4)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();

    // Normalize
    let norm: f32 = emb_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        emb_vec.iter().map(|x| x / norm).collect()
    } else {
        emb_vec // Should not happen for valid embeddings
    }
}

/// Edge in the similarity graph
#[derive(Clone)]
struct Edge {
    to: usize,   // Target node index
    weight: f32, // Edge weight (similarity = 1 - distance)
}

/// Run Chinese Whispers clustering on ALL faces
///
/// Re-implemented with performance optimizations:
/// 1. Pre-parses all embeddings to avoid allocations in inner loop (O(n) memory alloc)
/// 2. Uses zero-allocation distance calculation
/// 3. Returns Chinese Whispers algorithm for better quality (less chaining)
pub fn cluster_faces<F, C>(
    threshold: f32,
    mut progress_fn: F,
    is_cancelled_fn: C,
) -> Result<usize, String>
where
    F: FnMut(ClusterProgress),
    C: Fn() -> bool,
{
    // 1. Reset all existing assignments and delete persons
    Face::reset_all_assignments()?;

    // 2. Get ALL faces for clustering
    let faces = Face::get_all()?;
    let n = faces.len();
    if n == 0 {
        return Ok(0);
    }

    // 3. Pre-parse embeddings (Optimize: do this once)
    let mut parsed_embeddings: Vec<Option<Vec<f32>>> = Vec::with_capacity(n);
    for face in &faces {
        if let Some(bytes) = &face.embedding {
            parsed_embeddings.push(Some(parse_embedding(bytes)));
        } else {
            parsed_embeddings.push(None);
        }
    }

    // 4. Build K-NN Graph
    // Step A: Collect all potential edges (candidates)
    let mut candidate_lists: Vec<Vec<(usize, f32)>> = vec![Vec::new(); n];
    let total_pairs = n * (n - 1) / 2;
    let mut pairs_done: usize = 0;
    let mut last_pct = 0;

    for i in 0..n {
        // Check for cancellation
        if is_cancelled_fn() {
            return Ok(0);
        }

        if let Some(emb_i) = &parsed_embeddings[i] {
            for j in (i + 1)..n {
                if let Some(emb_j) = &parsed_embeddings[j] {
                    let dist = cosine_distance(emb_i, emb_j);

                    if dist < threshold {
                        let weight = 1.0 - dist;
                        // Square weight to punish weak links further (optional but recommended)
                        let adjusted_weight = weight * weight;

                        candidate_lists[i].push((j, adjusted_weight));
                        candidate_lists[j].push((i, adjusted_weight));
                    }
                }
                pairs_done += 1;
            }
        } else {
            pairs_done += n - 1 - i;
        }

        // Report progress every 5%
        let current_pct = if total_pairs > 0 {
            pairs_done * 100 / total_pairs
        } else {
            100
        };
        if current_pct >= last_pct + 5 || pairs_done == total_pairs {
            progress_fn(ClusterProgress {
                phase: "graph".to_string(),
                current: current_pct,
                total: 100,
            });
            last_pct = current_pct;
        }
    }

    // Step B: Prune edges to Top-K (K-NN)
    const K_NEIGHBORS: usize = t_common::K_NEIGHBORS;
    let mut graph: Vec<Vec<Edge>> = vec![Vec::new(); n];

    for (i, candidates) in candidate_lists.iter_mut().enumerate() {
        // Sort by weight descending (strongest similarity first)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Keep only Top-K
        let count = std::cmp::min(candidates.len(), K_NEIGHBORS);

        for k in 0..count {
            let (target, weight) = candidates[k];
            graph[i].push(Edge { to: target, weight });
        }
    }

    // 5. Run Chinese Whispers Algorithm
    let mut labels: Vec<usize> = (0..n).collect();
    let mut order: Vec<usize> = (0..n).collect();
    let mut rng = rand::thread_rng();
    let max_iterations = 20;

    for iter in 0..max_iterations {
        // Check for cancellation
        if is_cancelled_fn() {
            return Ok(0);
        }

        let mut changed = false;

        progress_fn(ClusterProgress {
            phase: "iterate".to_string(),
            current: iter + 1,
            total: max_iterations,
        });

        order.shuffle(&mut rng);

        for &node in &order {
            if graph[node].is_empty() {
                continue;
            }

            // Count weighted votes
            let mut label_weights: HashMap<usize, f32> = HashMap::new();
            for edge in &graph[node] {
                let neighbor_label = labels[edge.to];
                *label_weights.entry(neighbor_label).or_insert(0.0) += edge.weight;
            }

            // Find best label
            let best_label = label_weights
                .into_iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .map(|(label, _)| label)
                .unwrap_or(labels[node]);

            if labels[node] != best_label {
                labels[node] = best_label;
                changed = true;
            }
        }

        if !changed {
            progress_fn(ClusterProgress {
                phase: "converged".to_string(),
                current: iter + 1,
                total: max_iterations,
            });
            break;
        }
    }

    // Check for cancellation before assignment
    if is_cancelled_fn() {
        return Ok(0);
    }

    // 6. Collect clusters
    let mut cluster_map: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, &label) in labels.iter().enumerate() {
        // if !graph[i].is_empty() {
        cluster_map.entry(label).or_default().push(i);
        // }
    }

    // 7. Filter clusters
    const MIN_SAMPLES: usize = t_common::MIN_SAMPLES;
    let valid_clusters: Vec<_> = cluster_map
        .into_iter()
        .filter(|(_, face_indices)| face_indices.len() >= MIN_SAMPLES)
        .collect();

    let total_clusters = valid_clusters.len();

    // 8. Assign faces to persons
    let mut total_assigned = 0;

    for (cluster_idx, (_, cluster_face_indices)) in valid_clusters.into_iter().enumerate() {
        if is_cancelled_fn() {
            return Ok(total_assigned);
        }

        progress_fn(ClusterProgress {
            phase: "assign".to_string(),
            current: cluster_idx + 1,
            total: total_clusters,
        });

        let person_name = format!("Person {}", cluster_idx + 1);
        let person_id = Person::create(Some(&person_name))?;

        for face_idx in cluster_face_indices {
            Face::assign_to_person(faces[face_idx].id, person_id)?;
            total_assigned += 1;
        }
    }

    // 9. Generate thumbnails
    progress_fn(ClusterProgress {
        phase: "thumbnail".to_string(),
        current: 0,
        total: total_clusters,
    });

    Person::update_all_thumbnails()?;

    progress_fn(ClusterProgress {
        phase: "thumbnail".to_string(),
        current: total_clusters,
        total: total_clusters,
    });

    Ok(total_assigned)
}
