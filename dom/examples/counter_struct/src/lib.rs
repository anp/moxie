use mox::mox;
use moxie_dom::{
    elements::{html::*, text_content::Div},
    prelude::*,
};

#[derive(Clone, Debug, Default, PartialEq)]
struct Counter(i32);

#[moxie::updater(UpdateCount)]
impl Counter {
    fn increment(&mut self) {
        self.0 += 1;
    }

    fn decrement(&mut self) {
        self.0 -= 1;
    }
}

impl Stateful for Counter {
    type Output = Div;
    type Updater = UpdateCount;

    fn tick(&self, updater: UpdateCount) -> Div {
        let updater2 = updater.clone();
        mox! {
            <div>
                <button onclick={move |_| updater.decrement()}>"-"</button>
                {% "{}", self.0 }
                <button onclick={move |_| updater2.increment()}>"+"</button>
            </div>
        }
    }
}

moxie_dom::app_boot!(Counter);
