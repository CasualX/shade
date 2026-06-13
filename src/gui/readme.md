Retained GUI framework.

This module contains a small retained GUI framework, a simple way to create and manage user interfaces.

Conceptually the GUI is split between two parts: the UI tree composed of widgets and the application state. The UI tree is driven by directly by data from the application state, receives raw input events, and the application state is updated by semantic user events emitted from the UI tree.

## Widgets

Widgets are owned by the [`Scene`](Scene) and are stored in a slotmap using handles [`SlotKey`](SlotKey).

A [`Scene`](Scene) represents the interactive state of the UI tree, and is responsible for handling input events and drawing the widgets.

Widgets are constructed by their corresponding [DTO](dto) structs, which can be deserialized from data.

Data provided by the application state is defined by properties of the widgets. The mapping of [`PropKey`](struct@PropKey) to data is defined by the user of the GUI framework, and is used by the widgets to retrieve data from the application state. Typically you just define them as increasing numbers and match on them to retrieve the corresponding data.

## Application State

The application state is defined by the [`AppState`](AppState) trait, and is implemented by the user of the GUI framework. The application state is responsible for providing data to the UI tree, and for updating itself based on events from the UI tree.

Data is provided by implementing the [`prop`](AppState::prop) method, which given a [`PropKey`](struct@PropKey) provides the corresponding data to the given callback as `&dyn Any`.

Widgets receive [`InputEvent`](InputEvent) values from the scene and can emit semantic [`UserEvent`](UserEvent) values to [`emit`](AppState::emit). Emitted events can carry additional context such as which widget emitted the event and any other data.

To manage the application state, sections can be scoped with [`scope`](AppState::scope) and [`scope_mut`](AppState::scope_mut), PropKeys only need  to be unique within their scope, and the same PropKey can be reused in different scopes. This allows for better organization of the application state, and for reusing widgets with the same PropKeys in different parts of the UI tree.

A core limitation is emitted events must be handled in the same scope they were emitted.
