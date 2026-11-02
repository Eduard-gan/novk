use crate::errors::{LibraryError, LibraryResult};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct PendingUpload {
    pub uuid: Uuid,
    pub bytes: Arc<Vec<u8>>,
    pub original_filename: String,
    pub created_at: Instant,
    pub tag: id3::Tag,
    pub tag_error: Option<&'static str>,
}

#[derive(Default)]
pub struct PendingUploads {
    slots: Mutex<HashMap<i32, PendingUpload>>,
}

impl PendingUploads {
    pub fn insert(&self, user_id: i32, bytes: Vec<u8>, original_filename: String) -> Uuid {
        let (tag, tag_error) = crate::models::songs::read_tag(&bytes);
        let mut slots = self.slots.lock().expect("pending upload mutex poisoned");
        slots.retain(|_, upload| upload.created_at.elapsed() < Duration::from_secs(1800));
        let uuid = Uuid::new_v4();
        slots.insert(
            user_id,
            PendingUpload {
                uuid,
                bytes: Arc::new(bytes),
                original_filename,
                created_at: Instant::now(),
                tag,
                tag_error,
            },
        );
        if let Some(kind) = tag_error {
            tracing::warn!(
                error_kind = kind,
                "upload has unreadable ID3 tags, original bytes will be preserved"
            );
        }
        uuid
    }

    pub fn get(&self, user_id: i32, uuid: Option<Uuid>) -> LibraryResult<PendingUpload> {
        let mut slots = self.slots.lock().expect("pending upload mutex poisoned");
        slots.retain(|_, upload| upload.created_at.elapsed() < Duration::from_secs(1800));
        slots
            .get(&user_id)
            .filter(|upload| uuid.is_none_or(|uuid| uuid == upload.uuid))
            .cloned()
            .ok_or(LibraryError::UploadExpired)
    }

    pub fn take(&self, user_id: i32, uuid: Uuid) -> LibraryResult<PendingUpload> {
        let mut slots = self.slots.lock().expect("pending upload mutex poisoned");
        slots.retain(|_, upload| upload.created_at.elapsed() < Duration::from_secs(1800));
        if slots.get(&user_id).is_none_or(|upload| upload.uuid != uuid) {
            return Err(LibraryError::UploadExpired);
        }
        Ok(slots
            .remove(&user_id)
            .expect("validated pending upload disappeared while locked"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pending uploads must belong to one user and expire after 30 minutes.
    ///
    /// Case: TS-18 user 1 replaces A with B, user 2 requests B, then B ages 31 minutes.
    ///
    /// Expected: A, another user's B, and expired B are unavailable, while current B is accessible.
    #[test]
    fn ts18_pending_slots() {
        let store = PendingUploads::default();
        let a = store.insert(1, vec![1], "a.mp3".to_string());
        let b = store.insert(1, vec![2], "b.mp3".to_string());
        assert!(store.get(1, Some(a)).is_err());
        assert_eq!(store.get(1, Some(b)).unwrap().uuid, b);
        assert!(store.get(2, Some(b)).is_err());
        store.slots.lock().unwrap().get_mut(&1).unwrap().created_at = Instant::now() - Duration::from_secs(31 * 60);
        assert!(store.get(1, Some(b)).is_err());
    }
}
