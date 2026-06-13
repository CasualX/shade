use std::{mem, num::NonZeroU32};

/// A stable handle to a slot managed by [`SlotMap`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct SlotKey(NonZeroU32);

impl SlotKey {
	#[inline]
	fn new(index: usize) -> SlotKey {
		let value = (index + 1) as u32;
		SlotKey(NonZeroU32::new(value).expect("slot key value must be non-zero"))
	}

	#[inline]
	pub(super) fn index(self) -> usize {
		(self.0.get() - 1) as usize
	}
}

enum Slot<T> {
	Empty(usize),
	Occupied(T),
	Taken,
}

/// SlotMap, storage with stable keys.
///
/// This is a small handle table: you reserve a slot, receive an opaque key, and
/// use that key to access the stored value later. Slots remain valid until they
/// are explicitly removed, and `take` / `put` let you temporarily move a value
/// out of a reserved slot without freeing the handle.
///
/// # Example
///
/// ```rust
/// # use shade::gui::SlotMap;
/// let mut widgets = SlotMap::new();
///
/// let key = widgets.alloc();      // Reserve a stable handle.
/// widgets.put(key, "button");     // Store the initial value.
///
/// let value = widgets.take(key);   // Move the value out, but keep the slot reserved.
/// assert_eq!(value, "button");
///
/// widgets.put(key, "label");      // Put a replacement value back in.
/// assert_eq!(widgets.get(key), Some(&"label"));
///
/// let value = widgets.remove(key); // Remove the value and free the slot.
/// assert_eq!(value, "label");
/// assert!(!widgets.is_valid(key)); // The slot is no longer live.
/// ```
pub struct SlotMap<T> {
	slots: Vec<Slot<T>>,
	next: usize,
}

impl<T> Default for SlotMap<T> {
	#[inline]
	fn default() -> SlotMap<T> {
		SlotMap::new()
	}
}

impl<T> SlotMap<T> {
	/// Creates an empty instance.
	#[inline]
	pub const fn new() -> SlotMap<T> {
		SlotMap {
			slots: Vec::new(),
			next: 0,
		}
	}

	/// Returns true if the slot is taken, false otherwise.
	#[inline]
	pub fn is_taken(&self, key: SlotKey) -> bool {
		match self.slots.get(key.index()) {
			Some(Slot::Taken) => true,
			_ => false,
		}
	}

	/// Returns true if the slot is occupied, false otherwise.
	#[inline]
	pub fn is_occupied(&self, key: SlotKey) -> bool {
		match self.slots.get(key.index()) {
			Some(Slot::Occupied(_)) => true,
			_ => false,
		}
	}

	/// Returns true is the slot is valid (either occupied or taken), false otherwise.
	#[inline]
	pub fn is_valid(&self, key: SlotKey) -> bool {
		match self.slots.get(key.index()) {
			Some(Slot::Occupied(_)) | Some(Slot::Taken) => true,
			_ => false,
		}
	}

	/// Reserves a slot and returns its key.
	///
	/// The returned slot is marked as taken until a value is inserted with [`put`](SlotMap::put).
	///
	/// # Panics
	///
	/// Panics if the internal freelist is corrupt.
	pub fn alloc(&mut self) -> SlotKey {
		let key = SlotKey::new(self.next);
		let index = key.index();
		if index == self.slots.len() {
			self.slots.push(Slot::Taken);
			self.next = self.slots.len();
		}
		else if index < self.slots.len() {
			match mem::replace(&mut self.slots[index], Slot::Taken) {
				Slot::Empty(next) => self.next = next,
				Slot::Occupied(_) => panic!("slot is already occupied"),
				Slot::Taken => panic!("slot is already taken"),
			}
		}
		else {
			panic!("slot freelist is corrupt");
		}
		return key;
	}

	/// Removes and returns the value stored at `key`.
	///
	/// The slot becomes available for future allocations.
	///
	/// # Panics
	///
	/// Panics if `key` refers to an empty or taken slot, or if it is out of bounds.
	pub fn remove(&mut self, key: SlotKey) -> T {
		let index = key.index();
		let slot = &mut self.slots[index];
		match mem::replace(slot, Slot::Empty(self.next)) {
			Slot::Occupied(value) => {
				self.next = key.index();
				return value;
			},
			Slot::Empty(next) => {
				*slot = Slot::Empty(next);
				panic!("slot is empty");
			},
			Slot::Taken => {
				*slot = Slot::Taken;
				panic!("slot is taken");
			},
		}
	}

	/// Stores `value` in a slot previously reserved with [`alloc`](SlotMap::alloc).
	///
	/// # Panics
	///
	/// Panics if `key` refers to an empty or occupied slot, or if it is out of bounds.
	pub fn put(&mut self, key: SlotKey, value: T) {
		let slot = &mut self.slots[key.index()];
		match slot {
			slot @ Slot::Taken => *slot = Slot::Occupied(value),
			Slot::Empty(_) => panic!("slot is empty"),
			Slot::Occupied(_) => panic!("slot is already occupied"),
		}
	}

	/// Returns an immutable reference to the value at `key`, if present.
	#[inline]
	pub fn get(&self, key: SlotKey) -> Option<&T> {
		match self.slots.get(key.index()) {
			Some(Slot::Occupied(value)) => Some(value),
			_ => None,
		}
	}

	/// Returns a mutable reference to the value at `key`, if present.
	#[inline]
	pub fn get_mut(&mut self, key: SlotKey) -> Option<&mut T> {
		match self.slots.get_mut(key.index()) {
			Some(Slot::Occupied(value)) => Some(value),
			_ => None,
		}
	}

	/// Temporarily removes and returns the value at `key` without freeing the slot.
	///
	/// The slot remains reserved and must be filled again with [`put`](SlotMap::put).
	///
	/// # Panics
	///
	/// Panics if `key` refers to an empty or taken slot, or if it is out of bounds.
	pub fn take(&mut self, key: SlotKey) -> T {
		let slot = &mut self.slots[key.index()];
		match mem::replace(slot, Slot::Taken) {
			Slot::Occupied(value) => value,
			old @ Slot::Empty(_) => {
				*slot = old;
				panic!("slot is empty");
			},
			Slot::Taken => {
				*slot = Slot::Taken;
				panic!("slot is already taken");
			},
		}
	}

	/// Temporarily takes the value at `key`, runs `f`, and then puts it back.
	///
	/// This is useful when the callback needs mutable access to both the value stored at `key` and the map itself.
	///
	/// # Panics
	///
	/// Panics if `key` does not currently contain a value, or if it is out of bounds.
	#[inline]
	pub fn with<R>(&mut self, key: SlotKey, f: impl FnOnce(&mut T, &mut SlotMap<T>) -> R) -> R {
		let mut value = self.take(key);
		let result = f(&mut value, self);
		self.put(key, value);
		result
	}
}
