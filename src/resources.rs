#![allow(dead_code)]

use std::collections::HashMap;
use crate::handle::Handle;

/// Trait for resources and their associated Handle type.
pub trait Resource {
	type Handle: Handle;
}

/// Generic resource map for OpenGL resources.
///
/// Maps handles to values, names to handles and manages the next available ID.
pub struct ResourceMap<T: Resource> {
	map: HashMap<<T::Handle as Handle>::Raw, T>,
	names: HashMap<String, <T::Handle as Handle>::Raw>,
	next_id: T::Handle,
}

impl<T: Resource> ResourceMap<T> {
	/// Creates a new resource map.
	pub fn new() -> ResourceMap<T> {
		ResourceMap {
			map: HashMap::new(),
			names: HashMap::new(),
			next_id: <T::Handle as Default>::default(),
		}
	}

	/// Inserts a new resource into the map and creates a handle.
	pub fn insert(&mut self, name: Option<&str>, resource: T) -> T::Handle {
		self.next_id = self.next_id.next();
		let raw = self.next_id.id();
		let id = <T::Handle as Handle>::create(raw);
		self.map.insert(raw, resource);
		if let Some(name) = name {
			self.names.insert(name.to_string(), raw);
		}
		id
	}

	/// Returns a reference to the resource with the given handle.
	pub fn get(&self, id: T::Handle) -> Option<&T> {
		let raw = <T::Handle as Handle>::id(&id);
		self.map.get(&raw)
	}

	/// Returns a mutable reference to the resource with the given handle.
	pub fn get_mut(&mut self, id: T::Handle) -> Option<&mut T> {
		let raw = <T::Handle as Handle>::id(&id);
		self.map.get_mut(&raw)
	}

	/// Finds a resource by name and returns its handle.
	pub fn find_id(&self, name: &str) -> Option<T::Handle> {
		self.names.get(name).map(|id| <T::Handle as Handle>::create(*id))
	}

	/// Removes a resource from the map and returns it.
	pub fn remove(&mut self, id: T::Handle, free_handle: bool) -> Option<T> {
		assert!(free_handle, "not yet implemented");
		let raw = <T::Handle as Handle>::id(&id);
		self.map.remove(&raw)
	}
}
