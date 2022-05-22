extern crate winrt_notification;
use winrt_notification::{Header, Toast};

fn main() {
    let header = Header::from_title("Hello World");
    Toast::new(Toast::POWERSHELL_APP_ID)
        .text1("Hello")
        .header(header.clone())
        .show()
        .expect("unable to send notification");

    Toast::new(Toast::POWERSHELL_APP_ID)
        .text1("World")
        .header(header)
        .show()
        .expect("unable to send notification");
}
