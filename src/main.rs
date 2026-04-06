use std::time::Duration;

use async_hid::{AsyncHidRead, Device, HidBackend};
use futures_lite::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hid_backend = HidBackend::default();
    let mut devices = hid_backend.enumerate().await?;

    // // view all devices
    // while let Some(device) = devices.next().await {
    //     println!("{:?}", device);
    // }

    // get rod
    let mut reader = devices
        .find(|device: &Device| device.matches(1, 5, 4794, 1200)) // ! idk if this is same for all rods or just mine also might detect guitar hero controllers lmao
        .await
        .expect("Couldn't find device")
        .open_readable()
        .await?;
    println!("Found fishing rod probably");

    // buffer for inputs
    let mut inputs = [0u8; 27];

    loop {
        reader.read_input_report(&mut inputs).await?;
        // tokio::time::sleep(Duration::from_millis(100)).await;
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            inputs[19],
            inputs[20],
            inputs[21],
            inputs[22],
            inputs[23],
            inputs[24],
            inputs[25],
            inputs[26]
        );
        // println!(
        //     "{}\t{}\t{}\t{}",
        //     (inputs[19] as u16) * (inputs[20] as u16),
        //     (inputs[21] as u16) * (inputs[22] as u16),
        //     (inputs[23] as u16) * (inputs[24] as u16),
        //     (inputs[25] as u16) * (inputs[26] as u16),
        // );
        // println!("{:?}", inputs);
        // [0]: Buttons 1
        // bit 0: square
        // bit 1: x
        // bit 2: o
        // bit 3: triangle
        // bit 4: N/A????
        // bit 5: N/A????
        // bit 6: L2
        // bit 7: R2

        // [1]: Buttons 2
        // bit 0: select
        // bit 1: start
        // bit 2: stick press
        // bit 3: N/A????
        // bit 4: Playstation Button
        // bit 5: N/A????
        // bit 6: N/A????
        // bit 7: N/A????

        // [2]: Dpad
        // None: 15
        // U: 0
        // UR: 1
        // R: 2
        // DR: 3
        // D: 4
        // DL: 5
        // L: 6
        // UL: 7

        // [3]: Stick X Axis
        // Left: 0
        // Right: 255

        // [4]: Stick Y Axis
        // Up: 0
        // Down: 255

        // [5-14] N/A

        // [15] Reel outwards // WHERE IS THE INWARDS dude I'm stupid, my whole life I thought you reeled the other direction

        // [16] Trigger Thing

        // [17-18] N/A?

        // [19] X rot?, 0 at roughly 45 deg below horis? Y accel?

        // [20] Tells if is upright(1 when spool above rod and 2 when rod above spool)

        // [21] Motion B, still not sure X accel

        // [22] Tells if is left or right(1 when spool left of rod and 2 when rod left of spool)

        // [23] X rot?, 0 at roughly 45 deg above horis? thurst back and forth?

        // [24]

        // [25] N/A?

        // [26] 2?
    }
}
// 0, 0, 15, 130, 134, 128, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 253, 1, 217, 1, 6, 2, 0, 2
