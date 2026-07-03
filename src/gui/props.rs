use super::*;

/// Key to look up dynamic properties.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct PropKey {
	value: u32,
}

/// PropKey constructor.
#[inline]
#[allow(non_snake_case)]
pub const fn PropKey(value: u32) -> PropKey {
	PropKey { value }
}

impl PropKey {
	/// Returns the underlying value of the key.
	#[inline]
	pub const fn value(&self) -> u32 {
		self.value
	}
}

/// Property storing a basic value.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Property<T> {
	/// Resolves the value from [`AppState::prop`].
	Key(PropKey),
	/// Stores the value directly on the widget.
	Value(T),
}

impl<T: any::Any> Property<T> {
	/// Resolves the property inside `f`.
	#[inline]
	pub fn with<R>(&self, app: &dyn AppState, app_ctx: &dyn AppContext, f: impl FnOnce(&T) -> R) -> Option<R> {
		match self {
			&Property::Key(key) => {
				let mut f = Some(f);
				let mut result = None;
				app.prop(key, app_ctx, &mut |value| {
					let Some(value) = value.downcast_ref::<T>() else {
						#[cfg(debug_assertions)]
						panic!("property callback type mismatch");
						#[cfg(not(debug_assertions))]
						return;
					};
					if let Some(f) = f.take() {
						result = Some(f(value));
					}
					else {
						#[cfg(debug_assertions)]
						panic!("property callback should only run once");
					}
				});
				result
			},
			Property::Value(value) => Some(f(value)),
		}
	}

	/// Resolves the property by copying the value.
	pub fn copied(&self, app: &dyn AppState, app_ctx: &dyn AppContext) -> Option<T> where T: Copy {
		self.with(app, app_ctx, |value| *value)
	}

	/// Resolves the property by copying the value, or returns `default` if unavailable.
	pub fn copied_or(&self, app: &dyn AppState, app_ctx: &dyn AppContext, default: T) -> T where T: Copy {
		self.copied(app, app_ctx).unwrap_or(default)
	}
}

impl<T: any::Any> From<dto::Property<T>> for Property<T> {
	#[inline]
	fn from(value: dto::Property<T>) -> Property<T> {
		match value {
			dto::Property::Key(key) => Property::Key(PropKey(key)),
			dto::Property::Value(value) => Property::Value(value),
		}
	}
}

/// Property storing an owned or static value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OwnedProp<T: 'static + borrow::ToOwned + ?Sized> {
	/// Resolves the value from [`AppState::prop`].
	Key(PropKey),
	/// Uses a static value.
	Static(&'static T),
	/// Owns the value.
	Owned(T::Owned),
}

impl<T: borrow::ToOwned + ?Sized> OwnedProp<T> {
	/// Resolves the property inside `f`.
	pub fn with<R>(&self, app: &dyn AppState, app_ctx: &dyn AppContext, f: impl FnOnce(&T) -> R) -> Option<R> {
		match self {
			&OwnedProp::Key(key) => {
				let mut f = Some(f);
				let mut result = None;
				app.prop(key, app_ctx, &mut |value| {
					let value = if let Some(value) = value.downcast_ref::<T::Owned>() {
						<T::Owned as borrow::Borrow<T>>::borrow(value)
					}
					else if let Some(value) = value.downcast_ref::<&'static T>() {
						*value
					}
					else {
						#[cfg(debug_assertions)]
						panic!("property callback type mismatch");
						#[cfg(not(debug_assertions))]
						return;
					};
					if let Some(f) = f.take() {
						result = Some(f(value));
					}
					else {
						#[cfg(debug_assertions)]
						panic!("property callback should only run once");
					}
				});
				result
			},
			&OwnedProp::Static(value) => Some(f(value)),
			OwnedProp::Owned(value) => Some(f(<T::Owned as borrow::Borrow<T>>::borrow(value))),
		}
	}
}

impl<T: borrow::ToOwned + ?Sized> From<dto::Property<T::Owned>> for OwnedProp<T> {
	#[inline]
	fn from(value: dto::Property<T::Owned>) -> OwnedProp<T> {
		match value {
			dto::Property::Key(key) => OwnedProp::Key(PropKey(key)),
			dto::Property::Value(value) => OwnedProp::Owned(value),
		}
	}
}

/// Property storing a string value.
pub type StringProperty = OwnedProp<str>;
