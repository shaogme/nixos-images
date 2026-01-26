use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

pub const MAX_RELEASES: usize = 7;
pub const MIN_INTERVAL_DAYS: i64 = 7;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReleaseRecord {
    pub tag_name: String,
    pub created_at: DateTime<Utc>,
    pub release_id: u64,
}

#[derive(Debug)]
pub struct ReleaseManager {
    releases: Vec<ReleaseRecord>,
}

impl ReleaseManager {
    pub fn new(mut releases: Vec<ReleaseRecord>) -> Self {
        releases.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Self { releases }
    }

    pub fn get_releases(&self) -> &[ReleaseRecord] {
        &self.releases
    }

    /// Pushes a new release and returns a list of releases that should be deleted
    /// according to the retention policy.
    pub fn push_release(&mut self, new_release: ReleaseRecord) -> Vec<ReleaseRecord> {
        // Ensure the list is sorted by date before processing
        self.releases
            .sort_by(|a, b| a.created_at.cmp(&b.created_at));

        let mut kept = Vec::new();
        let mut to_delete = Vec::new();

        // 1. Filter existing releases based on minimum interval
        // We use std::mem::take to move ownership of all items out of self.releases
        // This allows us to reassign self.releases later without conflict
        let all_releases = std::mem::take(&mut self.releases);
        let mut iter = all_releases.into_iter();

        if let Some(first) = iter.next() {
            kept.push(first);

            for candidate in iter {
                let last_kept = kept.last().unwrap();
                let diff = candidate
                    .created_at
                    .signed_duration_since(last_kept.created_at);

                if diff < Duration::days(MIN_INTERVAL_DAYS) {
                    to_delete.push(candidate);
                } else {
                    kept.push(candidate);
                }
            }
        }

        self.releases = kept;

        // 2. Enforce MAX_RELEASES limit
        // ensure we have room for the new release
        while self.releases.len() >= MAX_RELEASES {
            let oldest = self.releases.remove(0);
            to_delete.push(oldest);
        }

        // 3. Add the new release
        self.releases.push(new_release);
        // Ensure final list is sorted
        self.releases
            .sort_by(|a, b| a.created_at.cmp(&b.created_at));

        to_delete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_release(days_ago: i64, id: u64) -> ReleaseRecord {
        ReleaseRecord {
            tag_name: format!("release-{}", id),
            created_at: Utc::now() - Duration::days(days_ago),
            release_id: id,
        }
    }

    #[test]
    fn test_push_normal_interval() {
        let r1 = create_release(20, 1);
        let r2 = create_release(10, 2);

        let mut manager = ReleaseManager::new(vec![r1.clone(), r2.clone()]);

        let new_release = create_release(0, 3);
        let deleted = manager.push_release(new_release);

        assert_eq!(deleted.len(), 0);
        assert_eq!(manager.releases.len(), 3);
    }

    #[test]
    fn test_cleanup_sparse_releases() {
        // R1 (20 days ago), R2 (19 days ago), R3 (10 days ago)
        // R2 is < 7 days from R1. Should be deleted.
        let r1 = create_release(20, 1);
        let r2 = create_release(19, 2);
        let r3 = create_release(10, 3);

        let mut manager = ReleaseManager::new(vec![r1, r2.clone(), r3]);

        // Push R4 (0 days ago)
        let r4 = create_release(0, 4);
        let deleted = manager.push_release(r4);

        assert_eq!(deleted.len(), 1);
        assert_eq!(deleted[0].release_id, 2); // R2 deleted
        assert_eq!(manager.releases.len(), 3); // R1, R3, R4
    }

    #[test]
    fn test_max_releases_limit() {
        // Create 7 releases with proper intervals (8 days)
        // R1: 56d, R2: 48d, ... R7: 8d
        let mut releases = Vec::new();
        for i in 0..7 {
            releases.push(create_release(56 - (i as i64 * 8), i as u64));
        }

        let mut manager = ReleaseManager::new(releases);
        assert_eq!(manager.releases.len(), 7);

        // Push new release
        let new_release = create_release(0, 100);
        let deleted = manager.push_release(new_release);

        assert_eq!(deleted.len(), 1);
        assert_eq!(deleted[0].release_id, 0); // Earliest (R1) should be deleted
        assert_eq!(manager.releases.len(), 7); // Still 7
    }

    #[test]
    fn test_complex_cleanup_and_limit() {
        // Initial:
        // A (30d), B (29d), C (25d), D (15d)
        // B is too close to A.
        // C is too close to A? 30-25 = 5 < 7. Yes.
        // After A, next valid is D (30-15=15 > 7).
        // So B and C should be deleted.

        let a = create_release(30, 1);
        let b = create_release(29, 2);
        let c = create_release(25, 3);
        let d = create_release(15, 4);

        let mut manager = ReleaseManager::new(vec![a, b.clone(), c.clone(), d]);

        let e = create_release(0, 5);
        let deleted = manager.push_release(e);

        assert_eq!(deleted.len(), 2);
        // Deleted should be B and C
        let deleted_ids: Vec<u64> = deleted.iter().map(|r| r.release_id).collect();
        assert!(deleted_ids.contains(&2));
        assert!(deleted_ids.contains(&3));

        assert_eq!(manager.releases.len(), 3); // A, D, E
    }
}
