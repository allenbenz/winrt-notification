extern crate winrt_notification;
use winrt_notification::{
    Toast,
    ToastWithHandlers,
};

fn main() {
    let toast = Toast::new(Toast::POWERSHELL_APP_ID).title("toast with handlers");
    ToastWithHandlers::new(toast)
        .on_activate(|_| {
            println!("activated");
            // exit the waiting loop
            std::process::exit(0);
        })
        .on_dismiss(|_, _| {
            println!("dismissed");
            // exit the waiting loop
            std::process::exit(0);
        })
        .on_fail(|_, _| {
            println!("failed!");
            // exit the waiting loop
            std::process::exit(0);
        })
        .show()
        .expect("unable to send notification");
    // wait for some event
    loop {}
}
