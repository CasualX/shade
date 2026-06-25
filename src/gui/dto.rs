//! Data transfer objects for declarative GUI construction.

use super::*;
use std::collections::HashMap;

/// Declarative property value.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Property<T> {
	Key(u32),
	Value(T),
}

/// Declarative scene description composed of nested widget DTOs.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Scene {
	pub size: cvmath::Vec2i,
	pub roots: Vec<ChildWidget>,
}

/// Context collected while constructing retained widgets from a DTO tree.
#[derive(Clone, Debug, Default)]
pub struct BuildContext {
	names: HashMap<String, SlotKey>,
}

impl BuildContext {
	/// Creates a new instance.
	#[inline]
	pub fn new() -> BuildContext {
		BuildContext::default()
	}

	/// Looks up a named widget key collected during construction.
	#[inline]
	pub fn key(&self, name: &str) -> Option<SlotKey> {
		self.names.get(name).copied()
	}

	#[inline]
	pub(crate) fn insert(&mut self, name: Option<String>, key: SlotKey) {
		if let Some(name) = name {
			self.names.insert(name, key);
		}
	}
}

/// Declarative widget child with explicit parent-relative bounds.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChildWidget {
	pub bounds: cvmath::Bounds2i,
	#[cfg_attr(feature = "serde", serde(flatten))]
	pub widget: Box<Widget>,
}

/// Declarative widget tree.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum Widget {
	Button(Button),
	Checkbox(Checkbox),
	ColorSwatch(ColorSwatch),
	DrawGrid(DrawGrid),
	Label(Label),
	Menu(Menu),
	MenuBar(MenuBar),
	MenuBarItem(MenuBarItem),
	MenuItem(MenuItem),
	Panel(Panel),
	ProgressBar(ProgressBar),
	RadioButton(RadioButton),
	ScrollPanel(ScrollPanel),
	Separator(Separator),
	Slider(Slider),
	Window(Window),
}

/// Declarative button.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Button {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub enabled: Option<Property<bool>>,
	pub content: Box<Widget>,
}

/// Declarative checkbox.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Checkbox {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	pub label: Property<String>,
	pub checked: Property<bool>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub enabled: Option<Property<bool>>,
}

/// Declarative color swatch.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColorSwatch {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	pub color: Property<cvmath::Vec4<u8>>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub border: Option<Property<cvmath::Vec4<u8>>>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub inset: Option<i32>,
}

/// Declarative decorative grid.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DrawGrid {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub background: Option<cvmath::Vec4<u8>>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub line_color: Option<cvmath::Vec4<u8>>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub spacing: Option<i32>,
}

/// Declarative label.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Label {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	pub text: Property<String>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub font_size: Option<f32>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub color: Option<cvmath::Vec4<u8>>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub align: Option<d2::TextAlign>,
}

/// Declarative vertical menu.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Menu {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	pub children: Vec<Widget>,
}

/// Declarative horizontal menu bar.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MenuBar {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	pub children: Vec<MenuBarItem>,
}

/// Declarative item that opens a menu from a menu bar.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MenuBarItem {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	pub label: Property<String>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub enabled: Option<Property<bool>>,
	pub menu: Box<Menu>,
}

/// Declarative menu item.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MenuItem {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	pub label: Property<String>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub enabled: Option<Property<bool>>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub submenu: Option<Box<Menu>>,
}

/// Declarative container.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Panel {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	pub children: Vec<ChildWidget>,
}

/// Declarative progress bar.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProgressBar {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	pub value: Property<f32>,
	pub fill: Property<cvmath::Vec4<u8>>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub background: Option<Property<cvmath::Vec4<u8>>>,
}

/// Declarative radio button.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RadioButton {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	pub label: Property<String>,
	pub selected: Property<usize>,
	pub index: usize,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub enabled: Option<Property<bool>>,
}

/// Declarative scroll panel.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScrollPanel {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	pub content_height: i32,
	pub content: Box<Widget>,
}

/// Declarative menu separator.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Separator {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
}

/// Declarative slider.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Slider {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	pub value: Property<f32>,
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub enabled: Option<Property<bool>>,
}

/// Declarative window.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Window {
	#[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
	pub name: Option<String>,
	pub title: Property<String>,
	pub content: Box<Widget>,
}
