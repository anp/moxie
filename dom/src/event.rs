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

/// The loading of a resource has been aborted.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/abort
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/UIEvent
event_ty!(Abort, "abort", sys::UiEvent);

/// Progression has been terminated (not due to an error).
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/abort_(ProgressEvent)
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/ProgressEvent
event_ty!(AbortProgress, "abort", sys::ProgressEvent);

/// A transaction has been aborted.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/abort_indexedDB
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(AbortTransaction, "abort", sys::Event);

/// The associated document has started printing or the print preview has been closed.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/afterprint
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(AfterPrint, "afterprint", sys::Event);

/// A [CSS animation] has aborted.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/animationcancel
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/AnimationEvent
/// [CSS animation]: https://developer.mozilla.org/en-US/docs/CSS/CSS_animations
/// [w3css]: http://www.w3.org/TR/css3-animations/#animation-events
event_ty!(AnimationCancel, "animationcancel", sys::AnimationEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/animationend"></a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/AnimationEvent"> </i></span>
// <a href="http://www.w3.org/TR/css3-animations/#animation-events">CSS Animations</a>
// A <a href="https://developer.mozilla.org/en-US/docs/CSS/CSS_animations">CSS animation</a> has completed.
event_ty!(AnimationEnd, "animationend", sys::AnimationEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/animationiteration"></a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/AnimationEvent"> </i></span>
// <a href="http://www.w3.org/TR/css3-animations/#animation-events">CSS Animations</a>
// A <a href="https://developer.mozilla.org/en-US/docs/CSS/CSS_animations">CSS animation</a> is repeated.
event_ty!(
    AnimationIteration,
    "animationiteration",
    sys::AnimationEvent
);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/animationstart"></a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/AnimationEvent"> </i></span>
// <a href="http://www.w3.org/TR/css3-animations/#animation-events">CSS Animations</a>
// A <a href="https://developer.mozilla.org/en-US/docs/CSS/CSS_animations">CSS animation</a> has started.
event_ty!(AnimationStart, "animationstart", sys::AnimationEvent);

/// A web application is successfully installed as a <a href="/en-US/Apps/Progressive">progressive web app</a>.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/appinstalled
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(AppInstalled, "appinstalled", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/audioprocess">audioprocess</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/AudioProcessingEvent"> </i></span>
// <a href="https://webaudio.github.io/web-audio-api/#AudioProcessingEvent" hreflang="en" lang="en">The definition of 'audioprocess' in that specification.</small></a>
// The input buffer of a <a href="https://developer.mozilla.org/en-US/docs/Web/API/ScriptProcessorNode">ScriptProcessorNode</a> is ready to be processed.
event_ty!(AudioProcess, "audioprocess", sys::AudioProcessingEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/audioend"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/Event">Event</a>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// The user agent has finished capturing audio for speech recognition.
event_ty!(AudioEnd, "audioend", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/audiostart"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/Event">Event</a>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// The user agent has started to capture audio for speech recognition.
event_ty!(AudioStart, "audiostart", sys::Event);

/// The associated document is about to be printed or previewed for printing.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/beforeprint
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(BeforePrint, "beforeprint", sys::Event);

/// The window, the document and its resources are about to be unloaded.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/beforeunload
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/BeforeUnloadEvent
event_ty!(BeforeUnload, "beforeunload", sys::BeforeUnloadEvent);

/// A <a href="https://developer.mozilla.org/en-US/docs/SVG/SVG_animation_with_SMIL">SMIL</a> animation element begins.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/beginEvent
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/TimeEvent
event_ty!(SvgAnimationBegin, "beginEvent", sys::TimeEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Reference/Events/blocked_indexedDB">blocked</a>
// &nbsp;
// <a href="http://www.w3.org/TR/IndexedDB/#request-api">IndexedDB</a>
// An open connection to a database is blocking a versionchange transaction on the same database.
event_ty!(ConnectionBlocked, "blocked", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/blur">blur</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/FocusEvent"> </i></span>
// <a href="http://www.w3.org/TR/DOM-Level-3-Events/#event-type-blur">DOM L3</a>
// An element has lost focus (does not bubble).
event_ty!(Blur, "blur", sys::FocusEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/boundary"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/SpeechSynthesisEvent">SpeechSynthesisEvent</a>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// The spoken utterance reaches a word or sentence boundary
event_ty!(SpeechBoundary, "boundary", sys::SpeechSynthesisEvent);

/// The user agent can play the media, but estimates that not enough data has been loaded to play the media up to its end without having to stop for further buffering of content.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/canplay
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(CanPlay, "canplay", sys::Event);

/// The user agent can play the media up to its end without having to stop for further buffering of content.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/canplaythrough
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(CanPlayThrough, "canplaythrough", sys::Event);

/// The change event is fired for <a href="https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input">&lt;textarea&gt;</a> elements when a change to the element's value is committed by the user.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/change
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Change, "change", sys::Event);

/// The battery begins or stops charging.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/chargingchange
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(ChargingChange, "chargingchange", sys::Event);

/// The chargingTime attribute has been updated.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/chargingtimechange
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(ChargingTime, "chargingtimechange", sys::Event);

/// A pointing device button has been pressed and released on an element.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/click
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent
event_ty!(Click, "click", sys::MouseEvent);

/// A WebSocket connection has been closed.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/close_websocket
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(CloseWebsocket, "close", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/complete">complete</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/OfflineAudioCompletionEvent"> </i></span>
// <a href="https://webaudio.github.io/web-audio-api/#OfflineAudioCompletionEvent-section" hreflang="en" lang="en">The definition of 'OfflineAudioCompletionEvent' in that specification.</small></a>
// The rendering of an <a href="https://developer.mozilla.org/en-US/docs/Web/API/OfflineAudioContext">OfflineAudioContext</a> is terminated.
event_ty!(AudioComplete, "complete", sys::OfflineAudioCompletionEvent);

/// The composition of a passage of text has been completed or canceled.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/compositionend
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/CompositionEvent
event_ty!(CompositionEnd, "compositionend", sys::CompositionEvent);

/// The composition of a passage of text is prepared (similar to keydown for a keyboard input, but works with other inputs such as speech recognition).
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/compositionstart
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/CompositionEvent
event_ty!(CompositionStart, "compositionstart", sys::CompositionEvent);

/// A character is added to a passage of text being composed.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/compositionupdate
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/CompositionEvent
event_ty!(
    CompositionUpdate,
    "compositionupdate",
    sys::CompositionEvent
);

/// The right button of the mouse is clicked (before the context menu is displayed).
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/contextmenu
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent
event_ty!(ContextMenu, "contextmenu", sys::MouseEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/copy">copy</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/ClipboardEvent"> </i></span>
// <a href="http://www.w3.org/TR/clipboard-apis/#copy-event">Clipboard</a>
// The text selection has been added to the clipboard.
event_ty!(Copy, "copy", sys::ClipboardEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/cut">cut</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/ClipboardEvent"> </i></span>
// <a href="http://www.w3.org/TR/clipboard-apis/#cut-event">Clipboard</a>
// The text selection has been removed from the document and added to the clipboard.
event_ty!(Cut, "cut", sys::ClipboardEvent);

/// A pointing device button is clicked twice on an element.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dblclick
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent
event_ty!(DoubleClick, "dblclick", sys::MouseEvent);

/// A media device such as a camera, microphone, or speaker is connected or removed from the system.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/devicechange
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(DeviceChange, "devicechange", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/devicemotion">devicemotion</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/DeviceMotionEvent"> </i></span>
// <a href="http://dev.w3.org/geo/api/spec-source-orientation.html" lang="en">Device Orientation Events</a>
// Fresh data is available from a motion sensor.
event_ty!(DeviceMotion, "devicemotion", sys::DeviceMotionEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/deviceorientation">deviceorientation</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/DeviceOrientationEvent"> </i></span>
// <a href="http://dev.w3.org/geo/api/spec-source-orientation.html" lang="en">Device Orientation Events</a>
// Fresh data is available from an orientation sensor.
event_ty!(
    DeviceOrientation,
    "deviceorientation",
    sys::DeviceOrientationEvent
);

/// The dischargingTime attribute has been updated.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dischargingtimechange
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(DischargingTime, "dischargingtimechange", sys::Event);

/// The document has finished loading (but not its dependent resources).
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/DOMContentLoaded
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(DomContentLoaded, "DOMContentLoaded", sys::Event);

/// An element or text selection is being dragged (every 350ms).
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/drag
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/DragEvent
event_ty!(Drag, "drag", sys::DragEvent);

/// A drag operation is being ended (by releasing a mouse button or hitting the escape key).
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragend
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/DragEvent
event_ty!(DragEnd, "dragend", sys::DragEvent);

/// A dragged element or text selection enters a valid drop target.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragenter
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/DragEvent
event_ty!(DragEnter, "dragenter", sys::DragEvent);

/// A dragged element or text selection leaves a valid drop target.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragleave
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/DragEvent
event_ty!(DragLeave, "dragleave", sys::DragEvent);

/// An element or text selection is being dragged over a valid drop target (every 350ms).
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragover
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/DragEvent
event_ty!(DragOver, "dragover", sys::DragEvent);

/// The user starts dragging an element or text selection.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/dragstart
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/DragEvent
event_ty!(DragStart, "dragstart", sys::DragEvent);

/// An element is dropped on a valid drop target.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/drop
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/DragEvent
event_ty!(Dropped, "drop", sys::DragEvent);

/// The duration attribute has been updated.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/durationchange
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(DurationChange, "durationchange", sys::Event);

/// The media has become empty; for example, this event is sent if the media has already been loaded (or partially loaded), and the <a href="https://developer.mozilla.org/en-US/docs/XPCOM_Interface_Reference/NsIDOMHTMLMediaElement">load()</a> method is called to reload it.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/emptied
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Emptied, "emptied", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/end_(SpeechRecognition)"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/Event">Event</a>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// The speech recognition service has disconnected.
event_ty!(SpeechRecognitionEnd, "end", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/end_(SpeechSynthesis)"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/SpeechSynthesisEvent">SpeechSynthesisEvent</a>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// The utterance has finished being spoken.
event_ty!(SpeechSynthesisEnd, "end", sys::SpeechSynthesisEvent);

/// Playback has stopped because the end of the media was reached.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/ended
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(PlaybackEnded, "ended", sys::Event);

/// Playback has stopped because the end of the media was reached.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/ended_(Web_Audio)
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(AudioEnded, "ended", sys::Event);

/// A <a href="https://developer.mozilla.org/en-US/docs/SVG/SVG_animation_with_SMIL">SMIL</a> animation element ends.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/endEvent
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/TimeEvent
event_ty!(SvgAnimationEnd, "endEvent", sys::TimeEvent);

/// A resource failed to load.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/UIEvent
event_ty!(ResourceError, "error", sys::UiEvent);

/// Progression has failed.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/ProgressEvent
event_ty!(ProgressError, "error", sys::ProgressEvent);

/// A WebSocket connection has been closed with prejudice (some data couldn't be sent for example).
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(WebsocketError, "error", sys::Event);

/// An event source connection has been failed.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(EventSourceError, "error", sys::Event);

/// A request caused an error and failed.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(RequestError, "error", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/error_(SpeechRecognitionError)"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/Event">Event</a>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// A speech recognition error occurs.
event_ty!(SpeechRecognitionError, "error", sys::Event);

/// An error occurs that prevents the utterance from being successfully spoken.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/error_(SpeechSynthesisError)
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/SpeechSynthesisErrorEvent
event_ty!(SpeechError, "error", sys::SpeechSynthesisErrorEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/focus">focus</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/FocusEvent"> </i></span>
// <a href="http://www.w3.org/TR/DOM-Level-3-Events/#event-type-focus">DOM L3</a>
// An element has received focus (does not bubble).
event_ty!(Focus, "focus", sys::FocusEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/focusin">focusin</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/FocusEvent"> </i></span>
// <a href="http://www.w3.org/TR/DOM-Level-3-Events/#event-type-focusIn">DOM L3</a>
// An element is about to receive focus (bubbles).
event_ty!(FocusIn, "focusin", sys::FocusEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/focusout">focusout</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/FocusEvent"> </i></span>
// <a href="http://www.w3.org/TR/DOM-Level-3-Events/#event-type-focusout">DOM L3</a>
// An element is about to lose focus (bubbles).
event_ty!(FocusOut, "focusout", sys::FocusEvent);

/// An element was turned to fullscreen mode or back to normal mode.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/fullscreenchange
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(FullscreenChange, "fullscreenchange", sys::Event);

/// It was impossible to switch to fullscreen mode for technical reasons or because the permission was denied.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/fullscreenerror
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(FullscreenError, "fullscreenerror", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/gamepadconnected">gamepadconnected</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/GamepadEvent"> </i></span>
// <a href="http://www.w3.org/TR/gamepad/#the-gamepadconnected-event">Gamepad</a>
// A gamepad has been connected.
event_ty!(GamepadConnected, "gamepadconnected", sys::GamepadEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/gamepaddisconnected">gamepaddisconnected</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/GamepadEvent"> </i></span>
// <a href="http://www.w3.org/TR/gamepad/#the-gamepaddisconnected-event">Gamepad</a>
// A gamepad has been disconnected.
event_ty!(
    GamepadDisconnected,
    "gamepaddisconnected",
    sys::GamepadEvent
);

/// Element receives pointer capture.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/gotpointercapture
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent
event_ty!(GotPointerCapture, "gotpointercapture", sys::PointerEvent);

/// The fragment identifier of the URL has changed (the part of the URL after the #).
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/hashchange
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/HashChangeEvent
event_ty!(HashChange, "hashchange", sys::HashChangeEvent);

/// Element lost pointer capture.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/lostpointercapture
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent
event_ty!(LostPointerCapture, "lostpointercapture", sys::PointerEvent);

/// The value of an element changes or the content of an element with the attribute <a href="https://developer.mozilla.org/en-US/docs/DOM/Element.contentEditable">contenteditable</a> is modified.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/input
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Input, "input", sys::Event);

/// A submittable element has been checked and doesn't satisfy its constraints.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/invalid
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Invalid, "invalid", sys::Event);

/// A key is pressed down.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/keydown
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent
event_ty!(KeyDown, "keydown", sys::KeyboardEvent);

/// A key is released.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/keyup
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent
event_ty!(KeyUp, "keyup", sys::KeyboardEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/languagechange"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/Event">Event</a>
// <a href="https://www.w3.org/TR/html51/#dom-navigator-languages" hreflang="en" lang="en">The definition of 'NavigatorLanguage.languages' in that specification.</small></a>
// The user's preferred languages have changed.
event_ty!(LanguageChange, "languagechange", sys::Event);

/// The level attribute has been updated.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/levelchange
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(LevelChange, "levelchange", sys::Event);

/// A resource and its dependent resources have finished loading.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/load
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/UIEvent
event_ty!(ResourceLoad, "load", sys::UiEvent);

/// Progression has been successful.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/load_(ProgressEvent)
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/ProgressEvent
event_ty!(ProgressLoad, "load", sys::ProgressEvent);

/// The first frame of the media has finished loading.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/loadeddata
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(DataLoaded, "loadeddata", sys::Event);

/// The metadata has been loaded.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/loadedmetadata
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(MetadataLoaded, "loadedmetadata", sys::Event);

/// Progress has stopped (after "error", "abort" or "load" have been dispatched).
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/loadend
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/ProgressEvent
event_ty!(LoadEnd, "loadend", sys::ProgressEvent);

/// Progress has begun.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/loadstart
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/ProgressEvent
event_ty!(LoadStart, "loadstart", sys::ProgressEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/mark"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/SpeechSynthesisEvent">SpeechSynthesisEvent</a>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// The spoken utterance reaches a named SSML "mark" tag.
event_ty!(SpeechMark, "mark", sys::SpeechSynthesisEvent);

/// A message is received through a WebSocket.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/message_websocket
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MessageEvent
event_ty!(WebsocketMessage, "message", sys::MessageEvent);

/// A message is received from a Web Worker.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/message_webworker
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MessageEvent
event_ty!(WorkerMessage, "message", sys::MessageEvent);

/// A message is received from a child (i)frame or a parent window.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/message_webmessaging
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MessageEvent
event_ty!(ChildMessage, "message", sys::MessageEvent);

/// A message is received through an event source.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/message_serversentevents
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MessageEvent
event_ty!(EventSourceMessage, "message", sys::MessageEvent);

/// A message error is raised when a message is received by an object.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/messageerror
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MessageEvent
event_ty!(MessageError, "messageerror", sys::MessageEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/message_(ServiceWorker)"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/ServiceWorkerMessageEvent">ExtendableMessageEvent</a>, depending on context.
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API">Service Workers</a>
// A message is received from a service worker, or a message is received in a service worker from another context.
event_ty!(ServiceWorkerMessage, "message", sys::MessageEvent);

/// A pointing device button (usually a mouse) is pressed on an element.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mousedown
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent
event_ty!(MouseDown, "mousedown", sys::MouseEvent);

/// A pointing device is moved onto the element that has the listener attached.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseenter
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent
event_ty!(MouseEnter, "mouseenter", sys::MouseEvent);

/// A pointing device is moved off the element that has the listener attached.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseleave
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent
event_ty!(MouseLeave, "mouseleave", sys::MouseEvent);

/// A pointing device is moved over an element.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mousemove
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent
event_ty!(MouseMove, "mousemove", sys::MouseEvent);

/// A pointing device is moved off the element that has the listener attached or off one of its children.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseout
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent
event_ty!(MouseOut, "mouseout", sys::MouseEvent);

/// A pointing device is moved onto the element that has the listener attached or onto one of its children.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseover
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent
event_ty!(MouseOver, "mouseover", sys::MouseEvent);

/// A pointing device button is released over an element.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/mouseup
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent
event_ty!(MouseUp, "mouseup", sys::MouseEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/nomatch"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/SpeechRecognitionEvent">SpeechRecognitionEvent</a>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// The speech recognition service returns a final result with no significant recognition.
event_ty!(
    SpeechRecognitionNoMatch,
    "nomatch",
    sys::SpeechRecognitionEvent
);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/notificationclick">notificationclick</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/NotificationEvent"> </i></span>
// <a href="https://notifications.spec.whatwg.org/#dom-serviceworkerglobalscope-onnotificationclick" hreflang="en" lang="en">The definition of 'onnotificationclick' in that specification.</small></a>
// A system notification<span style="line-height: 19.0909080505371px;"> spawned by <a href="https://developer.mozilla.org/en-US/docs/Web/API/ServiceWorkerRegistration/showNotification">ServiceWorkerRegistration.showNotification()</a> has been clicked.</span>
event_ty!(
    NotificationClick,
    "notificationclick",
    sys::NotificationEvent
);

/// The browser has lost access to the network.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/offline
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Offline, "offline", sys::Event);

/// The browser has gained access to the network (but particular websites might be unreachable).
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/online
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Online, "online", sys::Event);

/// A WebSocket connection has been established.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/open_websocket
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(WebsocketOpen, "open", sys::Event);

/// An event source connection has been established.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/open_serversentevents
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(EventSourceOpen, "open", sys::Event);

/// The orientation of the device (portrait/landscape) has changed
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/orientationchange
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(OrientationChange, "orientationchange", sys::Event);

/// A session history entry is being traversed from.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pagehide
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/PageTransitionEvent
event_ty!(PageHide, "pagehide", sys::PageTransitionEvent);

/// A session history entry is being traversed to.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pageshow
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/PageTransitionEvent
event_ty!(PageShow, "pageshow", sys::PageTransitionEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/paste">paste</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/ClipboardEvent"> </i></span>
// <a href="http://www.w3.org/TR/clipboard-apis/#paste-event">Clipboard</a>
// Data has been transferred from the system clipboard to the document.
event_ty!(Paste, "paste", sys::ClipboardEvent);

/// Playback has been paused.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pause
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Pause, "pause", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/pause_(SpeechSynthesis)"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/SpeechSynthesisEvent">SpeechSynthesisEvent</a>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// The utterance is paused part way through.
event_ty!(SpeechPause, "pause", sys::SpeechSynthesisEvent);

/// The pointer is unlikely to produce any more events.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointercancel
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent
event_ty!(PointerCancel, "pointercancel", sys::PointerEvent);

/// The pointer enters the active buttons state.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerdown
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent
event_ty!(PointerDown, "pointerdown", sys::PointerEvent);

/// Pointing device is moved inside the hit-testing boundary.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerenter
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent
event_ty!(PointerEnter, "pointerenter", sys::PointerEvent);

/// Pointing device is moved out of the hit-testing boundary.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerleave
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent
event_ty!(PointerLeave, "pointerleave", sys::PointerEvent);

/// The pointer was locked or released.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerlockchange
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(PointerLockChange, "pointerlockchange", sys::Event);

/// It was impossible to lock the pointer for technical reasons or because the permission was denied.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerlockerror
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(PointerLockError, "pointerlockerror", sys::Event);

/// The pointer changed coordinates.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointermove
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent
event_ty!(PointerMove, "pointermove", sys::PointerEvent);

/// The pointing device moved out of hit-testing boundary or leaves detectable hover range.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerout
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent
event_ty!(PointerOut, "pointerout", sys::PointerEvent);

/// The pointing device is moved into the hit-testing boundary.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerover
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent
event_ty!(PointerOver, "pointerover", sys::PointerEvent);

/// The pointer leaves the active buttons state.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/pointerup
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent
event_ty!(PointerUp, "pointerup", sys::PointerEvent);

/// Playback has begun.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/play
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Play, "play", sys::Event);

/// Playback is ready to start after having been paused or delayed due to lack of data.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/playing
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Playing, "playing", sys::Event);

/// A session history entry is being navigated to (in certain cases).
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/popstate
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/PopStateEvent
event_ty!(PopState, "popstate", sys::PopStateEvent);

/// In progress.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/progress
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/ProgressEvent
event_ty!(Progress, "progress", sys::ProgressEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/push">push</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/PushEvent"> </i></span>
// <a href="https://w3c.github.io/push-api/" hreflang="en" lang="en">Push API</a>
// A <a href="https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API">Service Worker</a> has received a push message.
event_ty!(Push, "push", sys::PushEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/pushsubscriptionchange">pushsubscriptionchange</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/PushEvent"> </i></span>
// <a href="https://w3c.github.io/push-api/" hreflang="en" lang="en">Push API</a>
// A <a href="https://developer.mozilla.org/en-US/docs/Web/API/PushSubscription">PushSubscription</a> has expired.
event_ty!(
    PushSubscriptionChange,
    "pushsubscriptionchange",
    sys::PushEvent
);

/// The playback rate has changed.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/ratechange
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(PlaybackRateChange, "ratechange", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/readystatechange">readystatechange</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/Event">Event</a>
// <span>HTML5 <span>and </span></span><a href="http://www.w3.org/TR/XMLHttpRequest/#event-xhr-readystatechange">XMLHttpRequest</a>
// The readyState attribute of a document has changed.
event_ty!(ReadyStateChange, "readystatechange", sys::Event);

/// A <a href="https://developer.mozilla.org/en-US/docs/SVG/SVG_animation_with_SMIL">SMIL</a> animation element is repeated.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/repeatEvent
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/TimeEvent
event_ty!(AnimationRepeat, "repeatEvent", sys::TimeEvent);

/// A form is reset.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/reset
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(FormReset, "reset", sys::Event);

/// The document view has been resized.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/resize
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/UIEvent
event_ty!(ViewResize, "resize", sys::UiEvent);

/// The browser's resource timing buffer is full.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/resourcetimingbufferfull
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Performance
event_ty!(TimingBufferFull, "resourcetimingbufferfull", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/result"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/SpeechRecognitionEvent"> </i></span>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// The speech recognition service returns a result — a word or phrase has been positively recognized and this has been communicated back to the app.
event_ty!(
    SpeechRecognitionResult,
    "result",
    sys::SpeechRecognitionEvent
);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/resume"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/SpeechSynthesisEvent"> </i></span>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// A paused utterance is resumed.
event_ty!(SpeechResume, "resume", sys::SpeechSynthesisEvent);

/// The document view or an element has been scrolled.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/scroll
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/UIEvent
event_ty!(Scroll, "scroll", sys::UiEvent);

/// A <em>seek</em> operation completed.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/seeked
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Seeked, "seeked", sys::Event);

/// A <em>seek</em> operation began.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/seeking
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Seeking, "seeking", sys::Event);

/// Some text is being selected.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/select
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/UIEvent
event_ty!(Select, "select", sys::UiEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/selectstart"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/Event">Event</a>
// <a href="https://w3c.github.io/selection-api/" hreflang="en" lang="en">Selection API</a>
// A selection just started.
event_ty!(SelectionStart, "selectstart", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/selectionchange"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/Event">Event</a>
// <a href="https://w3c.github.io/selection-api/" hreflang="en" lang="en">Selection API</a>
// The selection in the document has been changed.
event_ty!(SelectionChange, "selectionchange", sys::Event);

/// A contextmenu event was fired on/bubbled to an element that has a <a href="https://developer.mozilla.org/en-US/docs/DOM/element.contextmenu">contextmenu</a> attribute
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/show
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent
event_ty!(ContextMenuShow, "show", sys::MouseEvent);

/// The node contents of a <a href="https://developer.mozilla.org/en-US/docs/Web/API/HTMLSlotElement">&lt;slot&gt;</a>) have changed.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/slotchange
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(SlotChange, "slotchange", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/soundend"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/Event">Event</a>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// Any sound — recognisable speech or not — has stopped being detected.
event_ty!(SoundEnd, "sounded", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/soundstart"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/Event">Event</a>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// Any sound — recognisable speech or not — has been detected.
event_ty!(SoundStart, "soundstart", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/speechend"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/Event">Event</a>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// Speech recognised by the speech recognition service has stopped being detected.
event_ty!(SpeechEnd, "speechend", sys::Event);

// // <a href="https://developer.mozilla.org/en-US/docs/Web/Events/speechstart"> </i></span>
// // <a href="https://developer.mozilla.org/en-US/docs/Web/API/Event">Event</a>
// // <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// // Sound that is recognised by the speech recognition service as speech has been detected.
// event_ty!(SpeechStart, "speechstart", sys::Event);

// /// The utterance has begun to be spoken.
// ///
// /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/start_(SpeechSynthesis)
// /// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/SpeechSynthesisEvent
// event_ty!(SpeechStart, "start", sys::SpeechSynthesisEvent);

/// The user agent is trying to fetch media data, but data is unexpectedly not forthcoming.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/stalled
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Stalled, "stalled", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/start_(SpeechRecognition)"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/Event">Event</a>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// The speech recognition service has begun listening to incoming audio with intent to recognize grammars associated with the current SpeechRecognition.
event_ty!(SpeechRecognitionStart, "start", sys::Event);

/// A storage area (<a href="https://developer.mozilla.org/en-US/docs/DOM/Storage#localStorage">localStorage</a> or <a href="https://developer.mozilla.org/en-US/docs/DOM/Storage#sessionStorage">sessionStorage</a>) has changed.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/storage
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/StorageEvent
event_ty!(Storage, "storage", sys::StorageEvent);

/// A form is submitted.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/submit
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Submit, "submit", sys::Event);

/// A request successfully completed.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Reference/Events/success_indexedDB
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Success, "success", sys::Event);

/// Media data loading has been suspended.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/suspend
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Suspend, "suspend", sys::Event);

/// Page loading has been stopped before the <a href="https://developer.mozilla.org/en-US/docs/SVG">SVG</a> was loaded.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGAbort
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/SVGEvent
event_ty!(SvgAbort, "SVGAbort", sys::Event);

/// An error has occurred before the <a href="https://developer.mozilla.org/en-US/docs/SVG">SVG</a> was loaded.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGError
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/SVGEvent
event_ty!(SvgError, "SVGError", sys::Event);

/// An <a href="https://developer.mozilla.org/en-US/docs/SVG">SVG</a> document has been loaded and parsed.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGLoad
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/SVGEvent
event_ty!(SvgLoad, "SVGLoad", sys::Event);

/// An <a href="https://developer.mozilla.org/en-US/docs/SVG">SVG</a> document is being resized.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGResize
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/SVGEvent
event_ty!(SvgResize, "SVGResize", sys::Event);

/// An <a href="https://developer.mozilla.org/en-US/docs/SVG">SVG</a> document is being scrolled.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGScroll
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/SVGEvent
event_ty!(SvgScroll, "SVGScroll", sys::Event);

/// An <a href="https://developer.mozilla.org/en-US/docs/SVG">SVG</a> document has been removed from a window or frame.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGUnload
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/SVGEvent
event_ty!(SvgUnload, "SVGUnload", sys::Event);

/// An <a href="https://developer.mozilla.org/en-US/docs/SVG">SVG</a> document is being zoomed.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/SVGZoom
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/SVGZoomEvent
event_ty!(SvgZoom, "SVGZoom", sys::Event);

/// &nbsp;
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/timeout
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/ProgressEvent
event_ty!(Timeout, "timeout", sys::ProgressEvent);

/// The time indicated by the currentTime attribute has been updated.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/timeupdate
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(TimeUpdate, "timeupdate", sys::Event);

/// A touch point has been disrupted in an implementation-specific manners (too many touch points for example).
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/touchcancel
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/TouchEvent
event_ty!(TouchCancel, "touchcancel", sys::TouchEvent);

/// A touch point is removed from the touch surface.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/touchend
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/TouchEvent
event_ty!(TouchEnd, "touchend", sys::TouchEvent);

/// A touch point is moved along the touch surface.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/touchmove
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/TouchEvent
event_ty!(TouchMove, "touchmove", sys::TouchEvent);

/// A touch point is placed on the touch surface.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/touchstart
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/TouchEvent
event_ty!(TouchStart, "touchstart", sys::TouchEvent);

/// The document or a dependent resource is being unloaded.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/unload
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/UIEvent
event_ty!(Unload, "unload", sys::UiEvent);

/// The content of a tab has become visible or has been hidden.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/visibilitychange
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(VisibilityChange, "visibilitychange", sys::Event);

/// The volume has changed.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/volumechange
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(VolumeChange, "volumechange", sys::Event);

/// Playback has stopped because of a temporary lack of data.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/waiting
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/Event
event_ty!(Waiting, "waiting", sys::Event);

/// A wheel button of a pointing device is rotated in any direction.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/Events/wheel
/// [mdn2]: https://developer.mozilla.org/en-US/docs/Web/API/WheelEvent
event_ty!(Wheel, "wheel", sys::WheelEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/transitionend">transitionend</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/TransitionEvent"> </i></span>
// <a href="http://www.w3.org/TR/css3-transitions/#transition-events">CSS Transitions</a>
// A <a href="https://developer.mozilla.org/en-US/docs/CSS/CSS_transitions">CSS transition</a> has completed.
event_ty!(TransitionEnd, "transitionend", sys::TransitionEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Reference/Events/upgradeneeded_indexedDB">upgradeneeded</a>
// &nbsp;
// <a href="http://www.w3.org/TR/IndexedDB/#request-api">IndexedDB</a>
// An attempt was made to open a database with a version number higher than its current version. A versionchange transaction has been created.
event_ty!(UpgradeNeeded, "upgradeneeded", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/userproximity">userproximity</a>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/UserProximityEvent"> </i></span>
// <a href="https://w3c.github.io/proximity/" hreflang="en" lang="en">Proximity Sensor</a>
// Fresh data is available from a proximity sensor (indicates whether the nearby object is near the device or not).
event_ty!(UserProximity, "userproximity", sys::UserProximityEvent);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Events/voiceschanged"> </i></span>
// <a href="https://developer.mozilla.org/en-US/docs/Web/API/Event">Event</a>
// <a href="https://w3c.github.io/speech-api/" hreflang="en" lang="en">Web Speech API</a>
// The list of <a href="https://developer.mozilla.org/en-US/docs/Web/API/SpeechSynthesisVoice">voiceschanged</a> event fires.)
event_ty!(VoicesChanged, "voiceschanged", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Reference/Events/versionchange_indexedDB">versionchange</a>
// &nbsp;
// <a href="http://www.w3.org/TR/IndexedDB/#database-interface">IndexedDB</a>
// A versionchange transaction completed.
event_ty!(TransactionVersionChange, "versionchange", sys::Event);

// <a href="https://developer.mozilla.org/en-US/docs/Web/Reference/Events/complete_indexedDB">complete</a>
// &nbsp;
// <a href="http://www.w3.org/TR/IndexedDB/#transaction">IndexedDB</a>
// A transaction successfully completed.
event_ty!(TransactionComplete, "complete", sys::Event);
