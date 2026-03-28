//! Voice management: priority-based polyphony with voice stealing.
//!
//! Manages a fixed pool of synthesizer "voices" (active sound instances).
//! When the pool is full and a new sound is requested, the lowest-priority
//! or oldest voice is stolen.
//!
//! This module is synthesizer-agnostic — it tracks voice metadata (priority,
//! age, active state) and provides slot indices. The caller manages the
//! actual synthesizer instances externally using the returned slot indices.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// How to select which voice to steal when the pool is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StealPolicy {
    /// Steal the oldest active voice.
    Oldest,
    /// Steal the lowest-priority voice (ties broken by age).
    LowestPriority,
    /// Do not steal — reject the new voice if pool is full.
    None,
}

/// Metadata for a single voice slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSlot {
    /// Whether this slot is currently active.
    pub active: bool,
    /// Priority (higher = more important, less likely to be stolen).
    pub priority: u8,
    /// Age in ticks since activation (incremented by `tick()`).
    pub age: u64,
    /// User-defined tag for identifying what sound this voice plays.
    pub tag: u32,
}

/// Voice pool manager — tracks active voices with priority-based stealing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicePool {
    slots: Vec<VoiceSlot>,
    steal_policy: StealPolicy,
    max_voices: usize,
}

impl VoicePool {
    /// Creates a new voice pool with the given capacity and steal policy.
    #[must_use]
    pub fn new(max_voices: usize, steal_policy: StealPolicy) -> Self {
        let max_voices = max_voices.max(1);
        let slots = (0..max_voices)
            .map(|_| VoiceSlot {
                active: false,
                priority: 0,
                age: 0,
                tag: 0,
            })
            .collect();
        Self {
            slots,
            steal_policy,
            max_voices,
        }
    }

    /// Returns the maximum number of voices.
    #[inline]
    #[must_use]
    pub fn max_voices(&self) -> usize {
        self.max_voices
    }

    /// Returns the number of currently active voices.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|s| s.active).count()
    }

    /// Requests a voice slot. Returns the slot index if successful.
    ///
    /// `priority`: higher values are harder to steal (0 = lowest, 255 = highest).
    /// `tag`: user-defined identifier for the sound.
    ///
    /// Returns `Some(index)` if a slot was allocated (either free or stolen),
    /// or `None` if the pool is full and the steal policy is `None`.
    pub fn allocate(&mut self, priority: u8, tag: u32) -> Option<usize> {
        // First: look for an inactive slot
        if let Some(idx) = self.slots.iter().position(|s| !s.active) {
            self.slots[idx] = VoiceSlot {
                active: true,
                priority,
                age: 0,
                tag,
            };
            return Some(idx);
        }

        // Pool is full — try to steal
        match self.steal_policy {
            StealPolicy::None => None,
            StealPolicy::Oldest => {
                let idx = self
                    .slots
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, s)| s.age)
                    .map(|(i, _)| i)?;
                self.slots[idx] = VoiceSlot {
                    active: true,
                    priority,
                    age: 0,
                    tag,
                };
                Some(idx)
            }
            StealPolicy::LowestPriority => {
                // Find lowest priority; break ties by oldest
                let idx = self
                    .slots
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| a.priority.cmp(&b.priority).then(b.age.cmp(&a.age)))
                    .map(|(i, _)| i)?;
                // Only steal if the new voice has equal or higher priority
                if priority >= self.slots[idx].priority {
                    self.slots[idx] = VoiceSlot {
                        active: true,
                        priority,
                        age: 0,
                        tag,
                    };
                    Some(idx)
                } else {
                    None
                }
            }
        }
    }

    /// Releases a voice slot, making it available for reuse.
    pub fn release(&mut self, index: usize) {
        if let Some(slot) = self.slots.get_mut(index) {
            slot.active = false;
            slot.age = 0;
        }
    }

    /// Advances the age counter for all active voices. Call once per audio block.
    pub fn tick(&mut self) {
        for slot in &mut self.slots {
            if slot.active {
                slot.age = slot.age.saturating_add(1);
            }
        }
    }

    /// Returns a reference to the voice slot at the given index.
    #[must_use]
    pub fn slot(&self, index: usize) -> Option<&VoiceSlot> {
        self.slots.get(index)
    }

    /// Returns a mutable reference to the voice slot at the given index.
    pub fn slot_mut(&mut self, index: usize) -> Option<&mut VoiceSlot> {
        self.slots.get_mut(index)
    }

    /// Returns an iterator over (index, slot) for all active voices.
    pub fn active_voices(&self) -> impl Iterator<Item = (usize, &VoiceSlot)> {
        self.slots.iter().enumerate().filter(|(_, s)| s.active)
    }

    /// Releases all voices.
    pub fn release_all(&mut self) {
        for slot in &mut self.slots {
            slot.active = false;
            slot.age = 0;
        }
    }
}
