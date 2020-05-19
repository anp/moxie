//! Trait for defining the methods related to the event handlers shared by all
//! HTML elements.

use crate::prelude::*;
use augdom::event::*;

/// An event which can be handled on any element.
pub trait GlobalEvent: Event + 'static {}

macro_rules! global_events {
    ($($property:ident <- $event:ident,)+) => {

        $(impl GlobalEvent for $event {})+

        /// These event handlers are defined on the [GlobalEventHandlers][mdn] mixin,
        /// and implemented by HTMLElement, Document, Window, as well as by
        /// WorkerGlobalScope for Web Workers.
        ///
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/GlobalEventHandlers
        pub trait GlobalEventHandler:
            HtmlElement
            $(+ EventTarget<$event>)+
        {$(
            /// Set an event handler.
            fn $property(self, callback: impl FnMut(augdom::event::$event) + 'static) -> Self {
                self.on(callback)
            }
        )+}

    };
}

global_events! {
    onabort              <- Abort,
    onblur               <- Blur,
    oncancel             <- Cancel,
    oncanplay            <- CanPlay,
    oncanplaythrough     <- CanPlayThrough,
    onchange             <- Change,
    onclick              <- Click,
    onclose              <- CloseWebsocket,
    oncontextmenu        <- ContextMenu,
    oncuechange          <- CueChange,
    ondblclick           <- DoubleClick,
    ondrag               <- Drag,
    ondragend            <- DragEnd,
    ondragenter          <- DragEnter,
    ondragexit           <- DragExit,
    ondragleave          <- DragLeave,
    ondragover           <- DragOver,
    ondragstart          <- DragStart,
    ondrop               <- Dropped,
    ondurationchange     <- DurationChange,
    onemptied            <- Emptied,
    onended              <- PlaybackEnded,
    onerror              <- ErrorEvent,
    onfocus              <- Focus,
    ongotpointercapture  <- GotPointerCapture,
    oninput              <- Input,
    oninvalid            <- Invalid,
    onkeydown            <- KeyDown,
    onkeypress           <- KeyPress,
    onkeyup              <- KeyUp,
    onload               <- ResourceLoad,
    onloadeddata         <- DataLoaded,
    onloadedmetadata     <- MetadataLoaded,
    onloadend            <- LoadEnd,
    onloadstart          <- LoadStart,
    onlostpointercapture <- LostPointerCapture,
    onmouseenter         <- MouseEnter,
    onmouseleave         <- MouseLeave,
    onmousemove          <- MouseMove,
    onmouseout           <- MouseOut,
    onmouseover          <- MouseOver,
    onmouseup            <- MouseUp,
    onpause              <- Pause,
    onplay               <- Play,
    onplaying            <- Playing,
    onpointercancel      <- PointerCancel,
    onpointerdown        <- PointerDown,
    onpointerenter       <- PointerEnter,
    onpointerleave       <- PointerLeave,
    onpointermove        <- PointerMove,
    onpointerout         <- PointerOut,
    onpointerover        <- PointerOver,
    onpointerup          <- PointerUp,
    onprogress           <- Progress,
    onratechange         <- PlaybackRateChange,
    onreset              <- FormReset,
    onresize             <- ViewResize,
    onscroll             <- Scroll,
    onseeked             <- Seeked,
    onseeking            <- Seeking,
    onselect             <- Select,
    onselectionchange    <- SelectionChange,
    onselectstart        <- SelectionStart,
    onshow               <- ContextMenuShow,
    onstalled            <- Stalled,
    onsubmit             <- Submit,
    onsuspend            <- Suspend,
    ontimeupdate         <- TimeUpdate,
    ontransitionend      <- TransitionEnd,
    onvolumechange       <- VolumeChange,
    onwaiting            <- Waiting,
    onwheel              <- Wheel,
}
