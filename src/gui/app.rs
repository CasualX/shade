use super::*;

/// Semantic event delivered to the app state.
pub trait AppEvent: any::Any {}

impl dyn AppEvent + '_ {
	/// Downcasts the event to a concrete type.
	#[inline]
	pub fn downcast_ref<T: AppEvent>(&self) -> Option<&T> {
		(self as &dyn any::Any).downcast_ref::<T>()
	}
}

/// Caller-owned context shared across tree traversal.
///
/// Unlike [`AppState`], this value does not change when the scene enters a scoped child state.
/// Consumers can use it for cross-scope state such as an event queue, command buffer, or other root-level services.
pub trait AppContext: any::Any {}

impl dyn AppContext + '_ {
	/// Downcasts the data to a concrete type.
	#[inline]
	pub fn downcast_ref<T: AppContext>(&self) -> Option<&T> {
		(self as &dyn any::Any).downcast_ref::<T>()
	}

	/// Downcasts the data to a concrete type.
	#[inline]
	pub fn downcast_mut<T: AppContext>(&mut self) -> Option<&mut T> {
		(self as &mut dyn any::Any).downcast_mut::<T>()
	}
}

/// Dynamic state store used by the retained-mode GUI tree.
#[allow(unused_variables)]
pub trait AppState {
	/// Returns the immutable state scope for the given widget `key`.
	fn scope(&self, key: SlotKey, app_ctx: &dyn AppContext) -> &dyn AppState;

	/// Returns the mutable state scope for the given widget `key`.
	fn scope_mut(&mut self, key: SlotKey, app_ctx: &mut dyn AppContext) -> &mut dyn AppState;

	/// Called while unwinding event routing for the child scope previously obtained with [`AppState::scope_mut`] for the same widget `key`.
	fn scope_exit(&mut self, key: SlotKey, app_ctx: &mut dyn AppContext) {}

	/// Looks up a dynamically typed property by key and passes it to `f` when available.
	fn prop(&self, key: PropKey, app_ctx: &dyn AppContext, f: &mut dyn FnMut(&dyn any::Any));

	/// Emits a widget-generated event to the currently scoped app state.
	fn emit(&mut self, event: &dyn AppEvent, app_ctx: &mut dyn AppContext) {}
}
