extern crate winrt_notification;

use winrt_notification::{
    InputType,
    Toast,
    ToastWithHandlers,
};

fn main() {
    let toast = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("toast with selection")
        .input(InputType::from_selection_contents(["1", "2", "3"]), "selection");
    ToastWithHandlers::new(toast)
        .on_activate(|args| {
            println!("selected: {}", args.get_user_input().expect("no event arguments")?["selection"]);
            // exit the waiting loop
            std::process::exit(0);
        })
        .show()
        .expect("unable to send notification");
    // wait for activation
    loop {}
}
