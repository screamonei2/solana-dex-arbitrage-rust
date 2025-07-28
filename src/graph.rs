use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub weight: f64, // -ln(rate * (1-fee))
    pub dex: crate::dex::DexType,
    pub original_rate: f64,
    pub fee: f64,
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub vertices: Vec<String>, // Token mint addresses
    pub edges: Vec<Edge>,
    pub vertex_map: HashMap<String, usize>, // mint -> index
}

#[derive(Debug, Clone)]
pub struct ArbitrageCycle {
    pub path: Vec<usize>, // Vertex indices
    pub edges: Vec<Edge>,
    pub profit_factor: f64, // How much profit (1.05 = 5% profit)
    pub tokens: Vec<String>, // Token addresses in cycle
}

impl Graph {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            vertex_map: HashMap::new(),
        }
    }
    
    pub fn add_vertex(&mut self, token_mint: String) -> usize {
        if let Some(&index) = self.vertex_map.get(&token_mint) {
            return index;
        }
        
        let index = self.vertices.len();
        self.vertices.push(token_mint.clone());
        self.vertex_map.insert(token_mint, index);
        index
    }
    
    pub fn add_edge(&mut self, from_token: &str, to_token: &str, rate: f64, fee: f64, dex: crate::dex::DexType) {
        let from_idx = self.add_vertex(from_token.to_string());
        let to_idx = self.add_vertex(to_token.to_string());
        
        // Weight = -ln(rate * (1-fee)) for Bellman-Ford negative cycle detection
        let effective_rate = rate * (1.0 - fee);
        let weight = if effective_rate > 0.0 {
            -effective_rate.ln()
        } else {
            f64::INFINITY
        };
        
        self.edges.push(Edge {
            from: from_idx,
            to: to_idx,
            weight,
            dex,
            original_rate: rate,
            fee,
        });
    }
    
    /// Find arbitrage opportunities using Bellman-Ford algorithm
    pub fn find_arbitrage_cycles(&self) -> Vec<ArbitrageCycle> {
        let n = self.vertices.len();
        if n == 0 {
            return Vec::new();
        }
        
        let mut cycles = Vec::new();
        
        // Run Bellman-Ford from each vertex to find all negative cycles
        for start in 0..n {
            if let Some(cycle) = self.bellman_ford_negative_cycle(start) {
                cycles.push(cycle);
            }
        }
        
        // Remove duplicate cycles
        self.deduplicate_cycles(cycles)
    }
    
    fn bellman_ford_negative_cycle(&self, start: usize) -> Option<ArbitrageCycle> {
        let n = self.vertices.len();
        let mut dist = vec![f64::INFINITY; n];
        let mut parent = vec![None; n];
        
        dist[start] = 0.0;
        
        // Relax edges n-1 times
        for _ in 0..(n - 1) {
            for edge in &self.edges {
                if dist[edge.from] != f64::INFINITY {
                    let new_dist = dist[edge.from] + edge.weight;
                    if new_dist < dist[edge.to] {
                        dist[edge.to] = new_dist;
                        parent[edge.to] = Some(edge.from);
                    }
                }
            }
        }
        
        // Check for negative cycles
        for edge in &self.edges {
            if dist[edge.from] != f64::INFINITY {
                let new_dist = dist[edge.from] + edge.weight;
                if new_dist < dist[edge.to] {
                    // Found negative cycle, reconstruct it
                    return self.reconstruct_cycle(edge.to, &parent);
                }
            }
        }
        
        None
    }
    
    fn reconstruct_cycle(&self, start: usize, parent: &[Option<usize>]) -> Option<ArbitrageCycle> {
        let mut visited = vec![false; self.vertices.len()];
        let mut current = start;
        
        // Find a vertex that's part of the cycle
        for _ in 0..self.vertices.len() {
            if let Some(p) = parent[current] {
                current = p;
            } else {
                return None;
            }
        }
        
        // Now current is definitely in the cycle
        let cycle_start = current;
        let mut path = Vec::new();
        let mut cycle_edges = Vec::new();
        
        loop {
            path.push(current);
            if let Some(next) = parent[current] {
                // Find the edge from next to current
                if let Some(edge) = self.edges.iter().find(|e| e.from == next && e.to == current) {
                    cycle_edges.push(edge.clone());
                }
                current = next;
                if current == cycle_start && path.len() > 1 {
                    break;
                }
            } else {
                return None;
            }
        }
        
        // Calculate profit factor
        let total_weight: f64 = cycle_edges.iter().map(|e| e.weight).sum();
        let profit_factor = (-total_weight).exp();
        
        // Only return profitable cycles
        if profit_factor > 1.0 {
            let tokens = path.iter().map(|&i| self.vertices[i].clone()).collect();
            
            Some(ArbitrageCycle {
                path,
                edges: cycle_edges,
                profit_factor,
                tokens,
            })
        } else {
            None
        }
    }
    
    fn deduplicate_cycles(&self, cycles: Vec<ArbitrageCycle>) -> Vec<ArbitrageCycle> {
        let mut unique_cycles = Vec::new();
        
        for cycle in cycles {
            // Check if this cycle is already in the list (considering rotations)
            let is_duplicate = unique_cycles.iter().any(|existing| {
                self.cycles_are_equivalent(&cycle, existing)
            });
            
            if !is_duplicate {
                unique_cycles.push(cycle);
            }
        }
        
        // Sort by profit factor (descending)
        unique_cycles.sort_by(|a, b| b.profit_factor.partial_cmp(&a.profit_factor).unwrap());
        
        unique_cycles
    }
    
    fn cycles_are_equivalent(&self, cycle1: &ArbitrageCycle, cycle2: &ArbitrageCycle) -> bool {
        if cycle1.path.len() != cycle2.path.len() {
            return false;
        }
        
        // Check if cycle2 is a rotation of cycle1
        let path1 = &cycle1.path;
        let path2 = &cycle2.path;
        
        for start in 0..path2.len() {
            let mut matches = true;
            for i in 0..path1.len() {
                if path1[i] != path2[(start + i) % path2.len()] {
                    matches = false;
                    break;
                }
            }
            if matches {
                return true;
            }
        }
        
        false
    }
    
    pub fn update_prices(&mut self, prices: &HashMap<String, HashMap<crate::dex::DexType, crate::monitoring::PriceData>>) {
        // Clear existing edges
        self.edges.clear();
        
        // Add edges based on current prices
        for (pair, dex_prices) in prices {
            let tokens: Vec<&str> = pair.split('/').collect();
            if tokens.len() != 2 {
                continue;
            }
            
            let token_a = tokens[0];
            let token_b = tokens[1];
            
            for (dex, price_data) in dex_prices {
                // Add edge A -> B
                self.add_edge(
                    token_a,
                    token_b,
                    price_data.price,
                    self.get_dex_fee(*dex),
                    *dex,
                );
                
                // Add reverse edge B -> A
                self.add_edge(
                    token_b,
                    token_a,
                    1.0 / price_data.price,
                    self.get_dex_fee(*dex),
                    *dex,
                );
            }
        }
    }
    
    fn get_dex_fee(&self, dex: crate::dex::DexType) -> f64 {
        match dex {
            crate::dex::DexType::Raydium => 0.0025,
            crate::dex::DexType::Orca => 0.002,
            crate::dex::DexType::Jupiter => 0.002,
            crate::dex::DexType::Meteora => 0.001,
            _ => 0.003,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dex::DexType;
    
    #[test]
    fn test_graph_creation() {
        let mut graph = Graph::new();
        
        // Add some test edges: SOL -> BONK -> USDC -> SOL
        graph.add_edge("SOL", "BONK", 1000000.0, 0.0025, DexType::Raydium);
        graph.add_edge("BONK", "USDC", 0.00002, 0.002, DexType::Orca);
        graph.add_edge("USDC", "SOL", 0.02, 0.003, DexType::Jupiter);
        
        assert_eq!(graph.vertices.len(), 3);
        assert_eq!(graph.edges.len(), 3);
    }
    
    #[test]
    fn test_arbitrage_detection() {
        let mut graph = Graph::new();
        
        // Create a profitable cycle
        graph.add_edge("SOL", "BONK", 1000000.0, 0.001, DexType::Raydium);
        graph.add_edge("BONK", "USDC", 0.00003, 0.001, DexType::Orca);
        graph.add_edge("USDC", "SOL", 0.025, 0.001, DexType::Jupiter); // Higher rate for profit
        
        let cycles = graph.find_arbitrage_cycles();
        
        // Should find at least one profitable cycle
        assert!(!cycles.is_empty());
        if let Some(cycle) = cycles.first() {
            assert!(cycle.profit_factor > 1.0);
        }
    }
} 