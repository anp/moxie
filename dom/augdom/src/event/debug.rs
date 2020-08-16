use crate::sys;
use js_sys::Reflect;
use std::{
    any::{Any, TypeId},
    fmt::{Debug, Formatter, Result as FmtResult},
};
use wasm_bindgen::JsValue;

pub(crate) trait JsDebug: Any + AsRef<JsValue> {
    type Parent: Any + JsDebug;
    const PROPERTIES: &'static [&'static str] = &[];

    /// Use prototypal inheritance to recursively populate a list of fields we
    /// need.
    fn add_props(props: &mut Vec<&str>) {
        props.extend(Self::PROPERTIES);

        // terminate recursion at ourselves
        if TypeId::of::<Self>() != TypeId::of::<Self::Parent>() {
            Self::Parent::add_props(props);
        }
    }
}

pub(crate) struct JsFormatter {
    name: &'static str,
    properties: Vec<&'static str>,
    value: JsValue,
}

impl JsFormatter {
    pub fn new<T>(name: &'static str, t: &T) -> Self
    where
        T: JsDebug,
    {
        let value = t.as_ref().clone();
        let mut properties = vec![];
        T::add_props(&mut properties);

        Self { name, properties, value }
    }
}

impl Debug for JsFormatter {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let mut f = f.debug_struct(self.name);

        for prop in &self.properties {
            let key = JsValue::from_str(prop);
            let val = Reflect::get(&self.value, &key)
                .expect("don't statically declare properties that don't exist");

            if let Some(s) = val.as_string() {
                f.field(prop, &s);
            } else if let Some(n) = val.as_f64() {
                f.field(prop, &n);
            } else if let Some(b) = val.as_bool() {
                f.field(prop, &b);
            } else {
                // TODO check for Array, Object, etc
                // the default case will just leave a `JsValue(...)` wrapping the output
                f.field(prop, &val);
            }
        }

        f.finish()
    }
}

impl JsDebug for sys::AnimationEvent {
    type Parent = sys::Event;

    const PROPERTIES: &'static [&'static str] = &["animationName", "elapsedTime", "pseudoElement"];
}

impl JsDebug for sys::CompositionEvent {
    type Parent = sys::UiEvent;

    const PROPERTIES: &'static [&'static str] = &["data"];
}

impl JsDebug for sys::DragEvent {
    type Parent = sys::MouseEvent;

    const PROPERTIES: &'static [&'static str] = &["dataTransfer"];
}

impl JsDebug for sys::Event {
    type Parent = Self;

    const PROPERTIES: &'static [&'static str] = &[
        "bubbles",
        "cancelable",
        "composed",
        "defaultPrevented",
        "eventPhase",
        "target",
        "timeStamp",
        "type",
        "isTrusted",
    ];
}

impl JsDebug for sys::FocusEvent {
    type Parent = sys::UiEvent;

    const PROPERTIES: &'static [&'static str] = &["relatedTarget"];
}

impl JsDebug for sys::GamepadEvent {
    type Parent = Self;

    const PROPERTIES: &'static [&'static str] = &["gamepad"];
}

impl JsDebug for sys::HashChangeEvent {
    type Parent = sys::Event;

    const PROPERTIES: &'static [&'static str] = &["newUrl", "oldUrl"];
}

impl JsDebug for sys::KeyboardEvent {
    type Parent = sys::UiEvent;

    const PROPERTIES: &'static [&'static str] =
        &["key", "isComposing", "repeat", "altKey", "ctrlKey", "metaKey", "shiftKey"];
}

impl JsDebug for sys::MessageEvent {
    type Parent = sys::Event;

    const PROPERTIES: &'static [&'static str] =
        &["data", "origin", "lastEventId", "source", "ports"];
}

impl JsDebug for sys::MouseEvent {
    type Parent = sys::UiEvent;

    const PROPERTIES: &'static [&'static str] = &[
        "altKey",
        "button",
        "buttons",
        "clientX",
        "clientY",
        "ctrlKey",
        "metaKey",
        "movementX",
        "movementY",
        "region",
        "relatedTarget",
        "screenX",
        "screenY",
        "shiftKey",
    ];
}

impl JsDebug for sys::PageTransitionEvent {
    type Parent = sys::Event;

    const PROPERTIES: &'static [&'static str] = &["persisted"];
}

impl JsDebug for sys::PointerEvent {
    type Parent = sys::MouseEvent;

    const PROPERTIES: &'static [&'static str] = &[
        "pointerId",
        "width",
        "height",
        "pressure",
        "tangentialPressure",
        "tiltX",
        "tiltY",
        "twist",
        "pointerType",
        "isPrimary",
    ];
}

impl JsDebug for sys::ProgressEvent {
    type Parent = sys::Event;

    const PROPERTIES: &'static [&'static str] = &["lengthComputable", "loaded", "total"];
}

impl JsDebug for sys::SpeechRecognitionEvent {
    type Parent = sys::Event;

    const PROPERTIES: &'static [&'static str] =
        &["emma", "interpretation", "resultIndex", "results"];
}

impl JsDebug for sys::StorageEvent {
    type Parent = sys::Event;

    const PROPERTIES: &'static [&'static str] =
        &["key", "newValue", "oldValue", "storageArea", "url"];
}

impl JsDebug for sys::TouchEvent {
    type Parent = sys::UiEvent;

    const PROPERTIES: &'static [&'static str] =
        &["altKey", "changedTouches", "ctrlKey", "metaKey", "shiftKey", "targetTouches", "touches"];
}

impl JsDebug for sys::UiEvent {
    type Parent = sys::Event;

    const PROPERTIES: &'static [&'static str] = &[];
}

impl JsDebug for sys::WheelEvent {
    type Parent = sys::MouseEvent;

    const PROPERTIES: &'static [&'static str] = &["deltaX", "deltaY", "deltaZ", "deltaMode"];
}
