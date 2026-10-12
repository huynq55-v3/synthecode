use super::signature::BehavioralSignature;

/// High-dimensional Novelty Archive maintaining behavioral diversity (MAP-Elites / Novelty Search).
#[derive(Debug, Clone)]
pub struct NoveltyArchive {
    /// Stored items: (problem_id, signature)
    entries: Vec<(usize, BehavioralSignature)>,
    /// Minimum distance to the nearest neighbor required to admit a new problem into the archive.
    pub novelty_threshold: f64,
}

impl Default for NoveltyArchive {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            // Require a significant behavioral divergence of at least 0.12
            novelty_threshold: 0.12,
        }
    }
}

impl NoveltyArchive {
    /// Create a new NoveltyArchive with custom distance threshold.
    pub const fn new(novelty_threshold: f64) -> Self {
        Self {
            entries: Vec::new(),
            novelty_threshold,
        }
    }

    /// Computes the distance to the nearest existing neighbor in behavioral space.
    pub fn nearest_distance(&self, candidate: &BehavioralSignature) -> f64 {
        if self.entries.is_empty() {
            return 1.0;
        }

        self.entries
            .iter()
            .map(|(_, sig)| candidate.distance(sig))
            .fold(f64::INFINITY, f64::min)
    }

    /// Checks if a candidate signature is behaviorally novel relative to all archived problems.
    pub fn is_novel(&self, candidate: &BehavioralSignature) -> (bool, f64) {
        let min_dist = self.nearest_distance(candidate);
        let novel = min_dist >= self.novelty_threshold;
        (novel, min_dist)
    }

    /// Inserts a problem into the archive if it meets the novelty threshold.
    /// Returns true if admitted, false if rejected due to behavioral duplication.
    pub fn insert(&mut self, problem_id: usize, signature: BehavioralSignature) -> bool {
        let (novel, _) = self.is_novel(&signature);
        if novel {
            self.entries.push((problem_id, signature));
            true
        } else {
            false
        }
    }

    /// Total number of diverse problems currently archived.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if archive is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_novelty_archive_admission_and_deduplication() {
        let mut archive = NoveltyArchive::new(0.2);

        let sig1 = BehavioralSignature { vector: vec![0.1, 0.2, 0.3] };
        let sig2 = BehavioralSignature { vector: vec![0.1, 0.2, 0.31] }; // Very close
        let sig3 = BehavioralSignature { vector: vec![0.8, 0.9, 0.95] }; // Very distant

        assert!(archive.insert(1, sig1));
        assert_eq!(archive.len(), 1);

        // sig2 is within threshold, must be rejected as duplicate motif
        assert!(!archive.insert(2, sig2));
        assert_eq!(archive.len(), 1);

        // sig3 is distant, must be admitted
        assert!(archive.insert(3, sig3));
        assert_eq!(archive.len(), 2);
    }
}
