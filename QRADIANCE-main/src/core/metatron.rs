//! Metatron-Graph-Topologie
//!
//! 13-Knoten, 78-Kanten Struktur basierend auf dem Metatron's Cube
//! Geometrie: 1 Zentrum + 6 Hexagon + 6 Cube

use nalgebra::SMatrix;
use serde::{Deserialize, Serialize};

/// Anzahl der Knoten im Metatron-Graph
pub const NUM_NODES: usize = 13;

/// Anzahl der Kanten im Metatron-Graph
/// Berechnung: 6 (Zentrum-Hexagon) + 6 (Zentrum-Cube) + 6 (Hexagon-Ring)
///            + 12 (Hexagon-Cube) + 12 (Cube-Kanten) + 4 (Cube-Diagonalen) = 46
pub const NUM_EDGES: usize = 46;

/// Metatron-Graph-Struktur
///
/// Topologie:
/// - v1: Zentrum
/// - v2..v7: Hexagonaler Ring
/// - v8..v13: Kubische Eckpunkte
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetatronGraph {
    /// Adjazenzmatrix A ∈ {0,1}^{13×13}
    pub adjacency: SMatrix<bool, NUM_NODES, NUM_NODES>,
    /// Graph-Laplacian L = D - A
    pub laplacian: SMatrix<f64, NUM_NODES, NUM_NODES>,
    /// Gradmatrix D (Diagonalmatrix mit Knotengraden)
    pub degree_matrix: SMatrix<f64, NUM_NODES, NUM_NODES>,
}

impl MetatronGraph {
    /// Erstellt den kanonischen Metatron-Graph
    pub fn new() -> Self {
        let mut adjacency = SMatrix::<bool, NUM_NODES, NUM_NODES>::from_element(false);

        // Zentrum (v0) verbunden mit Hexagon (v1..v6)
        for i in 1..=6 {
            adjacency[(0, i)] = true;
            adjacency[(i, 0)] = true;
        }

        // Zentrum (v0) verbunden mit Cube (v7..v12)
        for i in 7..=12 {
            adjacency[(0, i)] = true;
            adjacency[(i, 0)] = true;
        }

        // Hexagon (geschlossener Ring): v1-v2-v3-v4-v5-v6-v1
        for i in 1..=6 {
            let next = if i == 6 { 1 } else { i + 1 };
            adjacency[(i, next)] = true;
            adjacency[(next, i)] = true;
        }

        // Hexagon zu Cube (Kreuzverbindungen)
        let hex_cube_connections = [
            (1, 7),
            (1, 8),
            (2, 8),
            (2, 9),
            (3, 9),
            (3, 10),
            (4, 10),
            (4, 11),
            (5, 11),
            (5, 12),
            (6, 12),
            (6, 7),
        ];

        for &(i, j) in &hex_cube_connections {
            adjacency[(i, j)] = true;
            adjacency[(j, i)] = true;
        }

        // Cube-Kanten (12 Kanten des Würfels)
        let cube_edges = [
            (7, 8),
            (8, 9),
            (9, 10),
            (10, 11),
            (11, 12),
            (12, 7), // Unterer Ring
            (7, 9),
            (8, 10),
            (9, 11),
            (10, 12),
            (11, 7),
            (12, 8), // Kreuzdiagonalen
        ];

        for &(i, j) in &cube_edges {
            adjacency[(i, j)] = true;
            adjacency[(j, i)] = true;
        }

        // Cube-Raumdiagonalen (4 Diagonalen)
        let space_diagonals = [(7, 10), (8, 11), (9, 12), (7, 11)];

        for &(i, j) in &space_diagonals {
            adjacency[(i, j)] = true;
            adjacency[(j, i)] = true;
        }

        // Berechne Gradmatrix
        let mut degree_matrix = SMatrix::<f64, NUM_NODES, NUM_NODES>::zeros();
        for i in 0..NUM_NODES {
            let degree = adjacency.row(i).iter().filter(|&&x| x).count();
            degree_matrix[(i, i)] = degree as f64;
        }

        // Berechne Laplacian L = D - A
        let adjacency_f64 = adjacency.map(|x| if x { 1.0 } else { 0.0 });
        let laplacian = degree_matrix - adjacency_f64;

        Self {
            adjacency,
            laplacian,
            degree_matrix,
        }
    }

    /// Gibt die Nachbarn eines Knotens zurück
    pub fn neighbors(&self, node: usize) -> Vec<usize> {
        assert!(node < NUM_NODES);
        (0..NUM_NODES)
            .filter(|&j| self.adjacency[(node, j)])
            .collect()
    }

    /// Gibt den Grad eines Knotens zurück
    pub fn degree(&self, node: usize) -> usize {
        assert!(node < NUM_NODES);
        self.degree_matrix[(node, node)] as usize
    }

    /// Überprüft ob eine Kante existiert
    pub fn has_edge(&self, from: usize, to: usize) -> bool {
        assert!(from < NUM_NODES && to < NUM_NODES);
        self.adjacency[(from, to)]
    }

    /// Zählt die Gesamtanzahl der Kanten
    pub fn count_edges(&self) -> usize {
        let mut count = 0;
        for i in 0..NUM_NODES {
            for j in (i + 1)..NUM_NODES {
                if self.adjacency[(i, j)] {
                    count += 1;
                }
            }
        }
        count
    }

    /// Berechnet algebraische Konnektivität (λ₂, zweiter Eigenwert)
    ///
    /// Dies ist der Fiedler-Wert und misst die "Verbundenheit" des Graphen
    pub fn algebraic_connectivity(&self) -> f64 {
        use nalgebra::SymmetricEigen;

        let laplacian_symmetric = self.laplacian;
        let eigen = SymmetricEigen::new(laplacian_symmetric);
        let eigenvalues = eigen.eigenvalues;

        // Sortiere Eigenwerte aufsteigend
        let mut sorted_eigenvalues: Vec<f64> = eigenvalues.iter().copied().collect();
        sorted_eigenvalues.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // λ₂ ist der zweite Eigenwert (erster ist ≈0)
        sorted_eigenvalues[1]
    }

    /// Berechnet spektrale Lücke Δλ = λ₂ - λ₁
    pub fn spectral_gap(&self) -> f64 {
        use nalgebra::SymmetricEigen;

        let eigen = SymmetricEigen::new(self.laplacian);
        let eigenvalues = eigen.eigenvalues;

        let mut sorted: Vec<f64> = eigenvalues.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        sorted[1] - sorted[0]
    }

    /// Gibt die Knotentypen zurück
    pub fn node_type(&self, node: usize) -> NodeType {
        match node {
            0 => NodeType::Center,
            1..=6 => NodeType::Hexagon,
            7..=12 => NodeType::Cube,
            _ => panic!("Invalid node index"),
        }
    }
}

impl Default for MetatronGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Knotentypen im Metatron-Graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Zentraler Knoten (v0)
    Center,
    /// Hexagonaler Ring (v1..v6)
    Hexagon,
    /// Kubische Eckpunkte (v7..v12)
    Cube,
}

impl std::fmt::Display for MetatronGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Metatron-Graph:")?;
        writeln!(f, "  Knoten: {}", NUM_NODES)?;
        writeln!(f, "  Kanten: {}", self.count_edges())?;
        writeln!(
            f,
            "  Algebraische Konnektivität: {:.4}",
            self.algebraic_connectivity()
        )?;
        writeln!(f, "\nKnotengrade:")?;
        for i in 0..NUM_NODES {
            writeln!(
                f,
                "  v{:2} ({:8}): Grad {}",
                i,
                format!("{:?}", self.node_type(i)),
                self.degree(i)
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metatron_creation() {
        let graph = MetatronGraph::new();
        assert_eq!(graph.count_edges(), NUM_EDGES);
    }

    #[test]
    fn test_center_connections() {
        let graph = MetatronGraph::new();
        // Zentrum sollte mit allen anderen Knoten verbunden sein
        assert_eq!(graph.degree(0), 12);
    }

    #[test]
    fn test_symmetry() {
        let graph = MetatronGraph::new();
        // Adjazenzmatrix sollte symmetrisch sein
        for i in 0..NUM_NODES {
            for j in 0..NUM_NODES {
                assert_eq!(graph.adjacency[(i, j)], graph.adjacency[(j, i)]);
            }
        }
    }

    #[test]
    fn test_laplacian_properties() {
        let graph = MetatronGraph::new();

        // Laplacian sollte Zeilensumme 0 haben
        for i in 0..NUM_NODES {
            let row_sum: f64 = graph.laplacian.row(i).iter().sum();
            assert!((row_sum).abs() < 1e-10);
        }
    }

    #[test]
    fn test_algebraic_connectivity() {
        let graph = MetatronGraph::new();
        let lambda2 = graph.algebraic_connectivity();

        // λ₂ sollte > 0 sein für verbundenen Graphen
        assert!(lambda2 > 0.0);
    }

    #[test]
    fn test_neighbors() {
        let graph = MetatronGraph::new();

        // Zentrum hat 12 Nachbarn
        let center_neighbors = graph.neighbors(0);
        assert_eq!(center_neighbors.len(), 12);

        // Nachbarn sollten 1..12 sein
        for i in 1..=12 {
            assert!(center_neighbors.contains(&i));
        }
    }

    #[test]
    fn test_hexagon_ring() {
        let graph = MetatronGraph::new();

        // Hexagon sollte geschlossen sein
        for i in 1..=6 {
            let next = if i == 6 { 1 } else { i + 1 };
            assert!(graph.has_edge(i, next));
        }
    }

    #[test]
    fn test_node_types() {
        let graph = MetatronGraph::new();

        assert_eq!(graph.node_type(0), NodeType::Center);
        assert_eq!(graph.node_type(3), NodeType::Hexagon);
        assert_eq!(graph.node_type(10), NodeType::Cube);
    }
}
