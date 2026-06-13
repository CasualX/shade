use super::*;

/// Semantic event delivered to the app state.
pub trait UserEvent: any::Any {}

impl dyn UserEvent + '_ {
	#[inline]
	/// Downcasts the event to a concrete type.
	pub fn downcast_ref<T: UserEvent>(&self) -> Option<&T> {
		(self as &dyn any::Any).downcast_ref::<T>()
	}
}

/// Dynamic state store used by the retained-mode GUI tree.
#[allow(unused_variables)]
pub trait AppState {
	/// Returns the immutable state scope for the given widget `key`.
	fn scope(&self, key: SlotKey) -> &dyn AppState;

	/// Returns the mutable state scope for the given widget `key`.
	fn scope_mut(&mut self, key: SlotKey) -> &mut dyn AppState;

	/// Called while unwinding event routing for the child scope previously obtained with [`AppState::scope_mut`] for the same widget `key`.
	fn scope_exit(&mut self, key: SlotKey) {}

	/// Looks up a dynamically typed property by key and passes it to `f` when available.
	fn prop(&self, key: PropKey, f: &mut dyn FnMut(&dyn any::Any));

	/// Emits a widget-generated event back to the application.
	fn emit(&mut self, event: &dyn UserEvent) {}
}
