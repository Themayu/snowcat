pub use tokio::sync::{
	// mutex
	Mutex,
	MutexGuard,
	MappedMutexGuard,

	// rwlock
	RwLock,
	RwLockReadGuard,
	RwLockWriteGuard,
	RwLockMappedWriteGuard,
};
use crate::client::{
	ChannelList,
	CharacterList,
};

pub type ChannelCache = RwLock<ChannelList>;
pub type CharacterCache = RwLock<CharacterList>;

// these should always be Send + Sync + Default
fn _assert_channel_cache_traits()
where
	ChannelCache: Default + Send + Sync + 'static,
{ }

fn _assert_character_cache_traits()
where
	CharacterCache: Default + Send + Sync + 'static,
{ }
