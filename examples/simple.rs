extern crate winrt_notification;
use winrt_notification::{Duration, Sound, Toast};

fn main() {
    Toast::new("My application name")
        .title("Look at this flip!")
        .text1("(╯°□°）╯︵ ┻━┻")
        .sound(Some(Sound::SMS))
        .duration(Duration::Short)
        .show()
        .expect("unable to send notification");
}
