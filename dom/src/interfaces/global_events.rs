//! Trait for defining the methods related to the event handlers shared by all
//! HTML elements.

use crate::prelude::*;
use augdom::event::*;

macro_rules! global_event {
    ($property:ident $event:ident) => {
        /// Set an event handler.
        fn $property(&self, callback: impl FnMut(augdom::event::$event) + 'static) -> &Self {
            self.on(callback)
        }
    };
}

/// These event handlers are defined on the [GlobalEventHandlers][mdn] mixin,
/// and implemented by HTMLElement, Document, Window, as well as by
/// WorkerGlobalScope for Web Workers.
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/GlobalEventHandlers
pub trait GlobalEventHandler:
    HtmlElement
    + EventTarget<Abort>
    + EventTarget<Blur>
    + EventTarget<Cancel>
    + EventTarget<CanPlay>
    + EventTarget<CanPlayThrough>
    + EventTarget<Change>
    + EventTarget<Click>
    + EventTarget<CloseWebsocket>
    + EventTarget<ContextMenu>
    + EventTarget<ContextMenuShow>
    + EventTarget<CueChange>
    + EventTarget<DataLoaded>
    + EventTarget<DoubleClick>
    + EventTarget<Drag>
    + EventTarget<DragEnd>
    + EventTarget<DragEnter>
    + EventTarget<DragExit>
    + EventTarget<DragLeave>
    + EventTarget<DragOver>
    + EventTarget<DragStart>
    + EventTarget<Dropped>
    + EventTarget<DurationChange>
    + EventTarget<Emptied>
    + EventTarget<Error>
    + EventTarget<Focus>
    + EventTarget<FormReset>
    + EventTarget<GotPointerCapture>
    + EventTarget<Input>
    + EventTarget<Invalid>
    + EventTarget<KeyDown>
    + EventTarget<KeyPress>
    + EventTarget<KeyUp>
    + EventTarget<LoadEnd>
    + EventTarget<LoadStart>
    + EventTarget<LostPointerCapture>
    + EventTarget<MetadataLoaded>
    + EventTarget<MouseEnter>
    + EventTarget<MouseLeave>
    + EventTarget<MouseMove>
    + EventTarget<MouseOut>
    + EventTarget<MouseOver>
    + EventTarget<MouseUp>
    + EventTarget<Pause>
    + EventTarget<Play>
    + EventTarget<PlaybackEnded>
    + EventTarget<PlaybackRateChange>
    + EventTarget<Playing>
    + EventTarget<PointerCancel>
    + EventTarget<PointerDown>
    + EventTarget<PointerEnter>
    + EventTarget<PointerLeave>
    + EventTarget<PointerMove>
    + EventTarget<PointerOut>
    + EventTarget<PointerOver>
    + EventTarget<PointerUp>
    + EventTarget<Progress>
    + EventTarget<ResourceLoad>
    + EventTarget<Scroll>
    + EventTarget<Seeked>
    + EventTarget<Seeking>
    + EventTarget<Select>
    + EventTarget<SelectionChange>
    + EventTarget<SelectionStart>
    + EventTarget<Stalled>
    + EventTarget<Submit>
    + EventTarget<Suspend>
    + EventTarget<TimeUpdate>
    + EventTarget<TransitionEnd>
    + EventTarget<ViewResize>
    + EventTarget<VolumeChange>
    + EventTarget<Waiting>
    + EventTarget<Wheel>
{
    global_event!(onabort              Abort);
    global_event!(onblur               Blur);
    global_event!(oncancel             Cancel);
    global_event!(oncanplay            CanPlay);
    global_event!(oncanplaythrough     CanPlayThrough);
    global_event!(onchange             Change);
    global_event!(onclick              Click);
    global_event!(onclose              CloseWebsocket);
    global_event!(oncontextmenu        ContextMenu);
    global_event!(oncuechange          CueChange);
    global_event!(ondblclick           DoubleClick);
    global_event!(ondrag               Drag);
    global_event!(ondragend            DragEnd);
    global_event!(ondragenter          DragEnter);
    global_event!(ondragexit           DragExit);
    global_event!(ondragleave          DragLeave);
    global_event!(ondragover           DragOver);
    global_event!(ondragstart          DragStart);
    global_event!(ondrop               Dropped);
    global_event!(ondurationchange     DurationChange);
    global_event!(onemptied            Emptied);
    global_event!(onended              PlaybackEnded);
    global_event!(onerror              Error);
    global_event!(onfocus              Focus);
    global_event!(ongotpointercapture  GotPointerCapture);
    global_event!(oninput              Input);
    global_event!(oninvalid            Invalid);
    global_event!(onkeydown            KeyDown);
    global_event!(onkeypress           KeyPress);
    global_event!(onkeyup              KeyUp);
    global_event!(onload               ResourceLoad);
    global_event!(onloadeddata         DataLoaded);
    global_event!(onloadedmetadata     MetadataLoaded);
    global_event!(onloadend            LoadEnd);
    global_event!(onloadstart          LoadStart);
    global_event!(onlostpointercapture LostPointerCapture);
    global_event!(onmouseenter         MouseEnter);
    global_event!(onmouseleave         MouseLeave);
    global_event!(onmousemove          MouseMove);
    global_event!(onmouseout           MouseOut);
    global_event!(onmouseover          MouseOver);
    global_event!(onmouseup            MouseUp);
    global_event!(onpause              Pause);
    global_event!(onplay               Play);
    global_event!(onplaying            Playing);
    global_event!(onpointercancel      PointerCancel);
    global_event!(onpointerdown        PointerDown);
    global_event!(onpointerenter       PointerEnter);
    global_event!(onpointerleave       PointerLeave);
    global_event!(onpointermove        PointerMove);
    global_event!(onpointerout         PointerOut);
    global_event!(onpointerover        PointerOver);
    global_event!(onpointerup          PointerUp);
    global_event!(onprogress           Progress);
    global_event!(onratechange         PlaybackRateChange);
    global_event!(onreset              FormReset);
    global_event!(onresize             ViewResize);
    global_event!(onscroll             Scroll);
    global_event!(onseeked             Seeked);
    global_event!(onseeking            Seeking);
    global_event!(onselect             Select);
    global_event!(onselectionchange    SelectionChange);
    global_event!(onselectstart        SelectionStart);
    global_event!(onshow               ContextMenuShow);
    global_event!(onstalled            Stalled);
    global_event!(onsubmit             Submit);
    global_event!(onsuspend            Suspend);
    global_event!(ontimeupdate         TimeUpdate);
    global_event!(ontransitionend      TransitionEnd);
    global_event!(onvolumechange       VolumeChange);
    global_event!(onwaiting            Waiting);
    global_event!(onwheel              Wheel);
}
