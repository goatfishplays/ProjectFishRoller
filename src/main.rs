use async_hid::{AsyncHidRead, Device, HidBackend};
use futures_lite::StreamExt;
use vigem_client::XButtons;

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

    // BiGEmBus things
    let client = vigem_client::Client::connect().unwrap();

    // Create the virtual controller target
    let id = vigem_client::TargetId::XBOX360_WIRED;
    let mut target = vigem_client::Xbox360Wired::new(client, id);

    // Plugin the virtual controller
    target.plugin().unwrap();

    // Wait for the virtual controller to be ready to accept updates
    target.wait_ready().unwrap();

    // The input state of the virtual controller
    let mut gamepad = vigem_client::XGamepad {
        buttons: vigem_client::XButtons!(UP | RIGHT | LB | A | X),
        ..Default::default()
    };

    let mut x_old = 0.0;
    let mut y_old = 0.0;
    let mut z_old = 0.0;

    loop {
        reader.read_input_report(&mut inputs).await?;
        // tokio::time::sleep(Duration::from_millis(100)).await;
        // println!(
        //     "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        //     inputs[19] as i8,
        //     inputs[20],
        //     inputs[21] as i8,
        //     inputs[22],
        //     inputs[23] as i8,
        //     inputs[24],
        //     inputs[25],
        //     inputs[26] // inputs[19] as i8 + inputs[21] as i8 + inputs[23] as i8,
        // );

        // println!("{:?}", inputs);

        gamepad.buttons.raw = 0;

        // [0]: Buttons 1
        let b = inputs[0];
        // bit 0: square
        if bit_set(b, 0) {
            gamepad.buttons.raw |= XButtons::X;
        }
        // bit 1: x
        if bit_set(b, 1) {
            gamepad.buttons.raw |= XButtons::A;
        }
        // bit 2: o
        if bit_set(b, 2) {
            gamepad.buttons.raw |= XButtons::B;
        }
        // bit 3: triangle
        if bit_set(b, 3) {
            gamepad.buttons.raw |= XButtons::Y;
        }
        // bit 4: N/A????
        // bit 5: N/A????
        // bit 6: L2
        if bit_set(b, 6) {
            gamepad.buttons.raw |= XButtons::LB;
        }
        // bit 7: R2
        if bit_set(b, 7) {
            gamepad.buttons.raw |= XButtons::RB;
        }

        // [1]: Buttons 2
        let b = inputs[1];
        // bit 0: select
        if bit_set(b, 0) {
            gamepad.buttons.raw |= XButtons::BACK;
        }
        // bit 1: start
        if bit_set(b, 1) {
            gamepad.buttons.raw |= XButtons::START;
        }
        // bit 2: stick press
        if bit_set(b, 2) {
            gamepad.buttons.raw |= XButtons::LTHUMB;
        }
        // bit 3: N/A????
        // bit 4: Playstation Button
        if bit_set(b, 4) {
            gamepad.buttons.raw |= XButtons::GUIDE;
        }
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
        match inputs[2] {
            0 => gamepad.buttons.raw |= XButtons::UP,
            1 => gamepad.buttons.raw |= XButtons::UP | XButtons::RIGHT,
            2 => gamepad.buttons.raw |= XButtons::RIGHT,
            3 => gamepad.buttons.raw |= XButtons::DOWN | XButtons::RIGHT,
            4 => gamepad.buttons.raw |= XButtons::DOWN,
            5 => gamepad.buttons.raw |= XButtons::DOWN | XButtons::LEFT,
            6 => gamepad.buttons.raw |= XButtons::LEFT,
            7 => gamepad.buttons.raw |= XButtons::UP | XButtons::LEFT,
            _ => (),
        }

        // [3]: Stick X Axis
        // Left: 0
        // Right: 255
        gamepad.thumb_lx = (inputs[3] as i16 - 128) * 256; // cast from u8 to i16 range

        // [4]: Stick Y Axis
        // Up: 0
        // Down: 255
        gamepad.thumb_ly = (inputs[4] as i16 - 128) * -256; // cast from u8 to i16 range

        // [5-14] N/A

        // [15] Reel outwards // WHERE IS THE INWARDS dude I'm stupid, my whole life I thought you reeled the other direction
        gamepad.left_trigger = inputs[15];

        // [16] Trigger Thing
        gamepad.right_trigger = inputs[16];

        // [17-18] N/A?

        // [19] X rot?, 0 at roughly 45 deg below horis? Y accel?

        // [20] Tells if is upright(1 when spool above rod and 2 when rod above spool)

        // [21] Motion B, still not sure X accel

        // [22] Tells if is left or right(1 when spool left of rod and 2 when rod left of spool)

        // [23] X rot?, 0 at roughly 45 deg above horis? thurst back and forth?

        // [24]  idk

        // noise reduction
        let x = (inputs[19] as i8 as f32) * 0.95 + x_old * 0.05;
        let y = (inputs[21] as i8 as f32) * 0.95 + y_old * 0.05;
        let z = (inputs[23] as i8 as f32) * 0.95 + z_old * 0.05;
        // let mag = (x * x + y * y + z * z).sqrt();
        // println!("{}", mag);
        println!("{}, {}", x, y);
        gamepad.thumb_rx = (x as i16) * 256;
        gamepad.thumb_ry = (y as i16) * 256;

        // [25] N/A?

        // [26] 2?

        // Update the target
        let _ = target.update(&gamepad);

        x_old = x;
        y_old = y;
        z_old = z;
    }
}

fn bit_set(byte: u8, bit: u8) -> bool {
    byte & (1 << bit) != 0
}
