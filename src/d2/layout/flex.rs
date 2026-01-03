use super::*;

/// Flexible layout in one dimension.
///
/// Splits a given span into sub-spans based on a template and justification.
///
/// # Parameters
///
/// - `min`, `max`: The minimum and maximum values of the range.
/// - `gap`: The gap between spans.
/// - `justify`: The justification method for the spans.
/// - `template`: A slice of `Unit` values that define the size of each segment.
/// - Returns the resulting spans start and end values.
///
/// Fractional units use the remaining space after the absolute and percentage units are calculated.
///
/// When fractional units are used the justify parameter is ignored.
///
/// # Example
///
/// ```rust
/// use shade::d2::layout::*;
///
/// let gap = Some(Unit::Abs(5.0));
/// let justify = Justify::End;
/// let template = [Unit::Abs(10.0), Unit::Pct(50.0), Unit::Abs(15.0)];
///
/// let spans = flex1d(0.0, 100.0, gap, justify, &template);
///
/// assert_eq!(spans, [[15.0, 25.0], [30.0, 80.0], [85.0, 100.0]]);
/// ```
#[inline]
pub fn flex1d<const N: usize>(min: f32, max: f32, gap: Option<Unit>, justify: Justify, template: &[Unit; N]) -> [[f32; 2]; N] {
	let mut spans = [[0.0; 2]; N];
	flex1d_slice(min, max, gap, justify, template, &mut spans);
	spans
}

/// Flexible layout in one dimension.
///
/// See [flex1d] for details.
pub fn flex1d_slice(
	min: f32,
	max: f32,
	gap: Option<Unit>,
	justify: Justify,
	template: &[Unit],
	spans: &mut [[f32; 2]],
) {
	// Sanity check the parameters
	debug_assert_eq!(template.len(), spans.len());
	if template.len() == 0 || template.len() != spans.len() {
		return;
	}
	debug_assert!(min <= max);
	if min > max {
		return;
	}

	let size = max - min;

	// Calculate the total used space and fraction units
	let mut total_used = 0.0;
	let mut total_fr = match gap {
		None => 0.0,
		Some(Unit::Abs(_)) => 0.0,
		Some(Unit::Pct(_)) => 0.0,
		Some(Unit::Fr(v)) => v * (template.len() - 1) as f32,
	};

	// Analyze the template to extract total used space and fraction units
	for i in 0..template.len() {
		match template[i] {
			Unit::Abs(v) => total_used += v,
			Unit::Pct(v) => total_used += v * (0.01 * size),
			Unit::Fr(v) => total_fr += v,
		}
	}

	// The remaining space is used by the fraction units
	let mut fr_space = size - total_used;

	// Calculate the gap size if relevant
	let gap_size = if template.len() == 1 { 0.0 }
	else {
		match gap {
			None => 0.0,
			Some(Unit::Abs(v)) => v,
			Some(Unit::Pct(v)) => v * (0.01 * size),
			Some(Unit::Fr(v)) => v * (fr_space / total_fr),
		}
	};

	// Reduce the remaining space for fraction units if the gap is not a fraction
	if !matches!(gap, Some(Unit::Fr(_))) {
		fr_space -= gap_size * (template.len() - 1) as f32;
	}

	// Both fraction units and justification use the same space
	let justify_space = if total_fr == 0.0 { fr_space } else { 0.0 };

	// Calculate the values for justification
	let (mut base, shift) = match justify {
		Justify::Start => (0.0, 0.0),
		Justify::Center => (justify_space * 0.5, 0.0),
		Justify::End => (justify_space, 0.0),
		Justify::SpaceBetween => {
			let value = if spans.len() > 1 { justify_space / (spans.len() - 1) as f32 } else { 0.0 };
			(0.0, value)
		}
		Justify::SpaceAround => {
			let value = justify_space / spans.len() as f32;
			(value * 0.5, value)
		}
		Justify::SpaceEvenly => {
			let value = justify_space / (spans.len() + 1) as f32;
			(value, value)
		}
	};

	// Compute the spans
	let mut pos = min;
	for i in 0..template.len() {
		let size = match template[i] {
			Unit::Abs(v) => v,
			Unit::Pct(v) => v * (0.01 * size),
			Unit::Fr(v) => v * (fr_space / total_fr),
		};

		spans[i] = [base + pos, base + pos + size];
		pos += size + gap_size;
		base += shift;
	}
}

use cvmath::Bounds2;

/// Flexible horizontal or vertical layout.
#[inline]
pub fn flex2d<const N: usize>(rect: Bounds2<f32>, orientation: Orientation, gap: Option<Unit>, justify: Justify, template: &[Unit; N]) -> [Bounds2<f32>; N] {
	let values = match orientation {
		Orientation::Vertical => flex1d(rect.mins.y, rect.maxs.y, gap, justify, template),
		Orientation::Horizontal => flex1d(rect.mins.x, rect.maxs.x, gap, justify, template),
	};
	let mut rects = [Bounds2::ZERO; N];
	for (i, &[begin, end]) in values.iter().enumerate() {
		match orientation {
			Orientation::Vertical => rects[i] = Bounds2::c(rect.mins.x, begin, rect.maxs.x, end),
			Orientation::Horizontal => rects[i] = Bounds2::c(begin, rect.mins.y, end, rect.maxs.y),
		}
	}
	rects
}
