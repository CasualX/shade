/*!
The main graphics context interface.
*/

pub trait IGraphics {
	fn begin(&mut self);
	fn end(&mut self);
}
