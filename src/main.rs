use async_hid::{AsyncHidRead, Device, HidBackend};
use futures_lite::StreamExt;
use vigem_client::XButtons;

const PITCH_OFFSET: f32 = 0.0;
const ROLL_OFFSET: f32 = 0.0;

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

    let mut g_x = 0.0;
    let mut g_y = 0.0;
    let mut g_z = 0.0;
    let mut pitch;
    let mut roll = 0.0;

    loop {
        reader.read_input_report(&mut inputs).await?;
        // tokio::time::sleep(Duration::from_millis(100)).await;
        // print!(
        //     "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t",
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

        let a_y = inputs[19] as i8 as f32;
        let a_x = inputs[21] as i8 as f32;
        let a_z = inputs[23] as i8 as f32;

        // grav calc and smoothing
        g_x = a_x * 0.15 + g_x * 0.85;
        g_y = a_y * 0.15 + g_y * 0.85;
        g_z = a_z * 0.15 + g_z * 0.85;

        // calc angles
        let mag = (g_x * g_x + g_y * g_y + g_z * g_z).sqrt();

        let n_x = g_x / mag;
        let n_y = g_y / mag;
        let n_z = g_z / mag;

        let new_pitch =
            f32::atan2(n_z, f32::sqrt(n_x * n_x + n_y * n_y)).to_degrees() + PITCH_OFFSET;
        if inputs[20] == 2 || f32::abs(new_pitch) < 20.0 {
            pitch = new_pitch;
        } else {
            pitch = f32::signum(new_pitch) * 90.0;
        }

        // if f32::abs(pitch) < 75.0 {
        let new_roll = f32::atan2(n_x, n_y).to_degrees() + ROLL_OFFSET;
        let roll_mult = 1.0 - f32::abs(n_z); // as it approaches high pitch causes unstable roll
        // let roll_mult = if inputs[20] == 2 {
        //     // as it approaches high pitch causes unstable roll
        //     1.0 - f32::abs(n_z)
        // } else {
        //     0.005
        // };
        roll = (1.0 - roll_mult) * roll + roll_mult * new_roll;
        // }

        // if inputs[20] == 1 {
        //     if pitch > 0.0 {
        //         pitch = 180.0 - pitch;
        //     } else {
        //         pitch = -180.0 - pitch;
        //     }
        // }

        // println!("{}, {}", pitch, roll);

        gamepad.thumb_ry = (pitch * 300.0) as i16;
        gamepad.thumb_rx = (roll * -300.0) as i16;

        // let

        // [25] N/A?

        // [26] 2?

        // Update the target
        let _ = target.update(&gamepad);
    }
}

fn bit_set(byte: u8, bit: u8) -> bool {
    byte & (1 << bit) != 0
}
