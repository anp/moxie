//! Event types.

use crate::Node;

#[cfg(feature = "webdom")]
use {
    crate::webdom,
    wasm_bindgen::{prelude::*, JsCast},
    web_sys as sys,
};

/// An event that can be received as the first argument to a handler callback.
#[cfg(feature = "webdom")]
pub trait Event: AsRef<web_sys::Event> + JsCast {
    /// The name used to register for this event in `addEventListener`.
    const NAME: &'static str;
}

/// An event that can be received as the first argument to a handler callback.
#[cfg(not(feature = "webdom"))]
pub trait Event {
    /// The name used to register for this event in `addEventListener`.
    const NAME: &'static str;
}

/// A binding of a particular event listener to a DOM node. The listener is
/// removed when this value is dropped.
#[cfg(feature = "webdom")]
#[must_use]
pub struct EventHandle {
    target: Option<web_sys::EventTarget>,
    callback: webdom::Callback,
    name: &'static str,
}

#[cfg(not(feature = "webdom"))]
#[doc(hidden)]
pub(crate) struct EventHandle;

impl EventHandle {
    /// Construct a new `EventHandle`, binding the provided callback to its
    /// target if the target is able to receive events.
    pub fn new<Ev>(_target: &Node, _callback: impl FnMut(Ev) + 'static) -> Self
    where
        Ev: Event,
    {
        #[cfg(not(feature = "webdom"))]
        {
            Self
        }

        #[cfg(feature = "webdom")]
        {
            let name = Ev::NAME;
            let callback = webdom::Callback::new(_callback);
            let target = match _target {
                Node::Concrete(n) => {
                    let target: &web_sys::EventTarget = n.as_ref();
                    target.add_event_listener_with_callback(name, callback.as_fn()).unwrap();
                    Some(target.to_owned())
                }
                #[cfg(feature = "rsdom")]
                _ => None,
            };

            Self { target, callback, name }
        }
    }
}

#[cfg(feature = "webdom")]
impl Drop for EventHandle {
    fn drop(&mut self) {
        if let Some(target) = self.target.take() {
            target.remove_event_listener_with_callback(self.name, self.callback.as_fn()).unwrap();
        }
    }
}

#[cfg(not(feature = "webdom"))]
macro_rules! event_ty {
    ($(#[$attr:meta])* $name:ident, $ty_str:expr, $parent_ty:ty) => {
        $(#[$attr])*
        pub struct $name;

        impl Event for $name {
            const NAME: &'static str = $ty_str;
        }
    };
}

#[cfg(feature = "webdom")]
macro_rules! event_ty {
    ($(#[$attr:meta])* $name:ident, $ty_str:expr, $parent_ty:ty) => {
        $(#[$attr])*
        #[wasm_bindgen]
        pub struct $name($parent_ty);

        impl AsRef<web_sys::Event> for $name {
            fn as_ref(&self) -> &web_sys::Event {
                self.0.as_ref()
            }
        }

        impl AsRef<JsValue> for $name {
            fn as_ref(&self) -> &JsValue {
                self.0.as_ref()
            }
        }

        impl std::ops::Deref for $name {
            type Target = $parent_ty;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl JsCast for $name {
            fn instanceof(val: &JsValue) -> bool {
                <$parent_ty as JsCast>::instanceof(val)
            }

            fn unchecked_from_js(val: JsValue) -> Self {
                $name(<$parent_ty as JsCast>::unchecked_from_js(val))
            }

            fn unchecked_from_js_ref(_val: &JsValue) -> &Self {
                unimplemented!()
            }
        }

        impl Event for $name {
            const NAME: &'static str = $ty_str;
        }
    };
}

event_ty! {
    /// The loading of a resource has been aborted. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/abort
    Abort,
    "abort",
    sys::UiEvent
}

event_ty! {
    /// Progression has been terminated (not due to an error). [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/abort_(ProgressEvent)
    AbortProgress,
    "abort",
    sys::ProgressEvent
}

event_ty! {
    /// A transaction has been aborted. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/abort_indexedDB
    AbortTransaction,
    "abort",
    sys::Event
}

event_ty! {
    /// The associated document has started printing or the print preview has been closed.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/afterprint
    AfterPrint,
    "afterprint",
    sys::Event
}

event_ty! {
    /// A [CSS animation] has aborted. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/animationcancel
    /// [CSS animation]: https://developer.mozilla.org/en-US/docs/CSS/CSS_animations
    AnimationCancel,
    "animationcancel",
    sys::AnimationEvent
}

event_ty! {
    /// A [CSS animation] has completed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/animationend
    /// [CSS animation]: https://developer.mozilla.org/en-US/docs/CSS/CSS_animations
    AnimationEnd,
    "animationend",
    sys::AnimationEvent
}

event_ty! {
    /// A [CSS animation] is ticked. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/animationiteration
    /// [CSS animation]: https://developer.mozilla.org/en-US/docs/CSS/CSS_animations
    AnimationIteration,
    "animationiteration",
    sys::AnimationEvent
}

event_ty! {
    /// A [CSS animation] has started. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/animationstart
    /// [CSS animation]: https://developer.mozilla.org/en-US/docs/CSS/CSS_animations
    AnimationStart,
    "animationstart",
    sys::AnimationEvent
}

event_ty! {
    /// A web application is successfully installed as a progressive web app.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/appinstalled
    AppInstalled,
    "appinstalled",
    sys::Event
}

event_ty! {
    /// The input buffer of a [ScriptProcessorNode] is ready to be processed.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/audioprocess
    /// [ScriptProcessorNode]: https://developer.mozilla.org/en-US/docs/Web/API/ScriptProcessorNode
    AudioProcess,
    "audioprocess",
    sys::AudioProcessingEvent
}

event_ty! {
    /// The user agent has finished capturing audio for speech recognition. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/audioend
    AudioEnd,
    "audioend",
    sys::Event
}

event_ty! {
    /// The user agent has started to capture audio for speech recognition. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/audiostart
    AudioStart,
    "audiostart",
    sys::Event
}

event_ty! {
    /// The associated document is about to be printed or previewed for printing.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/beforeprint
    BeforePrint,
    "beforeprint",
    sys::Event
}

event_ty! {
    /// The window, the document and its resources are about to be unloaded.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/beforeunload
    BeforeUnload,
    "beforeunload",
    sys::BeforeUnloadEvent
}

event_ty! {
    /// A [SMIL] animation element begins. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/beginEvent
    /// [SMIL]: https://developer.mozilla.org/en-US/docs/SVG/SVG_animation_with_SMIL
    SvgAnimationBegin,
    "beginEvent",
    sys::TimeEvent
}

event_ty! {
    /// An open connection to a database is blocking a versionchange transaction on the same
    /// database. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/blocked_indexedDB
    ConnectionBlocked,
    "blocked",
    sys::Event
}

event_ty! {
    /// An element has lost focus (does not bubble). [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/blur
    Blur,
    "blur",
    sys::FocusEvent
}

event_ty! {
    /// The spoken utterance reaches a word or sentence boundary. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/boundary
    SpeechBoundary,
    "boundary",
    sys::SpeechSynthesisEvent
}

event_ty! {
    /// The cancel event fires on a <dialog> when the user instructs the browser that they wish to
    /// dismiss the current open dialog.
    Cancel,
    "cancel",
    sys::Event
}

event_ty! {
    /// The user agent can play the media, but estimates that not enough data has been loaded to
    /// play the media up to its end without having to stop for further buffering of content.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/canplay
    CanPlay,
    "canplay",
    sys::Event
}

event_ty! {
    /// The user agent can play the media up to its end without having to stop for further buffering
    /// of content. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/canplaythrough
    CanPlayThrough,
    "canplaythrough",
    sys::Event
}

event_ty! {
    /// The change event is fired for [`<textarea>`][textarea] elements when a change to the
    /// element's value is committed by the user. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/change
    /// [textarea]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input
    Change,
    "change",
    sys::Event
}

event_ty! {
    /// The battery begins or stops charging. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/chargingchange
    ChargingChange,
    "chargingchange",
    sys::Event
}

event_ty! {
    /// The chargingTime attribute has been updated. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/chargingtimechange
    ChargingTime,
    "chargingtimechange",
    sys::Event
}

event_ty! {
    /// A pointing device button has been pressed and released on an element.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/click
    Click,
    "click",
    sys::MouseEvent
}

event_ty! {
    /// A WebSocket connection has been closed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/close_websocket
    CloseWebsocket,
    "close",
    sys::Event
}

event_ty! {
    /// The rendering of an [OfflineAudioContext] is terminated. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/complete
    /// [OfflineAudioContext]: https://developer.mozilla.org/en-US/docs/Web/API/OfflineAudioContext
    AudioComplete,
    "complete",
    sys::OfflineAudioCompletionEvent
}

event_ty! {
    /// The composition of a passage of text has been completed or canceled.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/compositionend
    CompositionEnd,
    "compositionend",
    sys::CompositionEvent
}

event_ty! {
    /// The composition of a passage of text is prepared (similar to keydown for a keyboard input,
    /// but works with other inputs such as speech recognition). [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/compositionstart
    CompositionStart,
    "compositionstart",
    sys::CompositionEvent
}

event_ty! {
    /// A character is added to a passage of text being composed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/compositionupdate
    CompositionUpdate,
    "compositionupdate",
    sys::CompositionEvent
}

event_ty! {
    /// The right button of the mouse is clicked (before the context menu is displayed).
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/contextmenu
    ContextMenu,
    "contextmenu",
    sys::MouseEvent
}

event_ty! {
    /// The text selection has been added to the clipboard. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/copy
    Cp,
    "copy",
    sys::ClipboardEvent
}

event_ty! {
    /// The text selection has been removed from the document and added to the clipboard.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/cut
    Cut,
    "cut",
    sys::ClipboardEvent
}

event_ty! {
    /// A pointing device button is clicked twice on an element. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dblclick
    DoubleClick,
    "dblclick",
    sys::MouseEvent
}

event_ty! {
    /// A media device such as a camera, microphone, or speaker is connected or removed from the
    /// system. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/devicechange
    DeviceChange,
    "devicechange",
    sys::Event
}

event_ty! {
    /// Fresh data is available from a motion sensor. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/devicemotion
    DeviceMotion,
    "devicemotion",
    sys::DeviceMotionEvent
}

event_ty! {
    /// Fresh data is available from an orientation sensor. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/deviceorientation
    DeviceOrientation,
    "deviceorientation",
    sys::DeviceOrientationEvent
}

event_ty! {
    /// The dischargingTime attribute has been updated. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dischargingtimechange
    DischargingTime,
    "dischargingtimechange",
    sys::Event
}

event_ty! {
    /// The document has finished loading (but not its dependent resources).
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/DOMContentLoaded
    DomContentLoaded,
    "DOMContentLoaded",
    sys::Event
}

event_ty! {
    /// An element or text selection is being dragged (every 350ms). [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/drag
    Drag,
    "drag",
    sys::DragEvent
}

event_ty! {
    /// A drag operation is being ended (by releasing a mouse button or hitting the escape key).
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragend
    DragEnd,
    "dragend",
    sys::DragEvent
}

event_ty! {
    /// A dragged element or text selection enters a valid drop target. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragenter
    DragEnter,
    "dragenter",
    sys::DragEvent
}

event_ty! {
    /// A dragged element or text selection leaves a valid drop target. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragleave
    DragLeave,
    "dragleave",
    sys::DragEvent
}

event_ty! {
    /// An element or text selection is being dragged over a valid drop target (every 350ms).
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragover
    DragOver,
    "dragover",
    sys::DragEvent
}

event_ty! {
    /// The user starts dragging an element or text selection. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragstart
    DragStart,
    "dragstart",
    sys::DragEvent
}

event_ty! {
    /// An element is dropped on a valid drop target. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/drop
    Dropped,
    "drop",
    sys::DragEvent
}

event_ty! {
    /// The duration attribute has been updated. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/durationchange
    DurationChange,
    "durationchange",
    sys::Event
}

event_ty! {
    /// The media has become empty; for example, this event is sent if the media has already been
    /// loaded (or partially loaded), and the load() method is called to reload it.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/emptied
    Emptied,
    "emptied",
    sys::Event
}

event_ty! {
    /// The speech recognition service has disconnected. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/end_(SpeechRecognition)
    SpeechRecognitionEnd,
    "end",
    sys::Event
}

event_ty! {
    /// The utterance has finished being spoken. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/end_(SpeechSynthesis)
    SpeechSynthesisEnd,
    "end",
    sys::SpeechSynthesisEvent
}

event_ty! {
    /// Playback has stopped because the end of the media was reached. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/ended
    PlaybackEnded,
    "ended",
    sys::Event
}

event_ty! {
    /// Playback has stopped because the end of the media was reached. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/ended_(Web_Audio)
    AudioEnded,
    "ended",
    sys::Event
}

event_ty! {
    /// A [SMIL] animation element ends. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/endEvent
    /// [SMIL]: https://developer.mozilla.org/en-US/docs/SVG/SVG_animation_with_SMIL
    SvgAnimationEnd,
    "endEvent",
    sys::TimeEvent
}

event_ty! {
    /// A generic error event.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
    Error,
    "error",
    sys::Event
}

event_ty! {
    /// A resource failed to load. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
    ResourceError,
    "error",
    sys::UiEvent
}

event_ty! {
    /// Progression has failed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
    ProgressError,
    "error",
    sys::ProgressEvent
}

event_ty! {
    /// A WebSocket connection has been closed with prejudice (some data couldn't be sent for
    /// example). [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
    WebsocketError,
    "error",
    sys::Event
}

event_ty! {
    /// An event source connection has been failed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
    EventSourceError,
    "error",
    sys::Event
}

event_ty! {
    /// A request caused an error and failed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
    RequestError,
    "error",
    sys::Event
}

event_ty! {
    /// A speech recognition error occurs. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error_(SpeechRecognitionError)
    SpeechRecognitionError,
    "error",
    sys::Event
}

event_ty! {
    /// An error occurs that prevents the utterance from being successfully spoken.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error_(SpeechSynthesisError)
    SpeechError,
    "error",
    sys::SpeechSynthesisErrorEvent
}

event_ty! {
    /// An element has received focus (does not bubble). [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/focus
    Focus,
    "focus",
    sys::FocusEvent
}

event_ty! {
    /// An element is about to receive focus (bubbles). [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/focusin
    FocusIn,
    "focusin",
    sys::FocusEvent
}

event_ty! {
    /// An element is about to lose focus (bubbles). [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/focusout
    FocusOut,
    "focusout",
    sys::FocusEvent
}

event_ty! {
    /// An element was turned to fullscreen mode or back to normal mode. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/fullscreenchange
    FullscreenChange,
    "fullscreenchange",
    sys::Event
}

event_ty! {
    /// It was impossible to switch to fullscreen mode for technical reasons or because the
    /// permission was denied. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/fullscreenerror
    FullscreenError,
    "fullscreenerror",
    sys::Event
}

event_ty! {
    /// A gamepad has been connected. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/gamepadconnected
    GamepadConnected,
    "gamepadconnected",
    sys::GamepadEvent
}

event_ty! {
    /// A gamepad has been disconnected. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/gamepaddisconnected
    GamepadDisconnected,
    "gamepaddisconnected",
    sys::GamepadEvent
}

event_ty! {
    /// Element receives pointer capture. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/gotpointercapture
    GotPointerCapture,
    "gotpointercapture",
    sys::PointerEvent
}

event_ty! {
    /// The fragment identifier of the URL has changed (the part of the URL after the #).
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/hashchange
    HashChange,
    "hashchange",
    sys::HashChangeEvent
}

event_ty! {
    /// Element lost pointer capture. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/lostpointercapture
    LostPointerCapture,
    "lostpointercapture",
    sys::PointerEvent
}

event_ty! {
    /// The value of an element changes or the content of an element with the attribute
    /// [contenteditable] is modified.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/input
    /// [contenteditable]: https://developer.mozilla.org/en-US/docs/DOM/Element.contentEditable
    Input,
    "input",
    sys::Event
}

event_ty! {
    /// A submittable element has been checked and doesn't satisfy its constraints.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/invalid
    Invalid,
    "invalid",
    sys::Event
}

event_ty! {
    /// A key is pressed down. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/keydown
    KeyDown,
    "keydown",
    sys::KeyboardEvent
}

event_ty! {
    /// A key is released. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/keyup
    KeyUp,
    "keyup",
    sys::KeyboardEvent
}

event_ty! {
    /// The user's preferred languages have changed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/languagechange
    LanguageChange,
    "languagechange",
    sys::Event
}

event_ty! {
    /// The level attribute has been updated. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/levelchange
    LevelChange,
    "levelchange",
    sys::Event
}

event_ty! {
    /// A resource and its dependent resources have finished loading. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/load
    ResourceLoad,
    "load",
    sys::UiEvent
}

event_ty! {
    /// Progression has been successful. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/load_(ProgressEvent)
    ProgressLoad,
    "load",
    sys::ProgressEvent
}

event_ty! {
    /// The first frame of the media has finished loading. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/loadeddata
    DataLoaded,
    "loadeddata",
    sys::Event
}

event_ty! {
    /// The metadata has been loaded. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/loadedmetadata
    MetadataLoaded,
    "loadedmetadata",
    sys::Event
}

event_ty! {
    /// Progress has stopped (after "error", "abort" or "load" have been dispatched).
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/loadend
    LoadEnd,
    "loadend",
    sys::ProgressEvent
}

event_ty! {
    /// Progress has begun. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/loadstart
    LoadStart,
    "loadstart",
    sys::ProgressEvent
}

event_ty! {
    /// The spoken utterance reaches a named SSML "mark" tag. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mark
    SpeechMark,
    "mark",
    sys::SpeechSynthesisEvent
}

event_ty! {
    /// A message is received through a WebSocket. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/message_websocket
    WebsocketMessage,
    "message",
    sys::MessageEvent
}

event_ty! {
    /// A message is received from a Web Worker. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/message_webworker
    WorkerMessage,
    "message",
    sys::MessageEvent
}

event_ty! {
    /// A message is received from a child (i)frame or a parent window. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/message_webmessaging
    ChildMessage,
    "message",
    sys::MessageEvent
}

event_ty! {
    /// A message is received through an event source. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/message_serversentevents
    EventSourceMessage,
    "message",
    sys::MessageEvent
}

event_ty! {
    /// A message error is raised when a message is received by an object. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/messageerror
    MessageError,
    "messageerror",
    sys::MessageEvent
}

event_ty! {
    /// A message is received from a service worker, or a message is received in a service worker
    /// from another context. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/message_(ServiceWorker)
    ServiceWorkerMessage,
    "message",
    sys::MessageEvent
}

event_ty! {
    /// A pointing device button (usually a mouse) is pressed on an element.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mousedown
    MouseDown,
    "mousedown",
    sys::MouseEvent
}

event_ty! {
    /// A pointing device is moved onto the element that has the listener attached.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseenter
    MouseEnter,
    "mouseenter",
    sys::MouseEvent
}

event_ty! {
    /// A pointing device is moved off the element that has the listener attached.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseleave
    MouseLeave,
    "mouseleave",
    sys::MouseEvent
}

event_ty! {
    /// A pointing device is moved over an element. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mousemove
    MouseMove,
    "mousemove",
    sys::MouseEvent
}

event_ty! {
    /// A pointing device is moved off the element that has the listener attached or off one of its
    /// children. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseout
    MouseOut,
    "mouseout",
    sys::MouseEvent
}

event_ty! {
    /// A pointing device is moved onto the element that has the listener attached or onto one of
    /// its children. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseover
    MouseOver,
    "mouseover",
    sys::MouseEvent
}

event_ty! {
    /// A pointing device button is released over an element. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseup
    MouseUp,
    "mouseup",
    sys::MouseEvent
}

event_ty! {
    /// The speech recognition service returns a final result with no significant recognition.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/nomatch
    SpeechRecognitionNoMatch,
    "nomatch",
    sys::SpeechRecognitionEvent
}

event_ty! {
    /// A system notification spawned by [ServiceWorkerRegistration.showNotification()][notif] has
    /// been clicked. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/notificationclick
    /// [notif]: https://developer.mozilla.org/en-US/docs/Web/API/ServiceWorkerRegistration/showNotification
    NotificationClick,
    "notificationclick",
    sys::NotificationEvent
}

event_ty! {
    /// The browser has lost access to the network. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/offline
    Offline,
    "offline",
    sys::Event
}

event_ty! {
    /// The browser has gained access to the network (but particular websites might be unreachable).
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/online
    Online,
    "online",
    sys::Event
}

event_ty! {
    /// A WebSocket connection has been established. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/open_websocket
    WebsocketOpen,
    "open",
    sys::Event
}

event_ty! {
    /// An event source connection has been established. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/open_serversentevents
    EventSourceOpen,
    "open",
    sys::Event
}

event_ty! {
    /// The orientation of the device (portrait/landscape) has changed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/orientationchange
    OrientationChange,
    "orientationchange",
    sys::Event
}

event_ty! {
    /// A session history entry is being traversed from. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pagehide
    PageHide,
    "pagehide",
    sys::PageTransitionEvent
}

event_ty! {
    /// A session history entry is being traversed to. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pageshow
    PageShow,
    "pageshow",
    sys::PageTransitionEvent
}

event_ty! {
    /// Data has been transferred from the system clipboard to the document.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/paste
    Paste,
    "paste",
    sys::ClipboardEvent
}

event_ty! {
    /// Playback has been paused. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pause
    Pause,
    "pause",
    sys::Event
}

event_ty! {
    /// The utterance is paused part way through. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pause_(SpeechSynthesis)
    SpeechPause,
    "pause",
    sys::SpeechSynthesisEvent
}

event_ty! {
    /// The pointer is unlikely to produce any more events. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointercancel
    PointerCancel,
    "pointercancel",
    sys::PointerEvent
}

event_ty! {
    /// The pointer enters the active buttons state. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerdown
    PointerDown,
    "pointerdown",
    sys::PointerEvent
}

event_ty! {
    /// Pointing device is moved inside the hit-testing boundary. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerenter
    PointerEnter,
    "pointerenter",
    sys::PointerEvent
}

event_ty! {
    /// Pointing device is moved out of the hit-testing boundary. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerleave
    PointerLeave,
    "pointerleave",
    sys::PointerEvent
}

event_ty! {
    /// The pointer was locked or released. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerlockchange
    PointerLockChange,
    "pointerlockchange",
    sys::Event
}

event_ty! {
    /// It was impossible to lock the pointer for technical reasons or because the permission was
    /// denied. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerlockerror
    PointerLockError,
    "pointerlockerror",
    sys::Event
}

event_ty! {
    /// The pointer changed coordinates. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointermove
    PointerMove,
    "pointermove",
    sys::PointerEvent
}

event_ty! {
    /// The pointing device moved out of hit-testing boundary or leaves detectable hover range.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerout
    PointerOut,
    "pointerout",
    sys::PointerEvent
}

event_ty! {
    /// The pointing device is moved into the hit-testing boundary. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerover
    PointerOver,
    "pointerover",
    sys::PointerEvent
}

event_ty! {
    /// The pointer leaves the active buttons state. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerup
    PointerUp,
    "pointerup",
    sys::PointerEvent
}

event_ty! {
    /// Playback has begun. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/play
    Play,
    "play",
    sys::Event
}

event_ty! {
    /// Playback is ready to start after having been paused or delayed due to lack of data.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/playing
    Playing,
    "playing",
    sys::Event
}

event_ty! {
    /// A session history entry is being navigated to (in certain cases). [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/popstate
    PopState,
    "popstate",
    sys::PopStateEvent
}

event_ty! {
    /// In progress. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/progress
    Progress,
    "progress",
    sys::ProgressEvent
}

event_ty! {
    /// A [Service Worker] has received a push message. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/push
    /// [Service Worker]: https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API
    Push,
    "push",
    sys::PushEvent
}

event_ty! {
    /// A [PushSubscription] has expired. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pushsubscriptionchange
    /// [PushSubscription]: https://developer.mozilla.org/en-US/docs/Web/API/PushSubscription
    PushSubscriptionChange,
    "pushsubscriptionchange",
    sys::PushEvent
}

event_ty! {
    /// The playback rate has changed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/ratechange
    PlaybackRateChange,
    "ratechange",
    sys::Event
}

event_ty! {
    /// The readyState attribute of a document has changed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/readystatechange
    ReadyStateChange,
    "readystatechange",
    sys::Event
}

event_ty! {
    /// A [SMIL] animation element is repeated. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/repeatEvent
    /// [SMIL]: https://developer.mozilla.org/en-US/docs/SVG/SVG_animation_with_SMIL
    AnimationRepeat,
    "repeatEvent",
    sys::TimeEvent
}

event_ty! {
    /// A form is reset. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/reset
    FormReset,
    "reset",
    sys::Event
}

event_ty! {
    /// The document view has been resized. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/resize
    ViewResize,
    "resize",
    sys::UiEvent
}

event_ty! {
    /// The browser's resource timing buffer is full. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/resourcetimingbufferfull
    TimingBufferFull,
    "resourcetimingbufferfull",
    sys::Event
}

event_ty! {
    /// The speech recognition service returns a result — a word or phrase has been positively
    /// recognized and this has been communicated back to the app. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/result
    SpeechRecognitionResult,
    "result",
    sys::SpeechRecognitionEvent
}

event_ty! {
    /// A paused utterance is resumed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/resume
    SpeechResume,
    "resume",
    sys::SpeechSynthesisEvent
}

event_ty! {
    /// The document view or an element has been scrolled. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/scroll
    Scroll,
    "scroll",
    sys::UiEvent
}

event_ty! {
    /// A <em>seek</em> operation completed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/seeked
    Seeked,
    "seeked",
    sys::Event
}

event_ty! {
    /// A <em>seek</em> operation began. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/seeking
    Seeking,
    "seeking",
    sys::Event
}

event_ty! {
    /// Some text is being selected. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/select
    Select,
    "select",
    sys::UiEvent
}

event_ty! {
    /// A selection just started. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/selectstart
    SelectionStart,
    "selectstart",
    sys::Event
}

event_ty! {
    /// The selection in the document has been changed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/selectionchange
    SelectionChange,
    "selectionchange",
    sys::Event
}

event_ty! {
    /// A contextmenu event was fired on/bubbled to an element that has a [contextmenu] attribute.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/show
    /// [contextmenu]: https://developer.mozilla.org/en-US/docs/DOM/element.contextmenu
    ContextMenuShow,
    "show",
    sys::MouseEvent
}

event_ty! {
    /// The node contents of a [`<slot>`][slot] have changed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/slotchange
    /// [slot]: https://developer.mozilla.org/en-US/docs/Web/API/HTMLSlotElement
    SlotChange,
    "slotchange",
    sys::Event
}

event_ty! {
    /// Any sound — recognisable speech or not — has stopped being detected.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/soundend
    SoundEnd,
    "sounded",
    sys::Event
}

event_ty! {
    /// Any sound — recognisable speech or not — has been detected. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/soundstart
    SoundStart,
    "soundstart",
    sys::Event
}

event_ty! {
    /// Speech recognised by the speech recognition service has stopped being detected.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/speechend
    SpeechEnd,
    "speechend",
    sys::Event
}

event_ty! {
    /// The user agent is trying to fetch media data, but data is unexpectedly not forthcoming.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/stalled
    Stalled,
    "stalled",
    sys::Event
}

event_ty! {
    /// The speech recognition service has begun listening to incoming audio with intent to
    /// recognize grammars associated with the current SpeechRecognition. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/start_(SpeechRecognition)
    SpeechRecognitionStart,
    "start",
    sys::Event
}

event_ty! {
    /// Sound that is recognised by the speech recognition service as speech has been detected.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/speechstart
    SpeechRecognized,
    "speechstart",
    sys::Event
}

event_ty! {
    /// A storage area ([localStorage] or [sessionStorage]) has changed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/storage
    /// [localStorage]: https://developer.mozilla.org/en-US/docs/DOM/Storage#localStorage
    /// [sessionStorage]: https://developer.mozilla.org/en-US/docs/DOM/Storage#sessionStorage
    Storage,
    "storage",
    sys::StorageEvent
}

event_ty! {
    /// A form is submitted. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/submit
    Submit,
    "submit",
    sys::Event
}

event_ty! {
    /// A request successfully completed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/success_indexedDB
    Success,
    "success",
    sys::Event
}

event_ty! {
    /// Media data loading has been suspended. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/suspend
    Suspend,
    "suspend",
    sys::Event
}

event_ty! {
    /// Page loading has been stopped before the [SVG] was loaded. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGAbort
    /// [SVG]: https://developer.mozilla.org/en-US/docs/SVG
    SvgAbort,
    "SVGAbort",
    sys::Event
}

event_ty! {
    /// An error has occurred before the [SVG] was loaded. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGError
    /// [SVG]: https://developer.mozilla.org/en-US/docs/SVG
    SvgError,
    "SVGError",
    sys::Event
}

event_ty! {
    /// An [SVG] document has been loaded and parsed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGLoad
    /// [SVG]: https://developer.mozilla.org/en-US/docs/SVG
    SvgLoad,
    "SVGLoad",
    sys::Event
}

event_ty! {
    /// An [SVG] document is being resized. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGResize
    /// [SVG]: https://developer.mozilla.org/en-US/docs/SVG
    SvgResize,
    "SVGResize",
    sys::Event
}

event_ty! {
    /// An [SVG] document is being scrolled. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGScroll
    /// [SVG]: https://developer.mozilla.org/en-US/docs/SVG
    SvgScroll,
    "SVGScroll",
    sys::Event
}

event_ty! {
    /// An [SVG] document has been removed from a window or frame. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGUnload
    /// [SVG]: https://developer.mozilla.org/en-US/docs/SVG
    SvgUnload,
    "SVGUnload",
    sys::Event
}

event_ty! {
    /// An [SVG] document is being zoomed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGZoom
    /// [SVG]: https://developer.mozilla.org/en-US/docs/SVG
    SvgZoom,
    "SVGZoom",
    sys::Event
}

event_ty! {
    /// The timeout event is fired when progression is terminated due to preset time expiring.
    /// [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/timeout
    Timeout,
    "timeout",
    sys::ProgressEvent
}

event_ty! {
    /// The time indicated by the currentTime attribute has been updated. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/timeupdate
    TimeUpdate,
    "timeupdate",
    sys::Event
}

event_ty! {
    /// A touch point has been disrupted in an implementation-specific manners (too many touch
    /// points for example). [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/touchcancel
    TouchCancel,
    "touchcancel",
    sys::TouchEvent
}

event_ty! {
    /// A touch point is removed from the touch surface. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/touchend
    TouchEnd,
    "touchend",
    sys::TouchEvent
}

event_ty! {
    /// A touch point is moved along the touch surface. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/touchmove
    TouchMove,
    "touchmove",
    sys::TouchEvent
}

event_ty! {
    /// A touch point is placed on the touch surface. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/touchstart
    TouchStart,
    "touchstart",
    sys::TouchEvent
}

event_ty! {
    /// The document or a dependent resource is being unloaded. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/unload
    Unload,
    "unload",
    sys::UiEvent
}

event_ty! {
    /// The content of a tab has become visible or has been hidden. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/visibilitychange
    VisibilityChange,
    "visibilitychange",
    sys::Event
}

event_ty! {
    /// The volume has changed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/volumechange
    VolumeChange,
    "volumechange",
    sys::Event
}

event_ty! {
    /// Playback has stopped because of a temporary lack of data. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/waiting
    Waiting,
    "waiting",
    sys::Event
}

event_ty! {
    /// A wheel button of a pointing device is rotated in any direction. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/wheel
    Wheel,
    "wheel",
    sys::WheelEvent
}

event_ty! {
    /// A [CSS transition] has completed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/transitionend
    /// [CSS transition]: https://developer.mozilla.org/en-US/docs/CSS/CSS_transitions
    TransitionEnd,
    "transitionend",
    sys::TransitionEvent
}

event_ty! {
    /// An attempt was made to open a database with a version number higher than its current
    /// version. A versionchange transaction has been created. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/upgradeneeded_indexedDB
    UpgradeNeeded,
    "upgradeneeded",
    sys::Event
}

event_ty! {
    /// Fresh data is available from a proximity sensor (indicates whether the nearby object is near
    /// the device or not). [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/userproximity
    UserProximity,
    "userproximity",
    sys::UserProximityEvent
}

event_ty! {
    /// Fires when the list of SpeechSynthesisVoice objects that would be returned by the
    /// SpeechSynthesis.getVoices() method has changed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/voiceschanged
    VoicesChanged,
    "voiceschanged",
    sys::Event
}

event_ty! {
    /// A versionchange transaction completed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/versionchange_indexedDB
    TransactionVersionChange,
    "versionchange",
    sys::Event
}

event_ty! {
    /// A transaction successfully completed. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/complete_indexedDB
    TransactionComplete,
    "complete",
    sys::Event
}

event_ty! {
    /// The utterance has begun to be spoken. [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/start_(SpeechSynthesis)
    SpeechStart,
    "start",
    sys::SpeechSynthesisEvent
}
