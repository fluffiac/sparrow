//! Memory reading operations
//!
//! This module is organized into individual memory reading functions, which
//! communicate back to the main `HdMemReader` through a `Task`.

use std::io;
use std::time::Duration;

use async_io::Timer;
use futures_lite::future::zip;
use win_wrap::TVec;

use crate::MemReader;

use crate::builder::{mem_builder, Task};
use crate::util::{Interval, Never};

const FRAME: Duration = Duration::from_micros(8333);
const SECOND: Duration = Duration::from_secs(1);

mem_builder! {
    pub struct HdMemReader = MemBuilder::new()
        .with::<Vec<u32>>(queue_ids)
        .with::<(u32, u32)>(multiplayer_score)
        .build();
}

async fn queue_ids(mem: MemReader, mut task: Task<Vec<u32>>) -> io::Result<Never> {
    let mut tick = Interval::every(SECOND);

    let root = mem.startup_offset([0x3D61D0, 0x18, 0x30, 0x30, 0x68]).await;

    let start_ptr = root.add(0x100);
    let end_ptr = root.add(0x108);

    let mut tvec = TVec::from(Vec::<u32>::with_capacity(200));

    loop {
        tick.wait().await;

        let (s_res, e_res) = zip(start_ptr.offset(0), end_ptr.offset(0)).await;
        let (start_loc, end_loc) = (s_res?, e_res?);

        let length = (end_loc.addr - start_loc.addr) / 4;

        start_loc.vec_read(&mut tvec, length).await?;

        task.update(&*tvec.inner());
    }
}

async fn multiplayer_score(mem: MemReader, mut task: Task<(u32, u32)>) -> io::Result<Never> {
    // todo: There is a mysterious bug which causes the memory reader
    //       to not exit on application exit. I think it may be the winAPI blocking
    //       the async executor, but I'm not sure. This is a temporary fix to offset
    //       the two pollers so that the app can exit.
    Timer::after(SECOND / 2).await;

    let mut tick = Interval::every(SECOND);

    let root = mem.startup_offset([0x3D61D0, 0x00, 0x30, 0x60, 0x30, 0x28]).await;

    let score_loc_1 = root.add(0x8E4);
    let score_loc_2 = root.add(0x8E8);

    loop {
        tick.wait().await;

        let (res_1, res_2) = zip(score_loc_1.read(), score_loc_2.read()).await;
        let (score_1, score_2) = (res_1?, res_2?);

        task.update(&(score_1, score_2));
    }
}

// async fn pace(mem: MemReader, mut task: Task<Vec<f32>>) -> io::Result<Never> {
//     let mut tick = Interval::every(FRAME);

//     let pace_loc = mem.startup_offset([0x3D61D0, 0x00, 0x20, 0xb8]).await;
//     let len_loc = pace_loc.add(0x240);

//     let mut len: usize = 999;
//     let mut tvec = TVec::from(Vec::<f32>::with_capacity(155));

//     loop {
//         tick.wait().await;

//         let inc_len = len_loc.read::<u32>().await? as usize;
//         if len != inc_len {
//             let pace_loc = pace_loc.offset(0x228).await?;

//             pace_loc.vec_read(&mut tvec, inc_len).await?;

//             len = inc_len;
//             task.update(&tvec.inner());
//         }
//     }
// }

// async fn training_info(mem: MemReader, mut task: Task<Vec<u32>>) -> io::Result<Never> {
//     let mut tick = Interval::every(FRAME);

//     let info_loc = mem
//         .startup_offset([0x3D61D0, 0x00, 0x20, 0xb8, 0x08])
//         .await
//         .add(0x84);

//     loop {
//         tick.wait().await;

//         let prog = info_loc.read::<u32>().await?;

//         task.update(&vec![prog]);
//     }
// }
