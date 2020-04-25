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
pub trait GlobalEventHandler: HtmlElement + EventTarget<Abort>
// + EventTarget<Blur>
// + EventTarget<Cancel>
// + EventTarget<Error>
// + EventTarget<Focus>
// + EventTarget<CanPlay>
// + EventTarget<CanPlayThrough>
// + EventTarget<Change>
// + EventTarget<Click>
// + EventTarget<Close>
// + EventTarget<ContextMenu>
// + EventTarget<CueChange>
// + EventTarget<DoubleClick>
// + EventTarget<Drag>
// + EventTarget<DragEnd>
// + EventTarget<DragEnter>
// + EventTarget<DragExit>
// + EventTarget<DragLeave>
// + EventTarget<DragOver>
// + EventTarget<DragStart>
// + EventTarget<Dropped>
// + EventTarget<DurationChange>
// + EventTarget<Emptied>
// + EventTarget<Ended>
// + EventTarget<FormData>
// + EventTarget<PointerCapture>
// + EventTarget<Input>
// + EventTarget<Invalid>
// + EventTarget<KeyDown>
// + EventTarget<KeyPress>
// + EventTarget<KeyUp>
// + EventTarget<Load>
// + EventTarget<LoadedData>
// + EventTarget<LoadedMetadata>
// + EventTarget<LoadEnd>
// + EventTarget<LoadStart>
// + EventTarget<LostPointerCapture>
// + EventTarget<MouseEnter>
// + EventTarget<MouseLeave>
// + EventTarget<MouseMove>
// + EventTarget<MouseOut>
// + EventTarget<MouseOver>
// + EventTarget<MouseUp>
// + EventTarget<Wheel>
// + EventTarget<Pause>
// + EventTarget<Play>
// + EventTarget<Playing>
// + EventTarget<PointerDown>
// + EventTarget<PointerMove>
// + EventTarget<PointerUp>
// + EventTarget<PointerCancel>
// + EventTarget<PointerOver>
// + EventTarget<PointerOut>
// + EventTarget<PointerEnter>
// + EventTarget<PointerLeave>
// + EventTarget<Progress>
// + EventTarget<RateChange>
// + EventTarget<Reset>
// + EventTarget<Resize>
// + EventTarget<Scroll>
// + EventTarget<Seeked>
// + EventTarget<Seeking>
// + EventTarget<Select>
// + EventTarget<SelectStart>
// + EventTarget<SelectionChange>
// + EventTarget<Show>
// + EventTarget<Sort>
// + EventTarget<Stalled>
// + EventTarget<Submit>
// + EventTarget<Suspend>
// + EventTarget<TimeUpdate>
// + EventTarget<VolumeChange>
// + EventTarget<TransitionCancel>
// + EventTarget<TransitionEnd>
// + EventTarget<TransitionRun>
// + EventTarget<TransitionStart>
// + EventTarget<Waiting>
{
    global_event!(onabort              Abort);
    // global_event!(onblur               Blur);
    // global_event!(oncancel             Cancel);
    // global_event!(onerror              Error);
    // global_event!(onfocus              Focus);
    // global_event!(oncanplay            CanPlay);
    // global_event!(oncanplaythrough     CanPlayThrough);
    // global_event!(onchange             Change);
    // global_event!(onclick              Click);
    // global_event!(onclose              Close);
    // global_event!(oncontextmenu        ContextMenu);
    // global_event!(oncuechange          CueChange);
    // global_event!(ondblclick           DoubleClick);
    // global_event!(ondrag               Drag);
    // global_event!(ondragend            DragEnd);
    // global_event!(ondragenter          DragEnter);
    // global_event!(ondragexit           DragExit);
    // global_event!(ondragleave          DragLeave);
    // global_event!(ondragover           DragOver);
    // global_event!(ondragstart          DragStart);
    // global_event!(ondrop               Drop);
    // global_event!(ondurationchange     DurationChange);
    // global_event!(onemptied            Emptied);
    // global_event!(onended              Ended);
    // global_event!(onformdata           FormData);
    // global_event!(ongotpointercapture  PointerCapture);
    // global_event!(oninput              Input);
    // global_event!(oninvalid            Invalid);
    // global_event!(onkeydown            KeyDown);
    // global_event!(onkeypress           KeyPress);
    // global_event!(onkeyup              KeyUp);
    // global_event!(onload               Load);
    // global_event!(onloadeddata         LoadedData);
    // global_event!(onloadedmetadata     LoadedMetadata);
    // global_event!(onloadend            LoadEnd);
    // global_event!(onloadstart          LoadStart);
    // global_event!(onlostpointercapture LostPointerCapture);
    // global_event!(onmouseenter         MouseEnter);
    // global_event!(onmouseleave         MouseLeave);
    // global_event!(onmousemove          MouseMove);
    // global_event!(onmouseout           MouseOut);
    // global_event!(onmouseover          MouseOver);
    // global_event!(onmouseup            MouseUp);
    // global_event!(onwheel              Wheel);
    // global_event!(onpause              Pause);
    // global_event!(onplay               Play);
    // global_event!(onplaying            Playing);
    // global_event!(onpointerdown        PointerDown);
    // global_event!(onpointermove        PointerMove);
    // global_event!(onpointerup          PointerUp);
    // global_event!(onpointercancel      PointerCancel);
    // global_event!(onpointerover        PointerOver);
    // global_event!(onpointerout         PointerOut);
    // global_event!(onpointerenter       PointerEnter);
    // global_event!(onpointerleave       PointerLeave);
    // global_event!(onprogress           Progress);
    // global_event!(onratechange         RateChange);
    // global_event!(onreset              Reset);
    // global_event!(onresize             Resize);
    // global_event!(onscroll             Scroll);
    // global_event!(onseeked             Seeked);
    // global_event!(onseeking            Seeking);
    // global_event!(onselect             Select);
    // global_event!(onselectstart        SelectStart);
    // global_event!(onselectionchange    SelectionChange);
    // global_event!(onshow               Show);
    // global_event!(onsort               Sort);
    // global_event!(onstalled            Stalled);
    // global_event!(onsubmit             Submit);
    // global_event!(onsuspend            Suspend);
    // global_event!(ontimeupdate         TimeUpdate);
    // global_event!(onvolumechange       VolumeChange);
    // global_event!(ontransitioncancel   TransitionCancel);
    // global_event!(ontransitionend      TransitionEnd);
    // global_event!(ontransitionrun      TransitionRun);
    // global_event!(ontransitionstart    TransitionStart);
    // global_event!(onwaiting            Waiting);
}
