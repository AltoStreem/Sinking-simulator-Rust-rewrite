//! TimeSync$1.class: report, then delay 1000ms, then recheck the continuation flag.
#![allow(dead_code)]
use crate::{executor_kt::SingleThreadDispatcher, time_sync::ReportState};
use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};
pub(crate) fn launch(state: Arc<ReportState>, dispatcher: Arc<SingleThreadDispatcher>) {
    let next_dispatcher = dispatcher.clone();
    dispatcher
        .execute(move || step(state, next_dispatcher))
        .expect("TimeSync dispatcher rejected initial report");
}
fn step(state: Arc<ReportState>, dispatcher: Arc<SingleThreadDispatcher>) {
    if !state.running.load(Ordering::SeqCst) {
        return;
    }
    let resource = state.take_resource_average();
    let speed = state.take_speed_average();
    (state.callback)(resource, speed);
    let next_dispatcher = dispatcher.clone();
    dispatcher
        .schedule(Duration::from_millis(1000), move || {
            step(state, next_dispatcher)
        })
        .expect("TimeSync dispatcher rejected delayed report");
}
