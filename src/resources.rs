use super::*;

/// Sealed trait for all resources.
pub trait Resource: any::Any {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}
