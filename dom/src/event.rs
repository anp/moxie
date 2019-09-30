use {
    wasm_bindgen::{prelude::*, JsCast},
    web_sys as sys,
};

pub trait Event: AsRef<web_sys::Event> + JsCast {
    const NAME: &'static str;
}

struct Callback {
    cb: Closure<dyn FnMut(JsValue)>,
}

impl Callback {
    fn new<Ev>(mut cb: impl FnMut(Ev) + 'static) -> Self
    where
        Ev: Event,
    {
        let cb = Closure::wrap(Box::new(move |ev: JsValue| {
            let ev: Ev = ev.dyn_into().unwrap();
            cb(ev);
        }) as Box<dyn FnMut(JsValue)>);
        Self { cb }
    }

    fn as_fn(&self) -> &js_sys::Function {
        self.cb.as_ref().unchecked_ref()
    }
}

#[must_use]
pub struct EventHandle {
    target: web_sys::EventTarget,
    callback: Callback,
    name: &'static str,
}

impl EventHandle {
    pub(crate) fn new<Ev>(target: web_sys::EventTarget, callback: impl FnMut(Ev) + 'static) -> Self
    where
        Ev: Event,
    {
        let callback = Callback::new(callback);
        let name = Ev::NAME;
        target
            .add_event_listener_with_callback(name, callback.as_fn())
            .unwrap();
        Self {
            target,
            callback,
            name,
        }
    }
}

impl Drop for EventHandle {
    fn drop(&mut self) {
        self.target
            .remove_event_listener_with_callback(self.name, self.callback.as_fn())
            .unwrap();
    }
}

macro_rules! event_ty {
    ($name:ident, $ty_str:expr, $parent_ty:ty) => {
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

/// The loading of a resource has been aborted. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/abort
event_ty!(Abort, "abort", sys::UiEvent);

/// Progression has been terminated (not due to an error). [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/abort_(ProgressEvent)
event_ty!(AbortProgress, "abort", sys::ProgressEvent);

/// A transaction has been aborted. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/abort_indexedDB
event_ty!(AbortTransaction, "abort", sys::Event);

/// The associated document has started printing or the print preview has been closed.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/afterprint
event_ty!(AfterPrint, "afterprint", sys::Event);

/// A [CSS animation] has aborted. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/animationcancel
/// [CSS animation]: https://developer.mozilla.org/en-US/docs/CSS/CSS_animations
event_ty!(AnimationCancel, "animationcancel", sys::AnimationEvent);

/// A [CSS animation] has completed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/animationend
/// [CSS animation]: https://developer.mozilla.org/en-US/docs/CSS/CSS_animations
event_ty!(AnimationEnd, "animationend", sys::AnimationEvent);

/// A [CSS animation] is ticked. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/animationiteration
/// [CSS animation]: https://developer.mozilla.org/en-US/docs/CSS/CSS_animations
event_ty!(
    AnimationIteration,
    "animationiteration",
    sys::AnimationEvent
);

/// A [CSS animation] has started. [MDN documentation][mdn]
///
/// [mdn]: <a href="https://developer.mozilla.org/en-US/docs/Web/Events/animationstart"></a>
/// [CSS animation]: https://developer.mozilla.org/en-US/docs/CSS/CSS_animations
event_ty!(AnimationStart, "animationstart", sys::AnimationEvent);

/// A web application is successfully installed as a progressive web app. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/appinstalled
event_ty!(AppInstalled, "appinstalled", sys::Event);

/// The input buffer of a [ScriptProcessorNode] is ready to be processed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/audioprocess
/// [ScriptProcessorNode]: https://developer.mozilla.org/en-US/docs/Web/API/ScriptProcessorNode
event_ty!(AudioProcess, "audioprocess", sys::AudioProcessingEvent);

/// The user agent has finished capturing audio for speech recognition. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/audioend
event_ty!(AudioEnd, "audioend", sys::Event);

/// The user agent has started to capture audio for speech recognition. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/audiostart
event_ty!(AudioStart, "audiostart", sys::Event);

/// The associated document is about to be printed or previewed for printing. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/beforeprint
event_ty!(BeforePrint, "beforeprint", sys::Event);

/// The window, the document and its resources are about to be unloaded. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/beforeunload
event_ty!(BeforeUnload, "beforeunload", sys::BeforeUnloadEvent);

/// A [SMIL] animation element begins. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/beginEvent
/// [SMIL]: https://developer.mozilla.org/en-US/docs/SVG/SVG_animation_with_SMIL
event_ty!(SvgAnimationBegin, "beginEvent", sys::TimeEvent);

/// An open connection to a database is blocking a versionchange transaction on the same database.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/blocked_indexedDB
event_ty!(ConnectionBlocked, "blocked", sys::Event);

/// An element has lost focus (does not bubble). [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/blur
event_ty!(Blur, "blur", sys::FocusEvent);

/// The spoken utterance reaches a word or sentence boundary. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/boundary
event_ty!(SpeechBoundary, "boundary", sys::SpeechSynthesisEvent);

/// The user agent can play the media, but estimates that not enough data has been loaded to play
/// the media up to its end without having to stop for further buffering of content.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/canplay
event_ty!(CanPlay, "canplay", sys::Event);

/// The user agent can play the media up to its end without having to stop for further buffering of
/// content. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/canplaythrough
event_ty!(CanPlayThrough, "canplaythrough", sys::Event);

/// The change event is fired for [`<textarea>`][textarea] elements when a change to the element's
/// value is committed by the user. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/change
/// [textarea]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input
event_ty!(Change, "change", sys::Event);

/// The battery begins or stops charging. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/chargingchange
event_ty!(ChargingChange, "chargingchange", sys::Event);

/// The chargingTime attribute has been updated. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/chargingtimechange
event_ty!(ChargingTime, "chargingtimechange", sys::Event);

/// A pointing device button has been pressed and released on an element. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/click
event_ty!(Click, "click", sys::MouseEvent);

/// A WebSocket connection has been closed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/close_websocket
event_ty!(CloseWebsocket, "close", sys::Event);

/// The rendering of an [OfflineAudioContext] is terminated. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/complete
/// [OfflineAudioContext]: https://developer.mozilla.org/en-US/docs/Web/API/OfflineAudioContext
event_ty!(AudioComplete, "complete", sys::OfflineAudioCompletionEvent);

/// The composition of a passage of text has been completed or canceled. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/compositionend
event_ty!(CompositionEnd, "compositionend", sys::CompositionEvent);

/// The composition of a passage of text is prepared (similar to keydown for a keyboard input, but
/// works with other inputs such as speech recognition). [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/compositionstart
event_ty!(CompositionStart, "compositionstart", sys::CompositionEvent);

/// A character is added to a passage of text being composed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/compositionupdate
event_ty!(
    CompositionUpdate,
    "compositionupdate",
    sys::CompositionEvent
);

/// The right button of the mouse is clicked (before the context menu is displayed).
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/contextmenu
event_ty!(ContextMenu, "contextmenu", sys::MouseEvent);

/// The text selection has been added to the clipboard. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/copy
event_ty!(Copy, "copy", sys::ClipboardEvent);

/// The text selection has been removed from the document and added to the clipboard.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/cut
event_ty!(Cut, "cut", sys::ClipboardEvent);

/// A pointing device button is clicked twice on an element. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dblclick
event_ty!(DoubleClick, "dblclick", sys::MouseEvent);

/// A media device such as a camera, microphone, or speaker is connected or removed from the system.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/devicechange
event_ty!(DeviceChange, "devicechange", sys::Event);

/// Fresh data is available from a motion sensor. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/devicemotion
event_ty!(DeviceMotion, "devicemotion", sys::DeviceMotionEvent);

/// Fresh data is available from an orientation sensor. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/deviceorientation
event_ty!(
    DeviceOrientation,
    "deviceorientation",
    sys::DeviceOrientationEvent
);

/// The dischargingTime attribute has been updated. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dischargingtimechange
event_ty!(DischargingTime, "dischargingtimechange", sys::Event);

/// The document has finished loading (but not its dependent resources). [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/DOMContentLoaded
event_ty!(DomContentLoaded, "DOMContentLoaded", sys::Event);

/// An element or text selection is being dragged (every 350ms). [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/drag
event_ty!(Drag, "drag", sys::DragEvent);

/// A drag operation is being ended (by releasing a mouse button or hitting the escape key).
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragend
event_ty!(DragEnd, "dragend", sys::DragEvent);

/// A dragged element or text selection enters a valid drop target. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragenter
event_ty!(DragEnter, "dragenter", sys::DragEvent);

/// A dragged element or text selection leaves a valid drop target. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragleave
event_ty!(DragLeave, "dragleave", sys::DragEvent);

/// An element or text selection is being dragged over a valid drop target (every 350ms).
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragover
event_ty!(DragOver, "dragover", sys::DragEvent);

/// The user starts dragging an element or text selection. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragstart
event_ty!(DragStart, "dragstart", sys::DragEvent);

/// An element is dropped on a valid drop target. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/drop
event_ty!(Dropped, "drop", sys::DragEvent);

/// The duration attribute has been updated. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/durationchange
event_ty!(DurationChange, "durationchange", sys::Event);

/// The media has become empty; for example, this event is sent if the media has already been loaded
/// (or partially loaded), and the [load()][load] method is called to reload it.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/emptied
/// [load]: https://developer.mozilla.org/en-US/docs/XPCOM_Interface_Reference/NsIDOMHTMLMediaElement
event_ty!(Emptied, "emptied", sys::Event);

/// The speech recognition service has disconnected. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/end_(SpeechRecognition)
event_ty!(SpeechRecognitionEnd, "end", sys::Event);

/// The utterance has finished being spoken. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/end_(SpeechSynthesis)
event_ty!(SpeechSynthesisEnd, "end", sys::SpeechSynthesisEvent);

/// Playback has stopped because the end of the media was reached. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/ended
event_ty!(PlaybackEnded, "ended", sys::Event);

/// Playback has stopped because the end of the media was reached. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/ended_(Web_Audio)
event_ty!(AudioEnded, "ended", sys::Event);

/// A [SMIL] animation element ends. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/endEvent
/// [SMIL]: https://developer.mozilla.org/en-US/docs/SVG/SVG_animation_with_SMIL
event_ty!(SvgAnimationEnd, "endEvent", sys::TimeEvent);

/// A resource failed to load. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
event_ty!(ResourceError, "error", sys::UiEvent);

/// Progression has failed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
event_ty!(ProgressError, "error", sys::ProgressEvent);

/// A WebSocket connection has been closed with prejudice (some data couldn't be sent for example).
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
event_ty!(WebsocketError, "error", sys::Event);

/// An event source connection has been failed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
event_ty!(EventSourceError, "error", sys::Event);

/// A request caused an error and failed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
event_ty!(RequestError, "error", sys::Event);

/// A speech recognition error occurs. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error_(SpeechRecognitionError)
event_ty!(SpeechRecognitionError, "error", sys::Event);

/// An error occurs that prevents the utterance from being successfully spoken.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error_(SpeechSynthesisError)
event_ty!(SpeechError, "error", sys::SpeechSynthesisErrorEvent);

/// An element has received focus (does not bubble). [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/focus
event_ty!(Focus, "focus", sys::FocusEvent);

/// An element is about to receive focus (bubbles). [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/focusin
event_ty!(FocusIn, "focusin", sys::FocusEvent);

/// An element is about to lose focus (bubbles). [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/focusout
event_ty!(FocusOut, "focusout", sys::FocusEvent);

/// An element was turned to fullscreen mode or back to normal mode. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/fullscreenchange
event_ty!(FullscreenChange, "fullscreenchange", sys::Event);

/// It was impossible to switch to fullscreen mode for technical reasons or because the permission
/// was denied. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/fullscreenerror
event_ty!(FullscreenError, "fullscreenerror", sys::Event);

/// A gamepad has been connected. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/gamepadconnected
event_ty!(GamepadConnected, "gamepadconnected", sys::GamepadEvent);

/// A gamepad has been disconnected. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/gamepaddisconnected
event_ty!(
    GamepadDisconnected,
    "gamepaddisconnected",
    sys::GamepadEvent
);

/// Element receives pointer capture. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/gotpointercapture
event_ty!(GotPointerCapture, "gotpointercapture", sys::PointerEvent);

/// The fragment identifier of the URL has changed (the part of the URL after the #).
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/hashchange
event_ty!(HashChange, "hashchange", sys::HashChangeEvent);

/// Element lost pointer capture. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/lostpointercapture
event_ty!(LostPointerCapture, "lostpointercapture", sys::PointerEvent);

/// The value of an element changes or the content of an element with the attribute
/// [contenteditable] is modified.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/input
/// [contenteditable]: https://developer.mozilla.org/en-US/docs/DOM/Element.contentEditable
event_ty!(Input, "input", sys::Event);

/// A submittable element has been checked and doesn't satisfy its constraints.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/invalid
event_ty!(Invalid, "invalid", sys::Event);

/// A key is pressed down. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/keydown
event_ty!(KeyDown, "keydown", sys::KeyboardEvent);

/// A key is released. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/keyup
event_ty!(KeyUp, "keyup", sys::KeyboardEvent);

/// The user's preferred languages have changed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/languagechange
event_ty!(LanguageChange, "languagechange", sys::Event);

/// The level attribute has been updated. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/levelchange
event_ty!(LevelChange, "levelchange", sys::Event);

/// A resource and its dependent resources have finished loading. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/load
event_ty!(ResourceLoad, "load", sys::UiEvent);

/// Progression has been successful. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/load_(ProgressEvent)
event_ty!(ProgressLoad, "load", sys::ProgressEvent);

/// The first frame of the media has finished loading. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/loadeddata
event_ty!(DataLoaded, "loadeddata", sys::Event);

/// The metadata has been loaded. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/loadedmetadata
event_ty!(MetadataLoaded, "loadedmetadata", sys::Event);

/// Progress has stopped (after "error", "abort" or "load" have been dispatched).
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/loadend
event_ty!(LoadEnd, "loadend", sys::ProgressEvent);

/// Progress has begun. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/loadstart
event_ty!(LoadStart, "loadstart", sys::ProgressEvent);

/// The spoken utterance reaches a named SSML "mark" tag. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mark
event_ty!(SpeechMark, "mark", sys::SpeechSynthesisEvent);

/// A message is received through a WebSocket. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/message_websocket
event_ty!(WebsocketMessage, "message", sys::MessageEvent);

/// A message is received from a Web Worker. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/message_webworker
event_ty!(WorkerMessage, "message", sys::MessageEvent);

/// A message is received from a child (i)frame or a parent window. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/message_webmessaging
event_ty!(ChildMessage, "message", sys::MessageEvent);

/// A message is received through an event source. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/message_serversentevents
event_ty!(EventSourceMessage, "message", sys::MessageEvent);

/// A message error is raised when a message is received by an object. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/messageerror
event_ty!(MessageError, "messageerror", sys::MessageEvent);

/// A message is received from a service worker, or a message is received in a service worker from
/// another context. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/message_(ServiceWorker)
event_ty!(ServiceWorkerMessage, "message", sys::MessageEvent);

/// A pointing device button (usually a mouse) is pressed on an element. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mousedown
event_ty!(MouseDown, "mousedown", sys::MouseEvent);

/// A pointing device is moved onto the element that has the listener attached.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseenter
event_ty!(MouseEnter, "mouseenter", sys::MouseEvent);

/// A pointing device is moved off the element that has the listener attached.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseleave
event_ty!(MouseLeave, "mouseleave", sys::MouseEvent);

/// A pointing device is moved over an element. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mousemove
event_ty!(MouseMove, "mousemove", sys::MouseEvent);

/// A pointing device is moved off the element that has the listener attached or off one of its
/// children. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseout
event_ty!(MouseOut, "mouseout", sys::MouseEvent);

/// A pointing device is moved onto the element that has the listener attached or onto one of its
/// children. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseover
event_ty!(MouseOver, "mouseover", sys::MouseEvent);

/// A pointing device button is released over an element. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseup
event_ty!(MouseUp, "mouseup", sys::MouseEvent);

/// The speech recognition service returns a final result with no significant recognition.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/nomatch
event_ty!(
    SpeechRecognitionNoMatch,
    "nomatch",
    sys::SpeechRecognitionEvent
);

/// A system notification spawned by [ServiceWorkerRegistration.showNotification()][notif] has been
/// clicked. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/notificationclick
/// [notif]: https://developer.mozilla.org/en-US/docs/Web/API/ServiceWorkerRegistration/showNotification
event_ty!(
    NotificationClick,
    "notificationclick",
    sys::NotificationEvent
);

/// The browser has lost access to the network. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/offline
event_ty!(Offline, "offline", sys::Event);

/// The browser has gained access to the network (but particular websites might be unreachable).
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/online
event_ty!(Online, "online", sys::Event);

/// A WebSocket connection has been established. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/open_websocket
event_ty!(WebsocketOpen, "open", sys::Event);

/// An event source connection has been established. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/open_serversentevents
event_ty!(EventSourceOpen, "open", sys::Event);

/// The orientation of the device (portrait/landscape) has changed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/orientationchange
event_ty!(OrientationChange, "orientationchange", sys::Event);

/// A session history entry is being traversed from. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pagehide
event_ty!(PageHide, "pagehide", sys::PageTransitionEvent);

/// A session history entry is being traversed to. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pageshow
event_ty!(PageShow, "pageshow", sys::PageTransitionEvent);

/// Data has been transferred from the system clipboard to the document. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/paste
event_ty!(Paste, "paste", sys::ClipboardEvent);

/// Playback has been paused. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pause
event_ty!(Pause, "pause", sys::Event);

/// The utterance is paused part way through. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pause_(SpeechSynthesis)
event_ty!(SpeechPause, "pause", sys::SpeechSynthesisEvent);

/// The pointer is unlikely to produce any more events. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointercancel
event_ty!(PointerCancel, "pointercancel", sys::PointerEvent);

/// The pointer enters the active buttons state. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerdown
event_ty!(PointerDown, "pointerdown", sys::PointerEvent);

/// Pointing device is moved inside the hit-testing boundary. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerenter
event_ty!(PointerEnter, "pointerenter", sys::PointerEvent);

/// Pointing device is moved out of the hit-testing boundary. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerleave
event_ty!(PointerLeave, "pointerleave", sys::PointerEvent);

/// The pointer was locked or released. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerlockchange
event_ty!(PointerLockChange, "pointerlockchange", sys::Event);

/// It was impossible to lock the pointer for technical reasons or because the permission was
/// denied. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerlockerror
event_ty!(PointerLockError, "pointerlockerror", sys::Event);

/// The pointer changed coordinates. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointermove
event_ty!(PointerMove, "pointermove", sys::PointerEvent);

/// The pointing device moved out of hit-testing boundary or leaves detectable hover range.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerout
event_ty!(PointerOut, "pointerout", sys::PointerEvent);

/// The pointing device is moved into the hit-testing boundary. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerover
event_ty!(PointerOver, "pointerover", sys::PointerEvent);

/// The pointer leaves the active buttons state. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerup
event_ty!(PointerUp, "pointerup", sys::PointerEvent);

/// Playback has begun. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/play
event_ty!(Play, "play", sys::Event);

/// Playback is ready to start after having been paused or delayed due to lack of data.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/playing
event_ty!(Playing, "playing", sys::Event);

/// A session history entry is being navigated to (in certain cases). [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/popstate
event_ty!(PopState, "popstate", sys::PopStateEvent);

/// In progress. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/progress
event_ty!(Progress, "progress", sys::ProgressEvent);

/// A [Service Worker] has received a push message. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/push
/// [Service Worker]: https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API
event_ty!(Push, "push", sys::PushEvent);

/// A [PushSubscription] has expired. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pushsubscriptionchange
/// [PushSubscription]: https://developer.mozilla.org/en-US/docs/Web/API/PushSubscription
event_ty!(
    PushSubscriptionChange,
    "pushsubscriptionchange",
    sys::PushEvent
);

/// The playback rate has changed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/ratechange
event_ty!(PlaybackRateChange, "ratechange", sys::Event);

/// The readyState attribute of a document has changed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/readystatechange
event_ty!(ReadyStateChange, "readystatechange", sys::Event);

/// A [SMIL] animation element is repeated. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/repeatEvent
/// [SMIL]: https://developer.mozilla.org/en-US/docs/SVG/SVG_animation_with_SMIL
event_ty!(AnimationRepeat, "repeatEvent", sys::TimeEvent);

/// A form is reset. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/reset
event_ty!(FormReset, "reset", sys::Event);

/// The document view has been resized. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/resize
event_ty!(ViewResize, "resize", sys::UiEvent);

/// The browser's resource timing buffer is full. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/resourcetimingbufferfull
event_ty!(TimingBufferFull, "resourcetimingbufferfull", sys::Event);

/// The speech recognition service returns a result — a word or phrase has been positively
/// recognized and this has been communicated back to the app. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/result
event_ty!(
    SpeechRecognitionResult,
    "result",
    sys::SpeechRecognitionEvent
);

/// A paused utterance is resumed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/resume
event_ty!(SpeechResume, "resume", sys::SpeechSynthesisEvent);

/// The document view or an element has been scrolled. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/scroll
event_ty!(Scroll, "scroll", sys::UiEvent);

/// A <em>seek</em> operation completed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/seeked
event_ty!(Seeked, "seeked", sys::Event);

/// A <em>seek</em> operation began. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/seeking
event_ty!(Seeking, "seeking", sys::Event);

/// Some text is being selected. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/select
event_ty!(Select, "select", sys::UiEvent);

/// A selection just started. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/selectstart
event_ty!(SelectionStart, "selectstart", sys::Event);

/// The selection in the document has been changed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/selectionchange
event_ty!(SelectionChange, "selectionchange", sys::Event);

/// A contextmenu event was fired on/bubbled to an element that has a [contextmenu] attribute.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/show
/// [contextmenu]: https://developer.mozilla.org/en-US/docs/DOM/element.contextmenu
event_ty!(ContextMenuShow, "show", sys::MouseEvent);

/// The node contents of a [`<slot>`][slot] have changed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/slotchange
/// [slot]: https://developer.mozilla.org/en-US/docs/Web/API/HTMLSlotElement
event_ty!(SlotChange, "slotchange", sys::Event);

/// Any sound — recognisable speech or not — has stopped being detected. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/soundend
event_ty!(SoundEnd, "sounded", sys::Event);

/// Any sound — recognisable speech or not — has been detected. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/soundstart
event_ty!(SoundStart, "soundstart", sys::Event);

/// Speech recognised by the speech recognition service has stopped being detected.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/speechend
event_ty!(SpeechEnd, "speechend", sys::Event);

/// The user agent is trying to fetch media data, but data is unexpectedly not forthcoming.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/stalled
event_ty!(Stalled, "stalled", sys::Event);

/// The speech recognition service has begun listening to incoming audio with intent to recognize
/// grammars associated with the current SpeechRecognition. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/start_(SpeechRecognition)
event_ty!(SpeechRecognitionStart, "start", sys::Event);

/// Sound that is recognised by the speech recognition service as speech has been detected.
/// [MDN documentation][mdn]
///
/// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/speechstart
event_ty!(SpeechRecognized, "speechstart", sys::Event);

/// A storage area ([localStorage] or [sessionStorage]) has changed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/storage
/// [localStorage]: https://developer.mozilla.org/en-US/docs/DOM/Storage#localStorage
/// [sessionStorage]: https://developer.mozilla.org/en-US/docs/DOM/Storage#sessionStorage
event_ty!(Storage, "storage", sys::StorageEvent);

/// A form is submitted. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/submit
event_ty!(Submit, "submit", sys::Event);

/// A request successfully completed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/success_indexedDB
event_ty!(Success, "success", sys::Event);

/// Media data loading has been suspended. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/suspend
event_ty!(Suspend, "suspend", sys::Event);

/// Page loading has been stopped before the [SVG] was loaded. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGAbort
/// [SVG]: https://developer.mozilla.org/en-US/docs/SVG
event_ty!(SvgAbort, "SVGAbort", sys::Event);

/// An error has occurred before the [SVG] was loaded. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGError
/// [SVG]: https://developer.mozilla.org/en-US/docs/SVG
event_ty!(SvgError, "SVGError", sys::Event);

/// An [SVG] document has been loaded and parsed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGLoad
/// [SVG]: https://developer.mozilla.org/en-US/docs/SVG
event_ty!(SvgLoad, "SVGLoad", sys::Event);

/// An [SVG] document is being resized. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGResize
/// [SVG]: https://developer.mozilla.org/en-US/docs/SVG
event_ty!(SvgResize, "SVGResize", sys::Event);

/// An [SVG] document is being scrolled. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGScroll
/// [SVG]: https://developer.mozilla.org/en-US/docs/SVG
event_ty!(SvgScroll, "SVGScroll", sys::Event);

/// An [SVG] document has been removed from a window or frame. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGUnload
/// [SVG]: https://developer.mozilla.org/en-US/docs/SVG
event_ty!(SvgUnload, "SVGUnload", sys::Event);

/// An [SVG] document is being zoomed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGZoom
/// [SVG]: https://developer.mozilla.org/en-US/docs/SVG
event_ty!(SvgZoom, "SVGZoom", sys::Event);

/// The timeout event is fired when progression is terminated due to preset time expiring.
/// [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/timeout
event_ty!(Timeout, "timeout", sys::ProgressEvent);

/// The time indicated by the currentTime attribute has been updated. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/timeupdate
event_ty!(TimeUpdate, "timeupdate", sys::Event);

/// A touch point has been disrupted in an implementation-specific manners (too many touch points
/// for example). [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/touchcancel
event_ty!(TouchCancel, "touchcancel", sys::TouchEvent);

/// A touch point is removed from the touch surface. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/touchend
event_ty!(TouchEnd, "touchend", sys::TouchEvent);

/// A touch point is moved along the touch surface. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/touchmove
event_ty!(TouchMove, "touchmove", sys::TouchEvent);

/// A touch point is placed on the touch surface. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/touchstart
event_ty!(TouchStart, "touchstart", sys::TouchEvent);

/// The document or a dependent resource is being unloaded. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/unload
event_ty!(Unload, "unload", sys::UiEvent);

/// The content of a tab has become visible or has been hidden. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/visibilitychange
event_ty!(VisibilityChange, "visibilitychange", sys::Event);

/// The volume has changed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/volumechange
event_ty!(VolumeChange, "volumechange", sys::Event);

/// Playback has stopped because of a temporary lack of data. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/waiting
event_ty!(Waiting, "waiting", sys::Event);

/// A wheel button of a pointing device is rotated in any direction. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/wheel
event_ty!(Wheel, "wheel", sys::WheelEvent);

/// A [CSS transition] has completed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/transitionend
/// [CSS transition]: https://developer.mozilla.org/en-US/docs/CSS/CSS_transitions
event_ty!(TransitionEnd, "transitionend", sys::TransitionEvent);

/// An attempt was made to open a database with a version number higher than its current version. A
/// versionchange transaction has been created. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/upgradeneeded_indexedDB
event_ty!(UpgradeNeeded, "upgradeneeded", sys::Event);

/// Fresh data is available from a proximity sensor (indicates whether the nearby object is near the
/// device or not). [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/userproximity
event_ty!(UserProximity, "userproximity", sys::UserProximityEvent);

/// Fires when the list of SpeechSynthesisVoice objects that would be returned by the
/// SpeechSynthesis.getVoices() method has changed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/voiceschanged
event_ty!(VoicesChanged, "voiceschanged", sys::Event);

/// A versionchange transaction completed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/versionchange_indexedDB
event_ty!(TransactionVersionChange, "versionchange", sys::Event);

/// A transaction successfully completed. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/complete_indexedDB
event_ty!(TransactionComplete, "complete", sys::Event);

/// The utterance has begun to be spoken. [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/start_(SpeechSynthesis)
event_ty!(SpeechStart, "start", sys::SpeechSynthesisEvent);
