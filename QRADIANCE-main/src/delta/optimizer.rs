//! Evolutionäre Delta-Gradient-Optimierung
//!
//! Optimiert QRIK-Parameter zur Minimierung von ∇Ψ_Δ

use crate::delta::kernel::{DeltaKernel, QRIKParams};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Evolutionärer Optimierer
#[derive(Debug, Clone)]
pub struct EvolutionaryOptimizer {
    /// Populationsgröße
    pub population_size: usize,
    /// Maximale Generationen
    pub max_generations: usize,
    /// Mutations-Standardabweichung
    pub mutation_std: f64,
    /// Elitismus-Anteil (Top-N bleiben erhalten)
    pub elite_fraction: f64,
    /// Simulationszeit pro Individual
    pub simulation_time: f64,
}

impl EvolutionaryOptimizer {
    /// Erstellt neuen Optimierer
    pub fn new(
        population_size: usize,
        max_generations: usize,
        mutation_std: f64,
        elite_fraction: f64,
    ) -> Self {
        Self {
            population_size,
            max_generations,
            mutation_std,
            elite_fraction,
            simulation_time: 10.0,
        }
    }

    /// Optimiert QRIK-Parameter
    ///
    /// Returns: (beste_params, beste_fitness)
    pub fn optimize(&self, initial_params: QRIKParams) -> (QRIKParams, f64) {
        let mut population = self.initialize_population(initial_params);

        let mut best_params = initial_params;
        let mut best_fitness = f64::NEG_INFINITY;

        for generation in 0..self.max_generations {
            // Evaluate fitness
            let fitnesses = self.evaluate_population(&population);

            // Track best
            for (i, &fitness) in fitnesses.iter().enumerate() {
                if fitness > best_fitness {
                    best_fitness = fitness;
                    best_params = population[i];
                }
            }

            if generation % 10 == 0 {
                println!(
                    "Generation {}: Best fitness = {:.6}",
                    generation, best_fitness
                );
            }

            // Selection
            let elite_count = (self.population_size as f64 * self.elite_fraction) as usize;
            let selected = self.select_elite(&population, &fitnesses, elite_count);

            // Create next generation
            population = self.create_next_generation(&selected);
        }

        (best_params, best_fitness)
    }

    /// Initialisiert Population mit zufälligen Variationen
    fn initialize_population(&self, base_params: QRIKParams) -> Vec<QRIKParams> {
        let mut rng = rand::thread_rng();
        let mut population = Vec::with_capacity(self.population_size);

        // Erste Individual ist die Basis
        population.push(base_params);

        // Rest sind Variationen
        for _ in 1..self.population_size {
            let params = QRIKParams {
                hopping_strength: (base_params.hopping_strength + rng.gen_range(-0.5..0.5))
                    .max(0.1),
                base_frequency: base_params.base_frequency + rng.gen_range(-0.5..0.5),
                coupling_strength: (base_params.coupling_strength + rng.gen_range(-1.0..1.0))
                    .max(0.1),
                spectrum_size: base_params.spectrum_size,
                learning_rate: (base_params.learning_rate + rng.gen_range(-0.005..0.005))
                    .clamp(0.001, 0.1),
                epsilon_res: (base_params.epsilon_res + rng.gen_range(-0.1..0.1)).clamp(0.1, 0.9),
                lambda_flood: (base_params.lambda_flood + rng.gen_range(-0.5..0.5)).max(0.1),
            };
            population.push(params);
        }

        population
    }

    /// Evaluiert Fitness aller Individuals
    fn evaluate_population(&self, population: &[QRIKParams]) -> Vec<f64> {
        use rayon::prelude::*;

        population
            .par_iter()
            .map(|params| self.evaluate_individual(*params))
            .collect()
    }

    /// Evaluiert einzelnes Individual
    fn evaluate_individual(&self, params: QRIKParams) -> f64 {
        let mut kernel = DeltaKernel::new(params);

        // Simuliere System
        let dt = 0.01;
        let steps = (self.simulation_time / dt) as usize;

        let mut total_coherence = 0.0;
        let mut total_gradient = 0.0;

        for _ in 0..steps {
            kernel.evolve(dt);

            total_coherence += kernel.coherence();
            total_gradient += kernel.coherence_gradient();
        }

        let avg_coherence = total_coherence / steps as f64;
        let avg_gradient = total_gradient / steps as f64;

        // Fitness: Maximiere Kohärenz, minimiere Gradient
        avg_coherence - avg_gradient
    }

    /// Selektiert Elite-Individuals
    fn select_elite(
        &self,
        population: &[QRIKParams],
        fitnesses: &[f64],
        count: usize,
    ) -> Vec<QRIKParams> {
        let mut indexed: Vec<(usize, f64)> = fitnesses.iter().copied().enumerate().collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        indexed
            .iter()
            .take(count)
            .map(|&(idx, _)| population[idx])
            .collect()
    }

    /// Erstellt nächste Generation via Mutation
    fn create_next_generation(&self, elite: &[QRIKParams]) -> Vec<QRIKParams> {
        let mut rng = rand::thread_rng();
        let mut next_gen = Vec::with_capacity(self.population_size);

        // Elite bleibt erhalten
        next_gen.extend_from_slice(elite);

        // Fülle Rest mit mutierten Elite-Individuals auf
        while next_gen.len() < self.population_size {
            let parent_idx = rng.gen_range(0..elite.len());
            let parent = elite[parent_idx];

            let mutated = self.mutate(parent);
            next_gen.push(mutated);
        }

        next_gen
    }

    /// Mutiert Parameter
    fn mutate(&self, params: QRIKParams) -> QRIKParams {
        use rand_distr::{Distribution, Normal};
        let mut rng = rand::thread_rng();

        let normal = Normal::new(0.0, self.mutation_std).unwrap();

        QRIKParams {
            hopping_strength: (params.hopping_strength + normal.sample(&mut rng)).max(0.1),
            base_frequency: params.base_frequency + normal.sample(&mut rng),
            coupling_strength: (params.coupling_strength + normal.sample(&mut rng) * 2.0).max(0.1),
            spectrum_size: params.spectrum_size,
            learning_rate: (params.learning_rate + normal.sample(&mut rng) * 0.01)
                .clamp(0.001, 0.1),
            epsilon_res: (params.epsilon_res + normal.sample(&mut rng) * 0.1).clamp(0.1, 0.9),
            lambda_flood: (params.lambda_flood + normal.sample(&mut rng) * 0.5).max(0.1),
        }
    }
}

impl Default for EvolutionaryOptimizer {
    fn default() -> Self {
        Self::new(20, 50, 0.1, 0.2)
    }
}

/// Optimierungs-Ergebnis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// Beste gefundene Parameter
    pub best_params: QRIKParams,
    /// Beste Fitness
    pub best_fitness: f64,
    /// Anzahl Generationen
    pub generations: usize,
    /// Fitness-Historie
    pub fitness_history: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let optimizer = EvolutionaryOptimizer::default();
        assert_eq!(optimizer.population_size, 20);
        assert_eq!(optimizer.max_generations, 50);
    }

    #[test]
    fn test_population_initialization() {
        let optimizer = EvolutionaryOptimizer::default();
        let params = QRIKParams::default();

        let population = optimizer.initialize_population(params);
        assert_eq!(population.len(), optimizer.population_size);

        // Erste sollte Basis-Params sein
        assert_eq!(population[0].hopping_strength, params.hopping_strength);
    }

    #[test]
    fn test_individual_evaluation() {
        let optimizer = EvolutionaryOptimizer::default();
        let params = QRIKParams::default();

        let fitness = optimizer.evaluate_individual(params);
        assert!(fitness.is_finite());
    }

    #[test]
    fn test_mutation() {
        let optimizer = EvolutionaryOptimizer::default();
        let params = QRIKParams::default();

        let mutated = optimizer.mutate(params);

        // Parameter sollten sich geändert haben (mit hoher Wahrscheinlichkeit)
        // Aber innerhalb vernünftiger Grenzen bleiben
        assert!(mutated.hopping_strength > 0.0);
        assert!(mutated.coupling_strength > 0.0);
        assert!(mutated.learning_rate > 0.0);
    }

    #[test]
    #[ignore] // Langsam, nur bei Bedarf ausführen
    fn test_optimization_run() {
        let optimizer = EvolutionaryOptimizer::new(10, 5, 0.1, 0.2);
        let initial_params = QRIKParams::default();

        let (_best_params, best_fitness) = optimizer.optimize(initial_params);

        println!("Best fitness: {}", best_fitness);
        assert!(best_fitness.is_finite());
    }
}
